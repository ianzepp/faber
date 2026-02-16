/**
 * Generated ZIG norma registry.
 * Source: fons/norma/
 * Generator: bun run build:norma
 */

export interface Translation {
    method?: string;
    template?: string;
    params?: string[];
}

export interface NormaCollection {
    innatum?: string;
    methods: Record<string, Translation>;
}

export const norma: Record<string, NormaCollection> = {
  "mathesis": {
    "methods": {
      "pavimentum": {
        "template": "@floor(§)",
        "params": ["x"]
      },
      "tectum": {
        "template": "@ceil(§)",
        "params": ["x"]
      },
      "rotundum": {
        "template": "@round(§)",
        "params": ["x"]
      },
      "truncatum": {
        "template": "@trunc(§)",
        "params": ["x"]
      },
      "radix": {
        "template": "@sqrt(§)",
        "params": ["x"]
      },
      "potentia": {
        "template": "std.math.pow(§, §)",
        "params": ["base","exp"]
      },
      "logarithmus": {
        "template": "@log(§)",
        "params": ["x"]
      },
      "logarithmus10": {
        "template": "std.math.log10(§)",
        "params": ["x"]
      },
      "exponens": {
        "template": "@exp(§)",
        "params": ["x"]
      },
      "sinus": {
        "template": "@sin(§)",
        "params": ["x"]
      },
      "cosinus": {
        "template": "@cos(§)",
        "params": ["x"]
      },
      "tangens": {
        "template": "@tan(§)",
        "params": ["x"]
      },
      "absolutum": {
        "template": "@abs(§)",
        "params": ["x"]
      },
      "signum": {
        "template": "std.math.sign(§)",
        "params": ["x"]
      },
      "minimus": {
        "template": "@min(§, §)",
        "params": ["a","b"]
      },
      "maximus": {
        "template": "@max(§, §)",
        "params": ["a","b"]
      },
      "constringens": {
        "template": "std.math.clamp(§, §, §)",
        "params": ["x","lo","hi"]
      },
      "PI": {
        "template": "std.math.pi",
        "params": []
      },
      "E": {
        "template": "std.math.e",
        "params": []
      },
      "TAU": {
        "template": "(std.math.pi * 2)",
        "params": []
      }
    }
    , "innatum": "std.math"
  },
  "tabula": {
    "methods": {
      "pone": {
        "template": "§0.pone(§3, §1, §2)",
        "params": ["ego","k","v","alloc"]
      },
      "accipe": {
        "template": "§0.accipe(§1)",
        "params": ["ego","k"]
      },
      "accipeAut": {
        "template": "§0.accipeAut(§1, §2)",
        "params": ["ego","k","def"]
      },
      "habet": {
        "template": "§0.habet(§1)",
        "params": ["ego","k"]
      },
      "dele": {
        "template": "_ = §0.dele(§1)",
        "params": ["ego","k"]
      },
      "longitudo": {
        "template": "§.longitudo()",
        "params": ["ego"]
      },
      "vacua": {
        "template": "§.vacua()",
        "params": ["ego"]
      },
      "purga": {
        "template": "§.purga()",
        "params": ["ego"]
      },
      "claves": {
        "template": "§.claves()",
        "params": ["ego"]
      },
      "valores": {
        "template": "§.valores()",
        "params": ["ego"]
      },
      "paria": {
        "template": "§.paria()",
        "params": ["ego"]
      },
      "selecta": {
        "template": "@compileError(\\\"selecta not implemented for Zig - use explicit loop\\\")",
        "params": ["ego","claves"]
      },
      "omissa": {
        "template": "@compileError(\\\"omissa not implemented for Zig - use explicit loop\\\")",
        "params": ["ego","claves"]
      },
      "conflata": {
        "template": "§0.conflata(&§1)",
        "params": ["ego","alia"]
      },
      "inversa": {
        "template": "@compileError(\\\"inversa not implemented for Zig - use explicit loop\\\")",
        "params": ["ego"]
      },
      "mappaValores": {
        "template": "@compileError(\\\"mappaValores not implemented for Zig - use explicit loop\\\")",
        "params": ["ego","fn"]
      },
      "mappaClaves": {
        "template": "@compileError(\\\"mappaClaves not implemented for Zig - use explicit loop\\\")",
        "params": ["ego","fn"]
      },
      "inLista": {
        "template": "§.inLista(§)",
        "params": ["ego","alloc"]
      },
      "inObjectum": {
        "template": "@compileError(\\\"inObjectum not implemented for Zig - Zig has no object type\\\")",
        "params": ["ego"]
      }
    }
    , "innatum": "Tabula"
  },
  "numerus": {
    "methods": {
      "absolutum": {
        "template": "@intCast(@abs(§))",
        "params": ["ego"]
      },
      "signum": {
        "template": "std.math.sign(§)",
        "params": ["ego"]
      },
      "minimus": {
        "template": "@min(§, §)",
        "params": ["ego","other"]
      },
      "maximus": {
        "template": "@max(§, §)",
        "params": ["ego","other"]
      }
    }
    , "innatum": "i64"
  },
  "fractus": {
    "methods": {
      "absolutum": {
        "template": "@abs(§)",
        "params": ["ego"]
      },
      "signum": {
        "template": "std.math.sign(§)",
        "params": ["ego"]
      },
      "minimus": {
        "template": "@min(§, §)",
        "params": ["ego","other"]
      },
      "maximus": {
        "template": "@max(§, §)",
        "params": ["ego","other"]
      },
      "rotunda": {
        "template": "@intFromFloat(@round(§))",
        "params": ["ego"]
      },
      "pavimentum": {
        "template": "@intFromFloat(@floor(§))",
        "params": ["ego"]
      },
      "tectum": {
        "template": "@intFromFloat(@ceil(§))",
        "params": ["ego"]
      },
      "trunca": {
        "template": "@intFromFloat(@trunc(§))",
        "params": ["ego"]
      }
    }
    , "innatum": "f64"
  },
  "textus": {
    "methods": {
      "longitudo": {
        "template": "§.len",
        "params": ["ego"]
      },
      "sectio": {
        "template": "§[§..§]",
        "params": ["ego","start","end"]
      },
      "continet": {
        "template": "(std.mem.indexOf(u8, §, §) != null)",
        "params": ["ego","sub"]
      },
      "initium": {
        "template": "std.mem.startsWith(u8, §, §)",
        "params": ["ego","prefix"]
      },
      "finis": {
        "template": "std.mem.endsWith(u8, §, §)",
        "params": ["ego","suffix"]
      },
      "maiuscula": {
        "template": "std.ascii.upperString(§)",
        "params": ["ego"]
      },
      "minuscula": {
        "template": "std.ascii.lowerString(§)",
        "params": ["ego"]
      },
      "recide": {
        "template": "std.mem.trim(u8, §, \\\" \\\\t\\\\n\\\\r\\\")",
        "params": ["ego"]
      },
      "divide": {
        "template": "@compileError(\\\"Use std.mem.splitSequence for Zig\\\")",
        "params": ["ego","sep"]
      },
      "muta": {
        "template": "@compileError(\\\"Use std.mem.replace for Zig\\\")",
        "params": ["ego","old","new"]
      }
    }
    , "innatum": "[]const u8"
  },
  "copia": {
    "methods": {
      "adde": {
        "template": "§0.adde(§2, §1)",
        "params": ["ego","elem","alloc"]
      },
      "habet": {
        "template": "§0.habet(§1)",
        "params": ["ego","elem"]
      },
      "dele": {
        "template": "_ = §0.dele(§1)",
        "params": ["ego","elem"]
      },
      "longitudo": {
        "template": "§.longitudo()",
        "params": ["ego"]
      },
      "vacua": {
        "template": "§.vacua()",
        "params": ["ego"]
      },
      "purga": {
        "template": "§.purga()",
        "params": ["ego"]
      },
      "valores": {
        "template": "§.valores()",
        "params": ["ego"]
      },
      "perambula": {
        "template": "@compileError(\\\"perambula not implemented for Zig - use 'ex set.valores() pro item { ... }' loop\\\")",
        "params": ["ego","fn"]
      },
      "unio": {
        "template": "@compileError(\\\"unio not implemented for Zig - use explicit loop to merge sets\\\")",
        "params": ["ego","alia"]
      },
      "intersectio": {
        "template": "@compileError(\\\"intersectio not implemented for Zig - use explicit loop\\\")",
        "params": ["ego","alia"]
      },
      "differentia": {
        "template": "@compileError(\\\"differentia not implemented for Zig - use explicit loop\\\")",
        "params": ["ego","alia"]
      },
      "symmetrica": {
        "template": "@compileError(\\\"symmetrica not implemented for Zig - use explicit loop\\\")",
        "params": ["ego","alia"]
      },
      "subcopia": {
        "template": "@compileError(\\\"subcopia not implemented for Zig - use explicit loop\\\")",
        "params": ["ego","alia"]
      },
      "supercopia": {
        "template": "@compileError(\\\"supercopia not implemented for Zig - use explicit loop\\\")",
        "params": ["ego","alia"]
      },
      "inLista": {
        "template": "@compileError(\\\"inLista not implemented for Zig - iterate with ex...pro into ArrayList\\\")",
        "params": ["ego"]
      }
    }
    , "innatum": "Copia"
  },
  "lista": {
    "methods": {
      "appende": {
        "template": "§0.appende(§2, §1)",
        "params": ["ego","elem","alloc"]
      },
      "addita": {
        "template": "§0.addita(§2, §1)",
        "params": ["ego","elem","alloc"]
      },
      "praepone": {
        "template": "§0.praepone(§2, §1)",
        "params": ["ego","elem","alloc"]
      },
      "praeposita": {
        "template": "§0.praeposita(§2, §1)",
        "params": ["ego","elem","alloc"]
      },
      "remove": {
        "template": "§.remove()",
        "params": ["ego"]
      },
      "remota": {
        "template": "§.remota(§)",
        "params": ["ego","alloc"]
      },
      "decapita": {
        "template": "§.decapita()",
        "params": ["ego"]
      },
      "decapitata": {
        "template": "§.decapitata(§)",
        "params": ["ego","alloc"]
      },
      "purga": {
        "template": "§.purga()",
        "params": ["ego"]
      },
      "primus": {
        "template": "§.primus()",
        "params": ["ego"]
      },
      "ultimus": {
        "template": "§.ultimus()",
        "params": ["ego"]
      },
      "accipe": {
        "template": "§.accipe(§)",
        "params": ["ego","idx"]
      },
      "longitudo": {
        "template": "§.longitudo()",
        "params": ["ego"]
      },
      "vacua": {
        "template": "§.vacua()",
        "params": ["ego"]
      },
      "continet": {
        "template": "§.continet(§)",
        "params": ["ego","elem"]
      },
      "indiceDe": {
        "template": "§.indiceDe(§)",
        "params": ["ego","elem"]
      },
      "inveni": {
        "template": "§.inveni(§)",
        "params": ["ego","pred"]
      },
      "inveniIndicem": {
        "template": "§.inveniIndicem(§)",
        "params": ["ego","pred"]
      },
      "omnes": {
        "template": "§.omnes(§)",
        "params": ["ego","pred"]
      },
      "aliquis": {
        "template": "§.aliquis(§)",
        "params": ["ego","pred"]
      },
      "filtrata": {
        "template": "§0.filtrata(§2, §1)",
        "params": ["ego","pred","alloc"]
      },
      "mappata": {
        "template": "§0.mappata(§2, §1)",
        "params": ["ego","fn","alloc"]
      },
      "inversa": {
        "template": "§.inversa(§)",
        "params": ["ego","alloc"]
      },
      "ordinata": {
        "template": "§.ordinata(§)",
        "params": ["ego","alloc"]
      },
      "sectio": {
        "template": "§0.sectio(§3, §1, §2)",
        "params": ["ego","start","end","alloc"]
      },
      "prima": {
        "template": "§0.prima(§2, §1)",
        "params": ["ego","n","alloc"]
      },
      "ultima": {
        "template": "§0.ultima(§2, §1)",
        "params": ["ego","n","alloc"]
      },
      "omissa": {
        "template": "§0.omissa(§2, §1)",
        "params": ["ego","n","alloc"]
      },
      "reducta": {
        "template": "§.reducta(§, §)",
        "params": ["ego","fn","init"]
      },
      "ordina": {
        "template": "§.ordina()",
        "params": ["ego"]
      },
      "inverte": {
        "template": "§.inverte()",
        "params": ["ego"]
      },
      "perambula": {
        "template": "§.perambula(§)",
        "params": ["ego","fn"]
      },
      "summa": {
        "template": "§.summa()",
        "params": ["ego"]
      },
      "medium": {
        "template": "§.medium()",
        "params": ["ego"]
      },
      "minimus": {
        "template": "§.minimus()",
        "params": ["ego"]
      },
      "maximus": {
        "template": "§.maximus()",
        "params": ["ego"]
      },
      "numera": {
        "template": "§.numera(§)",
        "params": ["ego","pred"]
      },
    }
    , "innatum": "Lista"
  }
}

export const radixForms: Record<string, Record<string, string[]>> = {
  "tabula": {
    "pone": ["pon","imperativus"],
    "dele": ["del","imperativus"],
    "purga": ["purg","imperativus"],
    "selecta": ["select","perfectum"],
    "omissa": ["omis","perfectum"],
    "conflata": ["confl","perfectum"],
    "inversa": ["inver","perfectum"]
  },
  "copia": {
    "adde": ["add","imperativus"],
    "dele": ["del","imperativus"],
    "purga": ["purg","imperativus"]
  },
  "lista": {
    "appende": ["append","imperativus","perfectum"],
    "praepone": ["praepon","imperativus","perfectum"],
    "remove": ["remov","imperativus","perfectum"],
    "decapita": ["decapit","imperativus","perfectum"],
    "purga": ["purg","imperativus"],
    "filtrata": ["filtr","imperativus","perfectum"],
    "mappata": ["mapp","perfectum"],
    "explanata": ["explan","perfectum"],
    "inversa": ["inver","perfectum"],
    "ordinata": ["ordin","imperativus","perfectum"],
    "omissa": ["omis","perfectum"],
    "reducta": ["reduc","perfectum"],
    "filtra": ["filtr","imperativus","perfectum"],
    "ordina": ["ordin","imperativus","perfectum"],
    "inverte": ["invert","imperativus"],
    "miscita": ["misc","perfectum"]
  }
}
