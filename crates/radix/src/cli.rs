//! Target-independent CLI surface analysis.
//!
//! Faber source can declare command-line programs with annotations such as
//! `@ cli`, `@ imperium`, `@ optio`, and `@ operandus`. This module turns those
//! annotations into a normalized CLI IR that later compiler phases and codegen
//! can consume without re-reading syntax details from the AST.
//!
//! The analyzer is deliberately target-independent. It validates the language
//! contract for command paths, option names, operand ordering, global surface
//! placement, mounted command modules, and supported CLI-facing types, but it
//! does not decide how Rust, TypeScript, or another backend should parse argv.
//!
//! INVARIANTS
//! ==========
//! - A root CLI program has at most one `@ cli` incipit entry point.
//! - Command modules mounted into another CLI may expose `@ imperium` commands
//!   but must not declare their own root `@ cli` entry point.
//! - Global options and operands use `ubique` and are owned by the root entry
//!   point, not by individual commands.
//! - Validation errors are semantic diagnostics; analysis still returns the
//!   best normalized shape it can construct for tooling and inspection.

use crate::lexer::{Interner, Span};
use crate::semantic::{SemanticError, SemanticErrorKind};
use crate::syntax::{
    Annotation, AnnotationKind, Expr, ExprKind, FuncDecl, Ident, Literal, OperandusAnnotation, OptioAnnotation,
    Program, Stmt, StmtKind, TypeExpr, TypeExprKind,
};
use rustc_hash::{FxHashMap, FxHashSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliMode {
    /// Program contains no CLI entry point or mounted command surface.
    NotCli,

    /// One `@ cli` incipit owns the complete option and operand surface.
    SingleCommand,

    /// Root entry point dispatches to one or more `@ imperium` commands.
    Subcommand,
}

/// Result of extracting a CLI surface from one parsed program.
///
/// The analyzer separates structural mode from the optional normalized program
/// because mounted modules may be non-CLI files, and invalid CLI files still
/// need diagnostics even when a partial IR exists.
#[derive(Debug, Clone)]
pub struct CliAnalysis {
    /// Classification of the parsed source as a CLI surface.
    pub mode: CliMode,

    /// Normalized CLI contract, present when a root or mounted command surface
    /// was found.
    pub program: Option<CliProgram>,

    /// Semantic validation errors found while reading CLI annotations.
    pub errors: Vec<SemanticError>,
}

/// Normalized command-line program contract consumed by codegen and tooling.
///
/// This type intentionally models the CLI surface, not the implementation body.
/// Bindings point back to Faber argument records and command functions so later
/// phases can generate parser glue without depending on annotation syntax.
#[derive(Debug, Clone)]
pub struct CliProgram {
    /// User-facing command name from the root `@ cli` annotation.
    pub name: String,

    /// Binding name used by the root `incipit argumenta <ident>` declaration.
    pub entry_args: String,

    /// Whether this surface is a single command or subcommand dispatcher.
    pub mode: CliMode,

    /// Optional version metadata declared with `@ versio`.
    pub version: Option<String>,

    /// Optional long description declared with `@ descriptio`.
    pub description: Option<String>,

    /// Options available to every command through `ubique`.
    pub global_options: Vec<CliOption>,

    /// Operands available to every command through `ubique`.
    pub global_operands: Vec<CliOperand>,

    /// Single-command local options declared on the root entry point.
    pub options: Vec<CliOption>,

    /// Single-command local operands declared on the root entry point.
    pub operands: Vec<CliOperand>,

    /// Subcommands declared by `@ imperium` functions.
    pub commands: Vec<CliCommand>,

    /// Root exit expression reduced to the forms codegen currently supports.
    pub exit: Option<CliExit>,
}

/// Exit-code expression captured from a CLI `incipit`.
///
/// The variants preserve enough structure for codegen to emit stable exit
/// handling while refusing to invent meaning for arbitrary expressions.
#[derive(Debug, Clone)]
pub enum CliExit {
    /// Literal numeric exit status.
    Fixed(i64),

    /// Exit status comes from a local binding.
    Binding(String),

    /// Exit status comes from a field on a local binding.
    Field { object: String, field: String },

    /// Expression was present but cannot be lowered as a CLI exit contract.
    Unsupported,
}

/// Normalized subcommand declaration.
///
/// A command path is already split into routing segments. Mounted modules may
/// prefix the path and aliases after local validation, which is why this type
/// carries both source-local function identity and user-facing route metadata.
#[derive(Debug, Clone)]
pub struct CliCommand {
    /// Slash-separated route segments, stored normalized for collision checks.
    pub path: Vec<String>,

    /// Module path when this command came from an imported command module.
    pub module_path: Option<Vec<String>>,

    /// Faber function name that implements the command.
    pub function: String,

    /// Interned function symbol for later semantic/codegen lookup.
    pub function_symbol: crate::lexer::Symbol,

    /// Optional `argumenta <ident>` binding used by the command function.
    pub args_binding: Option<String>,

    /// Alternate route names declared with `@ alias`.
    pub aliases: Vec<String>,

    /// Optional user-facing command description.
    pub description: Option<String>,

    /// Options local to this command.
    pub options: Vec<CliOption>,

    /// Positional operands local to this command.
    pub operands: Vec<CliOperand>,

    /// Source span used for command-level diagnostics.
    pub span: Span,
}

/// Normalized option declared with `@ optio`.
#[derive(Debug, Clone)]
pub struct CliOption {
    /// Faber binding name that receives the parsed value.
    pub binding: String,

    /// Interned binding symbol for semantic/codegen lookup.
    pub binding_symbol: crate::lexer::Symbol,

    /// CLI-supported value type after syntax normalization.
    pub ty: CliType,

    /// Single-character short flag without leading `-`.
    pub short: Option<String>,

    /// Long flag without leading `--`.
    pub long: Option<String>,

    /// Optional user-facing help text.
    pub description: Option<String>,

    /// Whether the option is global through `ubique`.
    pub global: bool,

    /// Default value declared in source or synthesized for boolean flags.
    pub default: Option<CliDefault>,

    /// True when the option is a boolean flag rather than value-taking option.
    pub flag: bool,

    /// Source span used for option diagnostics.
    pub span: Span,
}

/// Normalized positional operand declared with `@ operandus`.
#[derive(Debug, Clone)]
pub struct CliOperand {
    /// Faber binding name that receives the parsed value.
    pub binding: String,

    /// Interned binding symbol for semantic/codegen lookup.
    pub binding_symbol: crate::lexer::Symbol,

    /// CLI-supported value type after syntax normalization.
    pub ty: CliType,

    /// Whether this operand captures the remaining positional arguments.
    pub rest: bool,

    /// Optional user-facing help text.
    pub description: Option<String>,

    /// Whether the operand is global through `ubique`.
    pub global: bool,

    /// Default value declared in source.
    pub default: Option<CliDefault>,

    /// Source span used for operand diagnostics.
    pub span: Span,
}

/// Faber types currently accepted at the CLI boundary.
///
/// The CLI type set is narrower than the language type system because argv
/// parsing needs an explicit, predictable conversion contract. Unsupported
/// types are reported during analysis instead of being guessed in codegen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliType {
    /// `textus`
    Textus,

    /// `numerus`
    Numerus,

    /// `fractus`
    Fractus,

    /// `bivalens`
    Bivalens,

    /// `octeti`
    Octeti,

    /// `ignotum`
    Ignotum,

    /// `lista<textus>`
    ListaTextus,

    /// `lista<numerus>`
    ListaNumerus,
}

/// Default value captured from CLI annotation syntax.
///
/// Literal defaults keep typed shape. Non-literal expressions are preserved as
/// debug text so inspection surfaces can explain what was seen while validation
/// and codegen decide whether the form is supported.
#[derive(Debug, Clone)]
pub enum CliDefault {
    /// String literal default.
    Text(String),

    /// Integer literal default.
    Integer(i64),

    /// Floating-point literal default.
    Float(f64),

    /// Boolean literal default.
    Bool(bool),

    /// `nihil` default.
    Nil,

    /// Non-literal expression captured for diagnostics/inspection.
    Expr(String),
}

/// Analyze a parsed root program for a CLI surface.
pub fn analyze(program: &Program, interner: &Interner) -> CliAnalysis {
    let mut builder = CliBuilder { interner, errors: Vec::new() };
    builder.analyze(program)
}

/// Analyze an imported command module and prefix its exposed routes.
///
/// Mounted modules are only allowed to contribute subcommands. They cannot
/// declare a root `@ cli` entry point or nested command mounts because the root
/// package owns dispatch topology.
pub fn analyze_mounted_module(program: &Program, interner: &Interner, mount_prefix: &[String]) -> CliAnalysis {
    let mut builder = CliBuilder { interner, errors: Vec::new() };
    builder.analyze_mounted_module(program, mount_prefix)
}

struct CliBuilder<'a> {
    interner: &'a Interner,
    errors: Vec<SemanticError>,
}

impl CliBuilder<'_> {
    fn analyze(&mut self, program: &Program) -> CliAnalysis {
        let cli_stmt = self.find_cli_stmt(program);
        let Some((entry_stmt, cli_name)) = cli_stmt else {
            return CliAnalysis { mode: CliMode::NotCli, program: None, errors: Vec::new() };
        };

        let entry = match &entry_stmt.kind {
            StmtKind::Incipit(incipit) => incipit,
            _ => unreachable!("find_cli_stmt only returns incipit statements"),
        };

        let entry_args = match &entry.args {
            Some(args) => self.ident(args),
            None => {
                self.error(
                    entry_stmt.span,
                    "@ cli programs must use an explicit 'incipit argumenta <ident>' entry point",
                );
                String::new()
            }
        };

        let mut globals = CliSurface::default();
        let mut single = CliSurface::default();
        self.collect_surface(&entry_stmt.annotations, SurfacePlacement::TopLevel, &mut globals, &mut single);

        let commands = self.collect_commands(program);
        let mode = if commands.is_empty() {
            CliMode::SingleCommand
        } else {
            if !single.options.is_empty() || !single.operands.is_empty() {
                self.error(
                    entry_stmt.span,
                    "subcommand CLI programs may only declare top-level options or operands with 'ubique'",
                );
            }
            if !incipit_body_is_empty(&entry.body) {
                self.error(
                    entry_stmt.span,
                    "subcommand CLI programs must use an empty root incipit body in Phase 04",
                );
            }
            CliMode::Subcommand
        };

        self.validate_surface("top-level", &globals.options, &globals.operands);
        self.validate_surface("single-command", &single.options, &single.operands);
        if mode == CliMode::SingleCommand {
            self.validate_global_surface_collisions("single-command", &single, &globals);
        }
        self.validate_commands(&commands, &globals, false);

        let cli_program = CliProgram {
            name: self.resolve(cli_name),
            entry_args,
            mode: mode.clone(),
            version: string_metadata(&entry_stmt.annotations, self.interner, "versio"),
            description: string_metadata(&entry_stmt.annotations, self.interner, "descriptio"),
            global_options: globals.options,
            global_operands: globals.operands,
            options: single.options,
            operands: single.operands,
            commands,
            exit: entry.exitus.as_deref().map(|expr| self.exit(expr)),
        };

        CliAnalysis { mode, program: Some(cli_program), errors: std::mem::take(&mut self.errors) }
    }

    fn analyze_mounted_module(&mut self, program: &Program, mount_prefix: &[String]) -> CliAnalysis {
        if let Some((stmt, _)) = self.find_cli_stmt(program) {
            self.error(
                stmt.span,
                "mounted CLI command modules must not declare their own @ cli entry point",
            );
        }
        self.reject_nested_module_mounts(program);

        let mut commands = self.collect_commands(program);
        self.validate_commands(&commands, &CliSurface::default(), false);
        for command in &mut commands {
            let local_path = std::mem::take(&mut command.path);
            command.path = mount_prefix.iter().cloned().chain(local_path).collect();
            command.aliases = command
                .aliases
                .iter()
                .map(|alias| {
                    mount_prefix
                        .iter()
                        .cloned()
                        .chain(
                            alias
                                .split('/')
                                .filter(|part| !part.is_empty())
                                .map(str::to_owned),
                        )
                        .collect::<Vec<_>>()
                        .join("/")
                })
                .collect();
        }

        self.validate_commands(&commands, &CliSurface::default(), true);

        let mode = if commands.is_empty() {
            CliMode::NotCli
        } else {
            CliMode::Subcommand
        };
        let program = if commands.is_empty() {
            None
        } else {
            Some(CliProgram {
                name: String::new(),
                entry_args: String::new(),
                mode: CliMode::Subcommand,
                version: None,
                description: None,
                global_options: Vec::new(),
                global_operands: Vec::new(),
                options: Vec::new(),
                operands: Vec::new(),
                commands,
                exit: None,
            })
        };

        CliAnalysis { mode, program, errors: std::mem::take(&mut self.errors) }
    }

    fn find_cli_stmt<'a>(&mut self, program: &'a Program) -> Option<(&'a Stmt, crate::lexer::Symbol)> {
        let mut found = None;
        for stmt in &program.stmts {
            for annotation in &stmt.annotations {
                if let AnnotationKind::Cli(cli) = &annotation.kind {
                    if !matches!(stmt.kind, StmtKind::Incipit(_)) {
                        self.error(annotation.span, "@ cli may only annotate an incipit entry point");
                    }
                    if found.is_some() {
                        self.error(annotation.span, "only one @ cli entry point is allowed");
                    } else {
                        found = Some((stmt, cli.name));
                    }
                }
            }
        }
        found
    }

    fn collect_commands(&mut self, program: &Program) -> Vec<CliCommand> {
        let mut commands = Vec::new();
        for stmt in &program.stmts {
            let Some(command_name) = imperium_annotation(&stmt.annotations) else {
                if !has_cli_annotation(&stmt.annotations) {
                    self.reject_cli_surface_without_owner(stmt);
                }
                continue;
            };

            let StmtKind::Func(func) = &stmt.kind else {
                self.error(stmt.span, "@ imperium may only annotate a functio declaration");
                continue;
            };

            let mut surface = CliSurface::default();
            let mut globals = CliSurface::default();
            self.collect_surface(&stmt.annotations, SurfacePlacement::Command, &mut globals, &mut surface);
            let path = self.command_path(command_name, stmt.span);
            let command = CliCommand {
                path,
                module_path: None,
                function: self.ident(&func.name),
                function_symbol: func.name.name,
                args_binding: command_argument_binding(&func.modifiers).map(|ident| self.ident(ident)),
                aliases: string_metadata_all(&stmt.annotations, self.interner, "alias"),
                description: string_metadata(&stmt.annotations, self.interner, "descriptio"),
                options: surface.options,
                operands: surface.operands,
                span: stmt.span,
            };
            self.validate_command_signature(func, stmt.span);
            if !globals.options.is_empty() || !globals.operands.is_empty() {
                self.error(
                    stmt.span,
                    "'ubique' options and operands must be declared on the @ cli entry point",
                );
            }
            commands.push(command);
        }
        commands
    }

    fn reject_cli_surface_without_owner(&mut self, stmt: &Stmt) {
        for annotation in &stmt.annotations {
            match &annotation.kind {
                AnnotationKind::Optio(_) | AnnotationKind::Operandus(_) => {
                    self.error(
                        annotation.span,
                        "@ optio and @ operandus must annotate @ cli incipit or @ imperium functio",
                    );
                }
                _ => {}
            }
        }
    }

    fn reject_nested_module_mounts(&mut self, program: &Program) {
        for stmt in &program.stmts {
            for annotation in &stmt.annotations {
                if let AnnotationKind::Statement(annotation_stmt) = &annotation.kind {
                    if self.interner.resolve(annotation_stmt.name.name) == "imperia" {
                        self.error(
                            annotation.span,
                            "@ imperia module mounts may only be declared on the root CLI entry point",
                        );
                    }
                }
            }
        }
    }

    fn collect_surface(
        &mut self,
        annotations: &[Annotation],
        placement: SurfacePlacement,
        globals: &mut CliSurface,
        local: &mut CliSurface,
    ) {
        for annotation in annotations {
            match &annotation.kind {
                AnnotationKind::Optio(optio) => {
                    let option = self.option(optio, annotation.span);
                    if option.global {
                        if placement != SurfacePlacement::TopLevel {
                            self.error(annotation.span, "'ubique' options must be declared on the @ cli entry point");
                        }
                        globals.options.push(option);
                    } else {
                        local.options.push(option);
                    }
                }
                AnnotationKind::Operandus(operandus) => {
                    let operand = self.operand(operandus, annotation.span);
                    if operand.global {
                        if placement != SurfacePlacement::TopLevel {
                            self.error(annotation.span, "'ubique' operands must be declared on the @ cli entry point");
                        }
                        globals.operands.push(operand);
                    } else {
                        local.operands.push(operand);
                    }
                }
                _ => {}
            }
        }
    }

    fn option(&mut self, optio: &OptioAnnotation, span: Span) -> CliOption {
        let ty = match &optio.ty {
            Some(ty) => self.cli_type(ty, span),
            None => CliType::Textus,
        };
        let default = optio
            .default
            .as_deref()
            .map(|expr| self.default(expr))
            .or_else(|| {
                if ty == CliType::Bivalens {
                    Some(CliDefault::Bool(false))
                } else {
                    None
                }
            });
        CliOption {
            binding: self.ident(&optio.binding),
            binding_symbol: optio.binding.name,
            flag: ty == CliType::Bivalens,
            ty,
            short: optio.short.map(|sym| self.resolve(sym)),
            long: optio.long.map(|sym| self.resolve(sym)),
            description: optio.description.map(|sym| self.resolve(sym)),
            global: optio.global,
            default,
            span,
        }
    }

    fn operand(&mut self, operandus: &OperandusAnnotation, span: Span) -> CliOperand {
        CliOperand {
            binding: self.ident(&operandus.binding),
            binding_symbol: operandus.binding.name,
            ty: self.cli_type(&operandus.ty, span),
            rest: operandus.rest,
            description: operandus.description.map(|sym| self.resolve(sym)),
            global: operandus.global,
            default: operandus.default.as_deref().map(|expr| self.default(expr)),
            span,
        }
    }

    fn cli_type(&mut self, ty: &TypeExpr, span: Span) -> CliType {
        match &ty.kind {
            TypeExprKind::Named(name, params) if params.is_empty() => match self.interner.resolve(name.name) {
                "textus" => CliType::Textus,
                "numerus" => CliType::Numerus,
                "fractus" => CliType::Fractus,
                "bivalens" => CliType::Bivalens,
                "octeti" => CliType::Octeti,
                "ignotum" => CliType::Ignotum,
                other => {
                    self.error(span, format!("unsupported CLI type '{other}'"));
                    CliType::Textus
                }
            },
            TypeExprKind::Named(name, params) if self.interner.resolve(name.name) == "lista" && params.len() == 1 => {
                match &params[0].kind {
                    TypeExprKind::Named(inner, inner_params) if inner_params.is_empty() => {
                        match self.interner.resolve(inner.name) {
                            "textus" => CliType::ListaTextus,
                            "numerus" => CliType::ListaNumerus,
                            other => {
                                self.error(span, format!("unsupported CLI list element type '{other}'"));
                                CliType::ListaTextus
                            }
                        }
                    }
                    _ => {
                        self.error(span, "CLI list types must use a named element type");
                        CliType::ListaTextus
                    }
                }
            }
            _ => {
                self.error(span, "unsupported CLI type shape");
                CliType::Textus
            }
        }
    }

    fn default(&self, expr: &Expr) -> CliDefault {
        match &expr.kind {
            ExprKind::Literal(Literal::String(sym)) => CliDefault::Text(self.resolve(*sym)),
            ExprKind::Literal(Literal::Integer(value)) => CliDefault::Integer(*value),
            ExprKind::Literal(Literal::Float(value)) => CliDefault::Float(*value),
            ExprKind::Literal(Literal::Bool(value)) => CliDefault::Bool(*value),
            ExprKind::Literal(Literal::Nil) => CliDefault::Nil,
            other => CliDefault::Expr(format!("{other:?}")),
        }
    }

    fn exit(&self, expr: &Expr) -> CliExit {
        match &expr.kind {
            ExprKind::Literal(Literal::Integer(value)) => CliExit::Fixed(*value),
            ExprKind::Ident(ident) => CliExit::Binding(self.ident(ident)),
            ExprKind::Member(member) => {
                if let ExprKind::Ident(object) = &member.object.kind {
                    CliExit::Field { object: self.ident(object), field: self.ident(&member.member) }
                } else {
                    CliExit::Unsupported
                }
            }
            ExprKind::Paren(inner) => self.exit(inner),
            _ => CliExit::Unsupported,
        }
    }

    fn validate_surface(&mut self, label: &str, options: &[CliOption], operands: &[CliOperand]) {
        let mut bindings = FxHashMap::default();
        let mut shorts = FxHashMap::default();
        let mut longs = FxHashMap::default();

        for option in options {
            self.validate_binding(label, &option.binding, option.span, &mut bindings);
            if option.short.is_none() && option.long.is_none() {
                self.error(
                    option.span,
                    format!("{label} option '{}' needs brevis or longum", option.binding),
                );
            }
            if let Some(short) = &option.short {
                if short.chars().count() != 1 || short.starts_with('-') {
                    self.error(
                        option.span,
                        format!("invalid short flag '{short}' for option '{}'", option.binding),
                    );
                }
                if shorts.insert(short.clone(), option.span).is_some() {
                    self.error(option.span, format!("duplicate short flag '{short}' in {label} CLI surface"));
                }
            }
            if let Some(long) = &option.long {
                if long.is_empty() || long.starts_with('-') || long.chars().any(char::is_whitespace) {
                    self.error(
                        option.span,
                        format!("invalid long flag '{long}' for option '{}'", option.binding),
                    );
                }
                if longs.insert(long.clone(), option.span).is_some() {
                    self.error(option.span, format!("duplicate long flag '{long}' in {label} CLI surface"));
                }
            }
        }

        let mut seen_rest = false;
        for (index, operand) in operands.iter().enumerate() {
            self.validate_binding(label, &operand.binding, operand.span, &mut bindings);
            if seen_rest {
                self.error(
                    operand.span,
                    format!("operand '{}' appears after ceteri operand", operand.binding),
                );
            }
            if operand.rest {
                if index + 1 != operands.len() {
                    self.error(operand.span, "ceteri operand must be the final operand");
                }
                if seen_rest {
                    self.error(operand.span, "only one ceteri operand is allowed");
                }
                seen_rest = true;
            }
        }
    }

    fn validate_binding(&mut self, label: &str, binding: &str, span: Span, bindings: &mut FxHashMap<String, Span>) {
        if bindings.insert(binding.to_owned(), span).is_some() {
            self.error(span, format!("duplicate CLI binding '{binding}' in {label} CLI surface"));
        }
    }

    fn validate_commands(&mut self, commands: &[CliCommand], globals: &CliSurface, allow_alias_paths: bool) {
        let mut paths = FxHashSet::default();
        for command in commands {
            let path = command.path.join("/");
            if !paths.insert(path.clone()) {
                self.error(command.span, format!("duplicate command path '{path}'"));
            }
        }

        let mut aliases = FxHashSet::default();
        for command in commands {
            let path = command.path.join("/");
            for alias in &command.aliases {
                if alias_path(alias).is_empty() || (!allow_alias_paths && alias.contains('/')) {
                    self.error(command.span, format!("invalid command alias '{alias}'"));
                }
                if !aliases.insert(alias.clone()) {
                    self.error(command.span, format!("duplicate command alias '{alias}'"));
                }
                if paths.contains(alias) {
                    self.error(command.span, format!("command alias '{alias}' collides with a command path"));
                }
            }
            self.validate_surface(&format!("command '{path}'"), &command.options, &command.operands);
            self.validate_global_collisions(command, globals);
        }
    }

    fn validate_global_collisions(&mut self, command: &CliCommand, globals: &CliSurface) {
        self.validate_global_surface_collisions(&format!("command '{}'", command.path.join("/")), command, globals);
    }

    fn validate_global_surface_collisions(&mut self, label: &str, local: &impl CliSurfaceView, globals: &CliSurface) {
        let mut global_bindings = FxHashSet::default();
        for option in &globals.options {
            global_bindings.insert(option.binding.as_str());
        }
        for operand in &globals.operands {
            global_bindings.insert(operand.binding.as_str());
        }
        for option in local.options() {
            if global_bindings.contains(option.binding.as_str()) {
                self.error(
                    option.span,
                    format!("{label} option '{}' collides with a global CLI binding", option.binding),
                );
            }
        }
        for operand in local.operands() {
            if global_bindings.contains(operand.binding.as_str()) {
                self.error(
                    operand.span,
                    format!("{label} operand '{}' collides with a global CLI binding", operand.binding),
                );
            }
        }
    }

    fn validate_command_signature(&mut self, func: &FuncDecl, span: Span) {
        if !func.params.is_empty() {
            self.error(
                span,
                "Phase 04 CLI commands must receive parsed values through 'argumenta <ident>', not ordinary parameters",
            );
        }
    }

    fn command_path(&mut self, name: crate::lexer::Symbol, span: Span) -> Vec<String> {
        let raw = self.resolve(name);
        let parts = raw
            .split('/')
            .filter(|part| !part.is_empty())
            .map(str::to_owned)
            .collect::<Vec<_>>();
        if parts.is_empty() || raw.starts_with('/') || raw.ends_with('/') || raw.contains("//") {
            self.error(span, format!("invalid command path '{raw}'"));
        }
        parts
    }

    fn ident(&self, ident: &Ident) -> String {
        self.resolve(ident.name)
    }

    fn resolve(&self, symbol: crate::lexer::Symbol) -> String {
        self.interner.resolve(symbol).to_owned()
    }

    fn error(&mut self, span: Span, message: impl Into<String>) {
        self.errors
            .push(SemanticError::new(SemanticErrorKind::CliValidation, message, span));
    }
}

fn alias_path(alias: &str) -> Vec<String> {
    alias
        .split('/')
        .filter(|part| !part.is_empty())
        .map(str::to_owned)
        .collect()
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SurfacePlacement {
    TopLevel,
    Command,
}

#[derive(Default)]
struct CliSurface {
    options: Vec<CliOption>,
    operands: Vec<CliOperand>,
}

trait CliSurfaceView {
    fn options(&self) -> &[CliOption];
    fn operands(&self) -> &[CliOperand];
}

impl CliSurfaceView for CliSurface {
    fn options(&self) -> &[CliOption] {
        &self.options
    }

    fn operands(&self) -> &[CliOperand] {
        &self.operands
    }
}

impl CliSurfaceView for CliCommand {
    fn options(&self) -> &[CliOption] {
        &self.options
    }

    fn operands(&self) -> &[CliOperand] {
        &self.operands
    }
}

fn imperium_annotation(annotations: &[Annotation]) -> Option<crate::lexer::Symbol> {
    annotations
        .iter()
        .find_map(|annotation| match &annotation.kind {
            AnnotationKind::Imperium(imperium) => Some(imperium.name),
            _ => None,
        })
}

fn has_cli_annotation(annotations: &[Annotation]) -> bool {
    annotations
        .iter()
        .any(|annotation| matches!(annotation.kind, AnnotationKind::Cli(_)))
}

fn command_argument_binding(modifiers: &[crate::syntax::FuncModifier]) -> Option<&Ident> {
    modifiers.iter().find_map(|modifier| match modifier {
        crate::syntax::FuncModifier::Argumenta(ident) => Some(ident),
        _ => None,
    })
}

fn incipit_body_is_empty(body: &crate::syntax::IfBody) -> bool {
    matches!(body, crate::syntax::IfBody::Block(block) if block.stmts.is_empty())
}

fn string_metadata(annotations: &[Annotation], interner: &Interner, name: &str) -> Option<String> {
    string_metadata_all(annotations, interner, name)
        .into_iter()
        .next()
}

fn string_metadata_all(annotations: &[Annotation], interner: &Interner, name: &str) -> Vec<String> {
    annotations
        .iter()
        .filter_map(|annotation| match &annotation.kind {
            AnnotationKind::Statement(stmt) if interner.resolve(stmt.name.name) == name && stmt.args.len() == 1 => {
                match &stmt.args[0].kind {
                    crate::lexer::TokenKind::String(sym) => Some(interner.resolve(*sym).to_owned()),
                    _ => None,
                }
            }
            _ => None,
        })
        .collect()
}

#[cfg(test)]
#[path = "cli_test.rs"]
mod tests;
