/**
 * Generated ZIG norma registry.
 * Source: fons/norma/
 * Generator: bun run build:norma
 * Generated: 2026-01-15T14:33:44.456Z
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
  "json": {
    "methods": {
      "solve": {
        "template": "std.json.stringifyAlloc(§1, §0, .{})",
        "params": ["val","alloc"]
      },
      "solvePulchre": {
        "template": "std.json.stringifyAlloc(§2, §0, .{ .whitespace = .{ .indent_level = §1 } })",
        "params": ["val","indentum","alloc"]
      },
      "pange": {
        "template": "std.json.parseFromSlice(std.json.Value, std.heap.page_allocator, §, .{})",
        "params": ["textus"]
      },
      "estNihil": {
        "template": "(§ == .null)",
        "params": ["val"]
      },
      "estBivalens": {
        "template": "(§ == .true or § == .false)",
        "params": ["val"]
      },
      "estNumerus": {
        "template": "(§ == .integer or § == .float)",
        "params": ["val"]
      },
      "estTextus": {
        "template": "(§ == .string)",
        "params": ["val"]
      },
      "estLista": {
        "template": "(§ == .array)",
        "params": ["val"]
      },
      "estTabula": {
        "template": "(§ == .object)",
        "params": ["val"]
      },
      "utTextus": {
        "template": "(if (§ == .string) §.string else \\\"\\\")",
        "params": ["val"]
      },
      "utNumerus": {
        "template": "(if (§ == .integer) §.integer else 0)",
        "params": ["val"]
      },
      "utBivalens": {
        "template": "(if (§ == .true) true else false)",
        "params": ["val"]
      },
      "cape": {
        "template": "(if (§.object.get(§)) |v| v else .null)",
        "params": ["val","key"]
      },
      "capeIndice": {
        "template": "(if (§ < §.array.len) §.array[§] else .null)",
        "params": ["val","idx"]
      }
    }
    , "innatum": "std.json"
  },
  "tempus": {
    "methods": {
      "nunc": {
        "template": "std.time.milliTimestamp()",
        "params": []
      },
      "nunc_nano": {
        "template": "std.time.nanoTimestamp()",
        "params": []
      },
      "nunc_secunda": {
        "template": "std.time.timestamp()",
        "params": []
      },
      "dormi": {
        "template": "std.time.sleep(§ * 1_000_000)",
        "params": ["ms"]
      },
      "MILLISECUNDUM": {
        "template": "1",
        "params": []
      },
      "SECUNDUM": {
        "template": "1000",
        "params": []
      },
      "MINUTUM": {
        "template": "60000",
        "params": []
      },
      "HORA": {
        "template": "3600000",
        "params": []
      },
      "DIES": {
        "template": "86400000",
        "params": []
      }
    }
    , "innatum": "std.time"
  },
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
  "aleator": {
    "methods": {
      "fractus": {
        "template": "blk: { var seed: u64 = undefined; std.crypto.random.bytes(std.mem.asBytes(&seed)); var prng = std.rand.DefaultPrng.init(seed); break :blk prng.random().float(f64); }",
        "params": []
      },
      "inter": {
        "template": "blk: { var seed: u64 = undefined; std.crypto.random.bytes(std.mem.asBytes(&seed)); var prng = std.rand.DefaultPrng.init(seed); break :blk prng.random().intRangeAtMost(i64, §0, §1); }",
        "params": ["min","max"]
      },
      "octeti": {
        "template": "blk: { var buf = §1.alloc(u8, §0) catch unreachable; std.crypto.random.bytes(buf); break :blk buf; }",
        "params": ["n","alloc"]
      },
      "uuid": {
        "template": "blk: { var buf: [36]u8 = undefined; var bytes: [16]u8 = undefined; std.crypto.random.bytes(&bytes); bytes[6] = (bytes[6] & 0x0f) | 0x40; bytes[8] = (bytes[8] & 0x3f) | 0x80; const hex = \\\"0123456789abcdef\\\"; var i: usize = 0; for (bytes, 0..) |b, j| { if (j == 4 or j == 6 or j == 8 or j == 10) { buf[i] = '-'; i += 1; } buf[i] = hex[b >> 4]; buf[i + 1] = hex[b & 0x0f]; i += 2; } break :blk buf; }",
        "params": []
      },
      "selige": {
        "template": "blk: { var seed: u64 = undefined; std.crypto.random.bytes(std.mem.asBytes(&seed)); var prng = std.rand.DefaultPrng.init(seed); break :blk §0[prng.random().uintLessThan(usize, §0.len)]; }",
        "params": ["lista"]
      },
      "miscita": {
        "template": "blk: { var seed: u64 = undefined; std.crypto.random.bytes(std.mem.asBytes(&seed)); var prng = std.rand.DefaultPrng.init(seed); var copy = §1.dupe(@TypeOf(§0[0]), §0) catch unreachable; prng.random().shuffle(@TypeOf(copy[0]), copy); break :blk copy; }",
        "params": ["lista","alloc"]
      },
      "semen": {
        "template": "@compileLog(\\\"seed(§) - Zig requires explicit PRNG state management\\\")",
        "params": ["n"]
      }
    }
    , "innatum": "std.rand"
  },
  "solum": {
    "methods": {
      "legens": {
        "template": "solum.legens(§)",
        "params": ["path"]
      },
      "leget": {
        "template": "solum.leget(§1, §0)",
        "params": ["path","alloc"]
      },
      "lege": {
        "template": "solum.lege(§1, §0)",
        "params": ["path","alloc"]
      },
      "ausculta": {
        "template": "solum.ausculta()",
        "params": []
      },
      "hauri": {
        "template": "solum.hauri(§)",
        "params": ["alloc"]
      },
      "scribens": {
        "template": "solum.scribens(§)",
        "params": ["path"]
      },
      "scribet": {
        "template": "solum.scribet(§, §)",
        "params": ["path","data"]
      },
      "inscribe": {
        "template": "solum.inscribe(§, §)",
        "params": ["path","data"]
      },
      "apponet": {
        "template": "solum.apponet(§, §)",
        "params": ["path","data"]
      },
      "appone": {
        "template": "solum.appone(§, §)",
        "params": ["path","data"]
      },
      "exstat": {
        "template": "solum.exstat(§)",
        "params": ["path"]
      },
      "inspice": {
        "template": "solum.inspice(§)",
        "params": ["path"]
      },
      "dele": {
        "template": "solum.dele(§)",
        "params": ["path"]
      },
      "duplica": {
        "template": "solum.duplica(§, §)",
        "params": ["src","dest"]
      },
      "move": {
        "template": "solum.move(§, §)",
        "params": ["src","dest"]
      },
      "trunca": {
        "template": "solum.trunca(§, §)",
        "params": ["path","size"]
      },
      "tange": {
        "template": "solum.tange(§)",
        "params": ["path"]
      },
      "crea": {
        "template": "solum.crea(§)",
        "params": ["path"]
      },
      "elenca": {
        "template": "solum.elenca(§1, §0)",
        "params": ["path","alloc"]
      },
      "ambula": {
        "template": "solum.ambula(§)",
        "params": ["path"]
      },
      "vacua": {
        "template": "solum.vacua(§)",
        "params": ["path"]
      },
      "deleArborem": {
        "template": "solum.deleArborem(§)",
        "params": ["path"]
      },
      "iunge": {
        "template": "solum.iunge(§)",
        "params": ["parts"]
      },
      "dir": {
        "template": "solum.dir(§)",
        "params": ["path"]
      },
      "basis": {
        "template": "solum.basis(§)",
        "params": ["path"]
      },
      "extensio": {
        "template": "solum.extensio(§)",
        "params": ["path"]
      },
      "resolve": {
        "template": "solum.resolve(§1, §0)",
        "params": ["path","alloc"]
      },
      "domus": {
        "template": "solum.domus()",
        "params": []
      }
    }
    , "innatum": "solum"
  },
  "caelum": {
    "methods": {
      "pete": {
        "template": "caelum.pete(§)",
        "params": ["url"]
      },
      "mitte": {
        "template": "caelum.mitte(§, §)",
        "params": ["url","corpus"]
      },
      "pone": {
        "template": "caelum.pone(§, §)",
        "params": ["url","corpus"]
      },
      "dele": {
        "template": "caelum.dele(§)",
        "params": ["url"]
      },
      "muta": {
        "template": "caelum.muta(§, §)",
        "params": ["url","corpus"]
      },
      "roga": {
        "template": "caelum.roga(§, §, §, §)",
        "params": ["modus","url","capita","corpus"]
      },
      "exspecta": {
        "template": "caelum.exspecta(§, §)",
        "params": ["handler","portus"]
      },
      "siste": {
        "template": "§.siste()",
        "params": ["s"]
      },
      "replicatio": {
        "template": "caelum.replicatio(§, §, §)",
        "params": ["status","capita","corpus"]
      }
    }
    , "innatum": "caelum"
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
      "adde": {
        "template": "§0.adde(§2, §1)",
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
};

export const radixForms: Record<string, Record<string, string[]>> = {
  "aleator": {
    "miscita": ["misc","perfectum"]
  },
  "solum": {
    "legens": ["leg","participium_praesens"],
    "leget": ["leg","futurum_indicativum"],
    "lege": ["leg","imperativus"],
    "scribens": ["scrib","participium_praesens"],
    "scribet": ["scrib","futurum_indicativum"],
    "inscribe": ["inscrib","imperativus"],
    "apponet": ["appon","futurum_indicativum"],
    "appone": ["appon","imperativus"],
    "dele": ["del","futurum_indicativum","imperativus"],
    "duplica": ["duplic","futurum_indicativum","imperativus"],
    "move": ["mov","futurum_indicativum","imperativus"],
    "crea": ["cre","futurum_indicativum","imperativus"],
    "elenca": ["elenc","participium_praesens","futurum_indicativum","imperativus"],
    "ambula": ["ambul","participium_praesens"],
    "vacua": ["vacu","futurum_indicativum","imperativus"]
  },
  "caelum": {
    "pete": ["pet","imperativus"],
    "mitte": ["mitt","imperativus"],
    "pone": ["pon","imperativus"],
    "dele": ["del","imperativus"],
    "muta": ["mut","imperativus"],
    "roga": ["rog","imperativus"]
  },
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
    "adde": ["add","imperativus","perfectum"],
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
};
