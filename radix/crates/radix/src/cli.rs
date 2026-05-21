//! Target-independent CLI IR construction and validation.

use crate::lexer::{Interner, Span};
use crate::semantic::{SemanticError, SemanticErrorKind};
use crate::syntax::{
    Annotation, AnnotationKind, Expr, ExprKind, FuncDecl, Ident, Literal, OperandusAnnotation, OptioAnnotation,
    Program, Stmt, StmtKind, TypeExpr, TypeExprKind,
};
use rustc_hash::{FxHashMap, FxHashSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliMode {
    NotCli,
    SingleCommand,
    Subcommand,
}

#[derive(Debug, Clone)]
pub struct CliAnalysis {
    pub mode: CliMode,
    pub program: Option<CliProgram>,
    pub errors: Vec<SemanticError>,
}

#[derive(Debug, Clone)]
pub struct CliProgram {
    pub name: String,
    pub entry_args: String,
    pub mode: CliMode,
    pub version: Option<String>,
    pub description: Option<String>,
    pub global_options: Vec<CliOption>,
    pub global_operands: Vec<CliOperand>,
    pub options: Vec<CliOption>,
    pub operands: Vec<CliOperand>,
    pub commands: Vec<CliCommand>,
    pub exit: Option<CliExit>,
}

#[derive(Debug, Clone)]
pub enum CliExit {
    Fixed(i64),
    Binding(String),
    Field { object: String, field: String },
    Unsupported,
}

#[derive(Debug, Clone)]
pub struct CliCommand {
    pub path: Vec<String>,
    pub module_path: Option<Vec<String>>,
    pub function: String,
    pub function_symbol: crate::lexer::Symbol,
    pub args_binding: Option<String>,
    pub aliases: Vec<String>,
    pub description: Option<String>,
    pub options: Vec<CliOption>,
    pub operands: Vec<CliOperand>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct CliOption {
    pub binding: String,
    pub binding_symbol: crate::lexer::Symbol,
    pub ty: CliType,
    pub short: Option<String>,
    pub long: Option<String>,
    pub description: Option<String>,
    pub global: bool,
    pub default: Option<CliDefault>,
    pub flag: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct CliOperand {
    pub binding: String,
    pub binding_symbol: crate::lexer::Symbol,
    pub ty: CliType,
    pub rest: bool,
    pub description: Option<String>,
    pub global: bool,
    pub default: Option<CliDefault>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliType {
    Textus,
    Numerus,
    Fractus,
    Bivalens,
    Octeti,
    Ignotum,
    ListaTextus,
    ListaNumerus,
}

#[derive(Debug, Clone)]
pub enum CliDefault {
    Text(String),
    Integer(i64),
    Float(f64),
    Bool(bool),
    Nil,
    Expr(String),
}

pub fn analyze(program: &Program, interner: &Interner) -> CliAnalysis {
    let mut builder = CliBuilder { interner, errors: Vec::new() };
    builder.analyze(program)
}

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
            ExprKind::Literal(Literal::String(sym)) | ExprKind::Literal(Literal::TemplateString(sym)) => {
                CliDefault::Text(self.resolve(*sym))
            }
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
