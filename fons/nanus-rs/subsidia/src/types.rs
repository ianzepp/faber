use std::collections::HashMap;
use std::fmt;

/// SemanticTypus represents a resolved type in the Faber type system.
/// These are distinct from the AST Typus types which are syntax nodes.
#[derive(Debug, Clone)]
pub enum SemanticTypus {
    Primitivus {
        species: PrimitivusSpecies,
        nullabilis: bool,
    },
    Lista {
        elementum: Box<SemanticTypus>,
        nullabilis: bool,
    },
    Tabula {
        clavis: Box<SemanticTypus>,
        valor: Box<SemanticTypus>,
        nullabilis: bool,
    },
    Copia {
        elementum: Box<SemanticTypus>,
        nullabilis: bool,
    },
    Functio {
        params: Vec<SemanticTypus>,
        reditus: Option<Box<SemanticTypus>>,
        nullabilis: bool,
    },
    Genus {
        nomen: String,
        agri: HashMap<String, SemanticTypus>,
        methodi: HashMap<String, Box<SemanticTypus>>,
        nullabilis: bool,
    },
    Ordo {
        nomen: String,
        membra: HashMap<String, i64>,
    },
    Discretio {
        nomen: String,
        variantes: HashMap<String, Box<SemanticTypus>>,
    },
    Pactum {
        nomen: String,
        methodi: HashMap<String, Box<SemanticTypus>>,
    },
    Usitatum {
        nomen: String,
        nullabilis: bool,
    },
    Unio {
        membra: Vec<SemanticTypus>,
        nullabilis: bool,
    },
    Parametrum {
        nomen: String,
    },
    Ignotum {
        ratio: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum PrimitivusSpecies {
    Textus,
    Numerus,
    Fractus,
    Bivalens,
    Nihil,
    Vacuum,
}

impl fmt::Display for PrimitivusSpecies {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PrimitivusSpecies::Textus => write!(f, "textus"),
            PrimitivusSpecies::Numerus => write!(f, "numerus"),
            PrimitivusSpecies::Fractus => write!(f, "fractus"),
            PrimitivusSpecies::Bivalens => write!(f, "bivalens"),
            PrimitivusSpecies::Nihil => write!(f, "nihil"),
            PrimitivusSpecies::Vacuum => write!(f, "vacuum"),
        }
    }
}

impl fmt::Display for SemanticTypus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SemanticTypus::Primitivus { species, nullabilis } => {
                write!(f, "{}", species)?;
                if *nullabilis {
                    write!(f, "?")?;
                }
                Ok(())
            }
            SemanticTypus::Lista { elementum, nullabilis } => {
                write!(f, "lista<{}>", elementum)?;
                if *nullabilis {
                    write!(f, "?")?;
                }
                Ok(())
            }
            SemanticTypus::Tabula { clavis, valor, nullabilis } => {
                write!(f, "tabula<{}, {}>", clavis, valor)?;
                if *nullabilis {
                    write!(f, "?")?;
                }
                Ok(())
            }
            SemanticTypus::Copia { elementum, nullabilis } => {
                write!(f, "copia<{}>", elementum)?;
                if *nullabilis {
                    write!(f, "?")?;
                }
                Ok(())
            }
            SemanticTypus::Functio { params, reditus, nullabilis } => {
                write!(f, "functio(")?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", p)?;
                }
                write!(f, ")")?;
                if let Some(ret) = reditus {
                    write!(f, " -> {}", ret)?;
                }
                if *nullabilis {
                    write!(f, "?")?;
                }
                Ok(())
            }
            SemanticTypus::Genus { nomen, nullabilis, .. } => {
                write!(f, "{}", nomen)?;
                if *nullabilis {
                    write!(f, "?")?;
                }
                Ok(())
            }
            SemanticTypus::Ordo { nomen, .. } => write!(f, "{}", nomen),
            SemanticTypus::Discretio { nomen, .. } => write!(f, "{}", nomen),
            SemanticTypus::Pactum { nomen, .. } => write!(f, "{}", nomen),
            SemanticTypus::Usitatum { nomen, nullabilis } => {
                write!(f, "{}", nomen)?;
                if *nullabilis {
                    write!(f, "?")?;
                }
                Ok(())
            }
            SemanticTypus::Unio { membra, nullabilis } => {
                for (i, m) in membra.iter().enumerate() {
                    if i > 0 {
                        write!(f, " | ")?;
                    }
                    write!(f, "{}", m)?;
                }
                if *nullabilis {
                    write!(f, "?")?;
                }
                Ok(())
            }
            SemanticTypus::Parametrum { nomen } => write!(f, "{}", nomen),
            SemanticTypus::Ignotum { .. } => write!(f, "ignotum"),
        }
    }
}

/// Primitive type constants.
pub fn textus() -> SemanticTypus {
    SemanticTypus::Primitivus {
        species: PrimitivusSpecies::Textus,
        nullabilis: false,
    }
}

pub fn numerus() -> SemanticTypus {
    SemanticTypus::Primitivus {
        species: PrimitivusSpecies::Numerus,
        nullabilis: false,
    }
}

pub fn fractus() -> SemanticTypus {
    SemanticTypus::Primitivus {
        species: PrimitivusSpecies::Fractus,
        nullabilis: false,
    }
}

pub fn bivalens() -> SemanticTypus {
    SemanticTypus::Primitivus {
        species: PrimitivusSpecies::Bivalens,
        nullabilis: false,
    }
}

pub fn nihil() -> SemanticTypus {
    SemanticTypus::Primitivus {
        species: PrimitivusSpecies::Nihil,
        nullabilis: false,
    }
}

pub fn vacuum() -> SemanticTypus {
    SemanticTypus::Primitivus {
        species: PrimitivusSpecies::Vacuum,
        nullabilis: false,
    }
}

pub fn ignotum() -> SemanticTypus {
    SemanticTypus::Ignotum {
        ratio: "unresolved".to_string(),
    }
}

/// Make a type nullable.
pub fn nullabilis(t: SemanticTypus) -> SemanticTypus {
    match t {
        SemanticTypus::Primitivus { species, .. } => SemanticTypus::Primitivus {
            species,
            nullabilis: true,
        },
        SemanticTypus::Lista { elementum, .. } => SemanticTypus::Lista {
            elementum,
            nullabilis: true,
        },
        SemanticTypus::Tabula { clavis, valor, .. } => SemanticTypus::Tabula {
            clavis,
            valor,
            nullabilis: true,
        },
        SemanticTypus::Copia { elementum, .. } => SemanticTypus::Copia {
            elementum,
            nullabilis: true,
        },
        SemanticTypus::Functio { params, reditus, .. } => SemanticTypus::Functio {
            params,
            reditus,
            nullabilis: true,
        },
        SemanticTypus::Genus { nomen, agri, methodi, .. } => SemanticTypus::Genus {
            nomen,
            agri,
            methodi,
            nullabilis: true,
        },
        SemanticTypus::Usitatum { nomen, .. } => SemanticTypus::Usitatum {
            nomen,
            nullabilis: true,
        },
        SemanticTypus::Unio { membra, .. } => SemanticTypus::Unio {
            membra,
            nullabilis: true,
        },
        other => other,
    }
}

impl SemanticTypus {
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            SemanticTypus::Primitivus {
                species: PrimitivusSpecies::Numerus | PrimitivusSpecies::Fractus,
                ..
            }
        )
    }

    pub fn is_fractus(&self) -> bool {
        matches!(
            self,
            SemanticTypus::Primitivus {
                species: PrimitivusSpecies::Fractus,
                ..
            }
        )
    }

    pub fn is_textus(&self) -> bool {
        matches!(
            self,
            SemanticTypus::Primitivus {
                species: PrimitivusSpecies::Textus,
                ..
            }
        )
    }

    pub fn is_nullabilis(&self) -> bool {
        match self {
            SemanticTypus::Primitivus { nullabilis, .. } => *nullabilis,
            SemanticTypus::Lista { nullabilis, .. } => *nullabilis,
            SemanticTypus::Tabula { nullabilis, .. } => *nullabilis,
            SemanticTypus::Copia { nullabilis, .. } => *nullabilis,
            SemanticTypus::Functio { nullabilis, .. } => *nullabilis,
            SemanticTypus::Genus { nullabilis, .. } => *nullabilis,
            SemanticTypus::Usitatum { nullabilis, .. } => *nullabilis,
            SemanticTypus::Unio { nullabilis, .. } => *nullabilis,
            _ => false,
        }
    }
}
