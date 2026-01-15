/**
 * Generated PY norma registry.
 * Source: fons/norma/
 * Generator: bun run build:norma
 * Generated: 2026-01-15T14:33:44.454Z
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
        "template": "json.dumps(§)",
        "params": ["val"]
      },
      "solvePulchre": {
        "template": "json.dumps(§, indent=§)",
        "params": ["val","indentum"]
      },
      "pange": {
        "template": "json.loads(§)",
        "params": ["textus"]
      },
      "estNihil": {
        "template": "(§ is None)",
        "params": ["val"]
      },
      "estBivalens": {
        "template": "isinstance(§, bool)",
        "params": ["val"]
      },
      "estNumerus": {
        "template": "isinstance(§, (int, float))",
        "params": ["val"]
      },
      "estTextus": {
        "template": "isinstance(§, str)",
        "params": ["val"]
      },
      "estLista": {
        "template": "isinstance(§, list)",
        "params": ["val"]
      },
      "estTabula": {
        "template": "isinstance(§, dict)",
        "params": ["val"]
      },
      "utTextus": {
        "template": "(§ if isinstance(§, str) else '')",
        "params": ["val"]
      },
      "utNumerus": {
        "template": "(§ if isinstance(§, (int, float)) else 0)",
        "params": ["val"]
      },
      "utBivalens": {
        "template": "(§ if isinstance(§, bool) else False)",
        "params": ["val"]
      },
      "cape": {
        "template": "§.get(§)",
        "params": ["val","key"]
      },
      "capeIndice": {
        "template": "(§[§] if § < len(§) else None)",
        "params": ["val","idx"]
      }
    }
    , "innatum": "json"
  },
  "tempus": {
    "methods": {
      "nunc": {
        "template": "int(time.time() * 1000)",
        "params": []
      },
      "nunc_nano": {
        "template": "time.time_ns()",
        "params": []
      },
      "nunc_secunda": {
        "template": "int(time.time())",
        "params": []
      },
      "dormi": {
        "template": "time.sleep(§ / 1000)",
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
    , "innatum": "time"
  },
  "mathesis": {
    "methods": {
      "pavimentum": {
        "template": "math.floor(§)",
        "params": ["x"]
      },
      "tectum": {
        "template": "math.ceil(§)",
        "params": ["x"]
      },
      "rotundum": {
        "template": "round(§)",
        "params": ["x"]
      },
      "truncatum": {
        "template": "math.trunc(§)",
        "params": ["x"]
      },
      "radix": {
        "template": "math.sqrt(§)",
        "params": ["x"]
      },
      "potentia": {
        "template": "math.pow(§, §)",
        "params": ["base","exp"]
      },
      "logarithmus": {
        "template": "math.log(§)",
        "params": ["x"]
      },
      "logarithmus10": {
        "template": "math.log10(§)",
        "params": ["x"]
      },
      "exponens": {
        "template": "math.exp(§)",
        "params": ["x"]
      },
      "sinus": {
        "template": "math.sin(§)",
        "params": ["x"]
      },
      "cosinus": {
        "template": "math.cos(§)",
        "params": ["x"]
      },
      "tangens": {
        "template": "math.tan(§)",
        "params": ["x"]
      },
      "absolutum": {
        "template": "abs(§)",
        "params": ["x"]
      },
      "signum": {
        "template": "(1 if §0 > 0 else (-1 if §0 < 0 else 0))",
        "params": ["x"]
      },
      "minimus": {
        "template": "min(§, §)",
        "params": ["a","b"]
      },
      "maximus": {
        "template": "max(§, §)",
        "params": ["a","b"]
      },
      "constringens": {
        "template": "max(§1, min(§0, §2))",
        "params": ["x","lo","hi"]
      },
      "PI": {
        "template": "math.pi",
        "params": []
      },
      "E": {
        "template": "math.e",
        "params": []
      },
      "TAU": {
        "template": "(math.pi * 2)",
        "params": []
      }
    }
    , "innatum": "math"
  },
  "aleator": {
    "methods": {
      "fractus": {
        "template": "random.random()",
        "params": []
      },
      "inter": {
        "template": "random.randint(§0, §1)",
        "params": ["min","max"]
      },
      "octeti": {
        "template": "secrets.token_bytes(§)",
        "params": ["n"]
      },
      "uuid": {
        "template": "str(uuid.uuid4())",
        "params": []
      },
      "selige": {
        "template": "random.choice(§0)",
        "params": ["lista"]
      },
      "miscita": {
        "template": "random.sample(§0, len(§0))",
        "params": ["lista"]
      },
      "semen": {
        "template": "random.seed(§)",
        "params": ["n"]
      }
    }
    , "innatum": "random"
  },
  "solum": {
    "methods": {
      "legens": {
        "template": "open(§, 'rb')",
        "params": ["path"]
      },
      "leget": {
        "template": "aiofiles.open(§).read()",
        "params": ["path"]
      },
      "lege": {
        "template": "open(§).read()",
        "params": ["path"]
      },
      "ausculta": {
        "template": "sys.stdin",
        "params": []
      },
      "hauri": {
        "template": "sys.stdin.read()",
        "params": []
      },
      "scribens": {
        "template": "open(§, 'wb')",
        "params": ["path"]
      },
      "scribet": {
        "template": "aiofiles.open(§, 'w').write(§)",
        "params": ["path","data"]
      },
      "inscribe": {
        "template": "open(§, 'w').write(§)",
        "params": ["path","data"]
      },
      "apponet": {
        "template": "aiofiles.open(§, 'a').write(§)",
        "params": ["path","data"]
      },
      "appone": {
        "template": "open(§, 'a').write(§)",
        "params": ["path","data"]
      },
      "exstat": {
        "template": "os.path.exists(§)",
        "params": ["path"]
      },
      "inspice": {
        "template": "os.stat(§)",
        "params": ["path"]
      },
      "dele": {
        "template": "os.remove(§)",
        "params": ["path"]
      },
      "duplica": {
        "template": "shutil.copy2(§, §)",
        "params": ["src","dest"]
      },
      "move": {
        "template": "shutil.move(§, §)",
        "params": ["src","dest"]
      },
      "trunca": {
        "template": "os.truncate(§, §)",
        "params": ["path","size"]
      },
      "tange": {
        "template": "pathlib.Path(§).touch()",
        "params": ["path"]
      },
      "crea": {
        "template": "os.makedirs(§, exist_ok=True)",
        "params": ["path"]
      },
      "elenca": {
        "template": "os.listdir(§)",
        "params": ["path"]
      },
      "ambula": {
        "template": "os.walk(§)",
        "params": ["path"]
      },
      "vacua": {
        "template": "os.rmdir(§)",
        "params": ["path"]
      },
      "deleArborem": {
        "template": "shutil.rmtree(§)",
        "params": ["path"]
      },
      "iunge": {
        "template": "os.path.join(*§)",
        "params": ["parts"]
      },
      "dir": {
        "template": "os.path.dirname(§)",
        "params": ["path"]
      },
      "basis": {
        "template": "os.path.basename(§)",
        "params": ["path"]
      },
      "extensio": {
        "template": "os.path.splitext(§)[1]",
        "params": ["path"]
      },
      "resolve": {
        "template": "os.path.abspath(§)",
        "params": ["path"]
      },
      "domus": {
        "template": "os.path.expanduser('~')",
        "params": []
      }
    }
    , "innatum": "os"
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
        "template": "§0[§1] = §2",
        "params": ["ego","k","v"]
      },
      "accipe": {
        "template": "§0.get(§1)",
        "params": ["ego","k"]
      },
      "accipeAut": {
        "template": "§0.get(§1, §2)",
        "params": ["ego","k","def"]
      },
      "habet": {
        "template": "(§1 in §0)",
        "params": ["ego","k"]
      },
      "dele": {
        "template": "del §0[§1]",
        "params": ["ego","k"]
      },
      "longitudo": {
        "template": "len(§)",
        "params": ["ego"]
      },
      "vacua": {
        "template": "len(§) == 0",
        "params": ["ego"]
      },
      "purga": {
        "method": "clear"
      },
      "claves": {
        "method": "keys"
      },
      "valores": {
        "method": "values"
      },
      "paria": {
        "method": "items"
      },
      "selecta": {
        "template": "{k: v for k, v in §0.items() if k in [§1]}",
        "params": ["ego","claves"]
      },
      "omissa": {
        "template": "{k: v for k, v in §0.items() if k not in [§1]}",
        "params": ["ego","claves"]
      },
      "conflata": {
        "template": "{**§0, **§1}",
        "params": ["ego","alia"]
      },
      "inversa": {
        "template": "{v: k for k, v in §.items()}",
        "params": ["ego"]
      },
      "mappaValores": {
        "template": "{k: (§1)(v) for k, v in §0.items()}",
        "params": ["ego","fn"]
      },
      "mappaClaves": {
        "template": "{(§1)(k): v for k, v in §0.items()}",
        "params": ["ego","fn"]
      },
      "inLista": {
        "template": "list(§.items())",
        "params": ["ego"]
      },
      "inObjectum": {
        "template": "dict(§)",
        "params": ["ego"]
      }
    }
    , "innatum": "dict"
  },
  "numerus": {
    "methods": {
      "absolutum": {
        "template": "abs(§)",
        "params": ["ego"]
      },
      "signum": {
        "template": "((§0>0)-(§0<0))",
        "params": ["ego"]
      },
      "minimus": {
        "template": "min(§, §)",
        "params": ["ego","other"]
      },
      "maximus": {
        "template": "max(§, §)",
        "params": ["ego","other"]
      }
    }
    , "innatum": "int"
  },
  "fractus": {
    "methods": {
      "absolutum": {
        "template": "abs(§)",
        "params": ["ego"]
      },
      "signum": {
        "template": "math.copysign(1, §)",
        "params": ["ego"]
      },
      "minimus": {
        "template": "min(§, §)",
        "params": ["ego","other"]
      },
      "maximus": {
        "template": "max(§, §)",
        "params": ["ego","other"]
      },
      "rotunda": {
        "template": "round(§)",
        "params": ["ego"]
      },
      "pavimentum": {
        "template": "math.floor(§)",
        "params": ["ego"]
      },
      "tectum": {
        "template": "math.ceil(§)",
        "params": ["ego"]
      },
      "trunca": {
        "template": "math.trunc(§)",
        "params": ["ego"]
      }
    }
    , "innatum": "float"
  },
  "textus": {
    "methods": {
      "longitudo": {
        "template": "len(§)",
        "params": ["ego"]
      },
      "sectio": {
        "template": "§[§:§]",
        "params": ["ego","start","end"]
      },
      "continet": {
        "template": "§ in §",
        "params": ["ego","sub"]
      },
      "initium": {
        "method": "startswith"
      },
      "finis": {
        "method": "endswith"
      },
      "maiuscula": {
        "method": "upper"
      },
      "minuscula": {
        "method": "lower"
      },
      "recide": {
        "method": "strip"
      },
      "divide": {
        "method": "split"
      },
      "muta": {
        "method": "replace"
      }
    }
    , "innatum": "str"
  },
  "copia": {
    "methods": {
      "adde": {
        "method": "add"
      },
      "habet": {
        "template": "(§1 in §0)",
        "params": ["ego","elem"]
      },
      "dele": {
        "method": "discard"
      },
      "longitudo": {
        "template": "len(§)",
        "params": ["ego"]
      },
      "vacua": {
        "template": "len(§) == 0",
        "params": ["ego"]
      },
      "purga": {
        "method": "clear"
      },
      "valores": {
        "template": "iter(§)",
        "params": ["ego"]
      },
      "perambula": {
        "template": "[(§1)(x) for x in §0]",
        "params": ["ego","fn"]
      },
      "unio": {
        "template": "§0 | §1",
        "params": ["ego","alia"]
      },
      "intersectio": {
        "template": "§0 & §1",
        "params": ["ego","alia"]
      },
      "differentia": {
        "template": "§0 - §1",
        "params": ["ego","alia"]
      },
      "symmetrica": {
        "template": "§0 ^ §1",
        "params": ["ego","alia"]
      },
      "subcopia": {
        "template": "§0 <= §1",
        "params": ["ego","alia"]
      },
      "supercopia": {
        "template": "§0 >= §1",
        "params": ["ego","alia"]
      },
      "inLista": {
        "template": "list(§)",
        "params": ["ego"]
      }
    }
    , "innatum": "set"
  },
  "lista": {
    "methods": {
      "adde": {
        "method": "append"
      },
      "addita": {
        "template": "[*§, §]",
        "params": ["ego","elem"]
      },
      "praepone": {
        "template": "§.insert(0, §)",
        "params": ["ego","elem"]
      },
      "praeposita": {
        "template": "[§1, *§0]",
        "params": ["ego","elem"]
      },
      "remove": {
        "method": "pop"
      },
      "remota": {
        "template": "§[:-1]",
        "params": ["ego"]
      },
      "decapita": {
        "template": "§.pop(0)",
        "params": ["ego"]
      },
      "decapitata": {
        "template": "§[1:]",
        "params": ["ego"]
      },
      "purga": {
        "method": "clear"
      },
      "primus": {
        "template": "§[0]",
        "params": ["ego"]
      },
      "ultimus": {
        "template": "§[-1]",
        "params": ["ego"]
      },
      "accipe": {
        "template": "§[§]",
        "params": ["ego","idx"]
      },
      "longitudo": {
        "template": "len(§)",
        "params": ["ego"]
      },
      "vacua": {
        "template": "len(§) == 0",
        "params": ["ego"]
      },
      "continet": {
        "template": "(§1 in §0)",
        "params": ["ego","elem"]
      },
      "indiceDe": {
        "template": "§0.index(§1)",
        "params": ["ego","elem"]
      },
      "inveni": {
        "template": "next(filter(§1, §0), None)",
        "params": ["ego","pred"]
      },
      "inveniIndicem": {
        "template": "next((i for i, x in enumerate(§0) if (§1)(x)), -1)",
        "params": ["ego","pred"]
      },
      "omnes": {
        "template": "all(map(§, §))",
        "params": ["ego","pred"]
      },
      "aliquis": {
        "template": "any(map(§, §))",
        "params": ["ego","pred"]
      },
      "filtrata": {
        "template": "list(filter(§, §))",
        "params": ["ego","pred"]
      },
      "mappata": {
        "template": "list(map(§, §))",
        "params": ["ego","fn"]
      },
      "explanata": {
        "template": "[y for x in § for y in (§)(x)]",
        "params": ["ego","fn"]
      },
      "plana": {
        "template": "[y for x in § for y in x]",
        "params": ["ego"]
      },
      "inversa": {
        "template": "§[::-1]",
        "params": ["ego"]
      },
      "ordinata": {
        "template": "sorted(§)",
        "params": ["ego"]
      },
      "sectio": {
        "template": "§[§:§]",
        "params": ["ego","start","end"]
      },
      "prima": {
        "template": "§[:§]",
        "params": ["ego","n"]
      },
      "ultima": {
        "template": "§[-§:]",
        "params": ["ego","n"]
      },
      "omissa": {
        "template": "§[§:]",
        "params": ["ego","n"]
      },
      "reducta": {
        "template": "functools.reduce(§, §, §)",
        "params": ["ego","fn","init"]
      },
      "filtra": {
        "template": "§[:] = [x for x in § if (§)(x)]",
        "params": ["ego","pred"]
      },
      "ordina": {
        "template": "§.sort()",
        "params": ["ego"]
      },
      "inverte": {
        "template": "§.reverse()",
        "params": ["ego"]
      },
      "perambula": {
        "template": "[(§)(x) for x in §]",
        "params": ["ego","fn"]
      },
      "coniunge": {
        "template": "§.join(§)",
        "params": ["ego","sep"]
      },
      "summa": {
        "template": "sum(§)",
        "params": ["ego"]
      },
      "medium": {
        "template": "(sum(§0) / len(§0))",
        "params": ["ego"]
      },
      "minimus": {
        "template": "min(§)",
        "params": ["ego"]
      },
      "maximus": {
        "template": "max(§)",
        "params": ["ego"]
      },
      "numera": {
        "template": "sum(1 for x in § if (§)(x))",
        "params": ["ego","pred"]
      },
      "congrega": {
        "template": "{k: list(g) for k, g in itertools.groupby(sorted(§, key=§), key=§)}",
        "params": ["ego","fn"]
      },
      "unica": {
        "template": "list(dict.fromkeys(§))",
        "params": ["ego"]
      },
      "fragmenta": {
        "template": "[§[i:i+§] for i in range(0, len(§), §)]",
        "params": ["ego","n"]
      },
      "densa": {
        "template": "[x for x in § if x]",
        "params": ["ego"]
      },
      "partire": {
        "template": "[[x for x in § if (§)(x)], [x for x in § if not (§)(x)]]",
        "params": ["ego","pred"]
      },
      "miscita": {
        "template": "random.shuffle(§)",
        "params": ["ego"]
      },
      "specimen": {
        "template": "random.choice(§)",
        "params": ["ego"]
      },
      "specimina": {
        "template": "random.sample(§, §)",
        "params": ["ego","n"]
      }
    }
    , "innatum": "list"
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
