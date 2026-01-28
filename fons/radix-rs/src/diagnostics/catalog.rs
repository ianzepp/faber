//! Diagnostic codes and help text

use crate::lexer::LexErrorKind;
use crate::parser::ParseErrorKind;
use crate::semantic::{error::WarningKind, SemanticErrorKind};

#[derive(Debug, Clone, Copy)]
pub struct DiagnosticSpec {
    pub code: &'static str,
    pub help: Option<&'static str>,
}

pub fn lex_spec(kind: LexErrorKind) -> DiagnosticSpec {
    match kind {
        LexErrorKind::UnterminatedString => DiagnosticSpec {
            code: "LEX001",
            help: Some("close the string literal before the end of the line or file"),
        },
        LexErrorKind::UnterminatedComment => DiagnosticSpec {
            code: "LEX002",
            help: Some("close the block comment with '*/'"),
        },
        LexErrorKind::InvalidNumber => DiagnosticSpec {
            code: "LEX003",
            help: Some("check numeric separators and base prefixes"),
        },
        LexErrorKind::InvalidEscape => DiagnosticSpec {
            code: "LEX004",
            help: Some("use a supported escape sequence or escape the backslash"),
        },
        LexErrorKind::UnexpectedCharacter => DiagnosticSpec {
            code: "LEX005",
            help: Some("remove the character or escape it if it should be literal"),
        },
    }
}

pub fn parse_spec(kind: ParseErrorKind) -> DiagnosticSpec {
    match kind {
        ParseErrorKind::LexError => DiagnosticSpec {
            code: "PARSE000",
            help: Some("fix the lexer error first; parsing continues from clean tokens"),
        },
        ParseErrorKind::Expected => DiagnosticSpec {
            code: "PARSE001",
            help: Some("ensure the expected token is present in this position"),
        },
        ParseErrorKind::Unexpected => DiagnosticSpec {
            code: "PARSE002",
            help: Some("remove or reposition the unexpected token"),
        },
        ParseErrorKind::InvalidDeclaration => DiagnosticSpec {
            code: "PARSE010",
            help: Some("check declaration keywords and required parts"),
        },
        ParseErrorKind::MissingFunctionName => DiagnosticSpec {
            code: "PARSE011",
            help: Some("add a function name after 'functio'"),
        },
        ParseErrorKind::MissingFunctionBody => DiagnosticSpec {
            code: "PARSE012",
            help: Some("add a block or mark the function as external"),
        },
        ParseErrorKind::MissingClassName => DiagnosticSpec {
            code: "PARSE013",
            help: Some("add a class name after 'genus'"),
        },
        ParseErrorKind::MissingClassBody => DiagnosticSpec {
            code: "PARSE014",
            help: Some("add a class body block"),
        },
        ParseErrorKind::InvalidParameter => DiagnosticSpec {
            code: "PARSE015",
            help: Some("check parameter syntax and order"),
        },
        ParseErrorKind::InvalidTypeParameter => DiagnosticSpec {
            code: "PARSE016",
            help: Some("check type parameter list for commas and identifiers"),
        },
        ParseErrorKind::InvalidModifier => DiagnosticSpec {
            code: "PARSE017",
            help: Some("move or remove the modifier"),
        },
        ParseErrorKind::InvalidAnnotation => DiagnosticSpec {
            code: "PARSE018",
            help: Some("add an annotation name after '@'"),
        },
        ParseErrorKind::DuplicateModifier => DiagnosticSpec {
            code: "PARSE019",
            help: Some("remove the duplicate modifier"),
        },
        ParseErrorKind::InvalidStatement => DiagnosticSpec {
            code: "PARSE020",
            help: Some("check the statement form and required keywords"),
        },
        ParseErrorKind::MissingCondition => DiagnosticSpec {
            code: "PARSE021",
            help: Some("add a condition expression"),
        },
        ParseErrorKind::MissingBlock => DiagnosticSpec {
            code: "PARSE022",
            help: Some("add a block or use 'ergo' form"),
        },
        ParseErrorKind::InvalidPattern => DiagnosticSpec {
            code: "PARSE023",
            help: Some("use '_' or an identifier pattern"),
        },
        ParseErrorKind::InvalidMatchArm => DiagnosticSpec {
            code: "PARSE024",
            help: Some("use 'casu <pattern>' or 'ceterum'"),
        },
        ParseErrorKind::InvalidCaseValue => DiagnosticSpec {
            code: "PARSE025",
            help: Some("use a valid case expression"),
        },
        ParseErrorKind::InvalidExpression => DiagnosticSpec {
            code: "PARSE030",
            help: Some("check expression syntax"),
        },
        ParseErrorKind::InvalidLiteral => DiagnosticSpec {
            code: "PARSE031",
            help: Some("check the literal form"),
        },
        ParseErrorKind::InvalidOperator => DiagnosticSpec {
            code: "PARSE032",
            help: Some("check operator placement and operands"),
        },
        ParseErrorKind::UnterminatedGroup => DiagnosticSpec {
            code: "PARSE033",
            help: Some("close the grouping delimiter"),
        },
        ParseErrorKind::InvalidCallArgument => DiagnosticSpec {
            code: "PARSE034",
            help: Some("check call argument syntax"),
        },
        ParseErrorKind::InvalidMemberAccess => DiagnosticSpec {
            code: "PARSE035",
            help: Some("check member access syntax"),
        },
        ParseErrorKind::InvalidAssignmentTarget => DiagnosticSpec {
            code: "PARSE036",
            help: Some("assign only to valid lvalues"),
        },
        ParseErrorKind::InvalidType => DiagnosticSpec {
            code: "PARSE040",
            help: Some("check type syntax"),
        },
        ParseErrorKind::InvalidTypeAnnotation => DiagnosticSpec {
            code: "PARSE041",
            help: Some("check type annotation form"),
        },
        ParseErrorKind::UnterminatedTypeParams => DiagnosticSpec {
            code: "PARSE042",
            help: Some("close the type parameter list with '>'"),
        },
        ParseErrorKind::InvalidImport => DiagnosticSpec {
            code: "PARSE050",
            help: Some("check import syntax"),
        },
        ParseErrorKind::InvalidDirective => DiagnosticSpec {
            code: "PARSE051",
            help: Some("directives must appear at file scope"),
        },
    }
}

pub fn semantic_spec(kind: SemanticErrorKind) -> DiagnosticSpec {
    match kind {
        SemanticErrorKind::UndefinedVariable => DiagnosticSpec {
            code: "SEM001",
            help: Some("declare the variable before use"),
        },
        SemanticErrorKind::UndefinedType => DiagnosticSpec {
            code: "SEM002",
            help: Some("declare the type or import it"),
        },
        SemanticErrorKind::UndefinedFunction => DiagnosticSpec {
            code: "SEM003",
            help: Some("declare the function or import it"),
        },
        SemanticErrorKind::UndefinedMember => DiagnosticSpec {
            code: "SEM004",
            help: Some("check the member name and type"),
        },
        SemanticErrorKind::DuplicateDefinition => DiagnosticSpec {
            code: "SEM005",
            help: Some("rename one of the definitions"),
        },
        SemanticErrorKind::ImportNotFound => DiagnosticSpec {
            code: "SEM006",
            help: Some("check the import path"),
        },
        SemanticErrorKind::CircularDependency => DiagnosticSpec {
            code: "SEM007",
            help: Some("break the cycle between modules"),
        },
        SemanticErrorKind::TypeMismatch => DiagnosticSpec {
            code: "SEM010",
            help: Some("make the expression type match the expected type"),
        },
        SemanticErrorKind::InvalidOperandTypes => DiagnosticSpec {
            code: "SEM011",
            help: Some("use compatible operand types"),
        },
        SemanticErrorKind::NotCallable => DiagnosticSpec {
            code: "SEM012",
            help: Some("call a function or closure"),
        },
        SemanticErrorKind::WrongArity => DiagnosticSpec {
            code: "SEM013",
            help: Some("adjust the number of arguments"),
        },
        SemanticErrorKind::MissingTypeAnnotation => DiagnosticSpec {
            code: "SEM014",
            help: Some("add a type annotation"),
        },
        SemanticErrorKind::InvalidCast => DiagnosticSpec {
            code: "SEM015",
            help: Some("cast to a compatible type"),
        },
        SemanticErrorKind::InvalidConversion => DiagnosticSpec {
            code: "SEM016",
            help: Some("use a valid conversion or provide a fallback"),
        },
        SemanticErrorKind::ImmutableAssignment => DiagnosticSpec {
            code: "SEM020",
            help: Some("make the binding mutable or assign to a mutable target"),
        },
        SemanticErrorKind::InvalidAssignmentTarget => DiagnosticSpec {
            code: "SEM021",
            help: Some("assign to a valid lvalue"),
        },
        SemanticErrorKind::BreakOutsideLoop => DiagnosticSpec {
            code: "SEM030",
            help: Some("use 'rumpe' inside a loop"),
        },
        SemanticErrorKind::ContinueOutsideLoop => DiagnosticSpec {
            code: "SEM031",
            help: Some("use 'perge' inside a loop"),
        },
        SemanticErrorKind::ReturnOutsideFunction => DiagnosticSpec {
            code: "SEM032",
            help: Some("use 'redde' inside a function"),
        },
        SemanticErrorKind::MissingReturn => DiagnosticSpec {
            code: "SEM033",
            help: Some("add a return statement or make the return type 'vacuum'"),
        },
        SemanticErrorKind::NonExhaustiveMatch => DiagnosticSpec {
            code: "SEM040",
            help: Some("add missing cases or a 'ceterum' arm"),
        },
        SemanticErrorKind::UnreachablePattern => DiagnosticSpec {
            code: "SEM041",
            help: Some("remove or reorder the unreachable pattern"),
        },
        SemanticErrorKind::DuplicatePattern => DiagnosticSpec {
            code: "SEM042",
            help: Some("remove the duplicate pattern"),
        },
        SemanticErrorKind::UseAfterMove => DiagnosticSpec {
            code: "SEM050",
            help: Some("avoid using a value after it has been moved"),
        },
        SemanticErrorKind::BorrowOfMoved => DiagnosticSpec {
            code: "SEM051",
            help: Some("borrow only from values that have not been moved"),
        },
        SemanticErrorKind::MutableBorrowConflict => DiagnosticSpec {
            code: "SEM052",
            help: Some("avoid overlapping mutable borrows"),
        },
        SemanticErrorKind::CannotMoveOut => DiagnosticSpec {
            code: "SEM053",
            help: Some("use a reference or clone instead of moving"),
        },
        SemanticErrorKind::LifetimeMismatch => DiagnosticSpec {
            code: "SEM054",
            help: Some("ensure borrowed values live long enough"),
        },
        SemanticErrorKind::Warning(kind) => warning_spec(kind),
    }
}

fn warning_spec(kind: WarningKind) -> DiagnosticSpec {
    match kind {
        WarningKind::UnusedVariable => DiagnosticSpec {
            code: "WARN001",
            help: Some("remove the variable or prefix it with '_'"),
        },
        WarningKind::UnusedImport => DiagnosticSpec {
            code: "WARN002",
            help: Some("remove the unused import"),
        },
        WarningKind::UnusedFunction => DiagnosticSpec {
            code: "WARN003",
            help: Some("remove the function or mark it as used"),
        },
        WarningKind::UnreachableCode => DiagnosticSpec {
            code: "WARN004",
            help: Some("remove unreachable code"),
        },
        WarningKind::UnnecessaryCast => DiagnosticSpec {
            code: "WARN005",
            help: Some("remove the redundant cast"),
        },
        WarningKind::DeprecatedFeature => DiagnosticSpec {
            code: "WARN006",
            help: Some("replace the deprecated feature"),
        },
        WarningKind::ShadowedVariable => DiagnosticSpec {
            code: "WARN007",
            help: Some("rename one of the variables"),
        },
        WarningKind::TargetNoop => DiagnosticSpec {
            code: "WARN008",
            help: Some("remove the construct or gate it by target"),
        },
    }
}
