/**
 * Generated TS norma registry.
 * Source: fons/norma/
 * Generator: bun run build:norma
 * Generated: 2026-01-15T14:30:37.478Z
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
        "template": "JSON.stringify(§)",
        "params": ["val"]
      },
      "solvePulchre": {
        "template": "JSON.stringify(§, null, §)",
        "params": ["val","indentum"]
      },
      "pange": {
        "template": "JSON.parse(§)",
        "params": ["textus"]
      },
      "estNihil": {
        "template": "(§ === null)",
        "params": ["val"]
      },
      "estBivalens": {
        "template": "(typeof § === 'boolean')",
        "params": ["val"]
      },
      "estNumerus": {
        "template": "(typeof § === 'number')",
        "params": ["val"]
      },
      "estTextus": {
        "template": "(typeof § === 'string')",
        "params": ["val"]
      },
      "estLista": {
        "template": "Array.isArray(§)",
        "params": ["val"]
      },
      "estTabula": {
        "template": "(typeof § === 'object' && § !== null && !Array.isArray(§))",
        "params": ["val"]
      },
      "utTextus": {
        "template": "(typeof § === 'string' ? § : '')",
        "params": ["val"]
      },
      "utNumerus": {
        "template": "(typeof § === 'number' ? § : 0)",
        "params": ["val"]
      },
      "utBivalens": {
        "template": "(typeof § === 'boolean' ? § : false)",
        "params": ["val"]
      },
      "cape": {
        "template": "(§?.[§] ?? null)",
        "params": ["val","key"]
      },
      "capeIndice": {
        "template": "(§?.[§] ?? null)",
        "params": ["val","idx"]
      }
    }
    , "innatum": "JSON"
  },
  "tempus": {
    "methods": {
      "nunc": {
        "template": "Date.now()",
        "params": []
      },
      "nunc_nano": {
        "template": "BigInt(Date.now()) * 1000000n",
        "params": []
      },
      "nunc_secunda": {
        "template": "Math.floor(Date.now() / 1000)",
        "params": []
      },
      "dormi": {
        "template": "new Promise(r => setTimeout(r, §))",
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
    , "innatum": "Date"
  },
  "mathesis": {
    "methods": {
      "pavimentum": {
        "template": "Math.floor(§)",
        "params": ["x"]
      },
      "tectum": {
        "template": "Math.ceil(§)",
        "params": ["x"]
      },
      "rotundum": {
        "template": "Math.round(§)",
        "params": ["x"]
      },
      "truncatum": {
        "template": "Math.trunc(§)",
        "params": ["x"]
      },
      "radix": {
        "template": "Math.sqrt(§)",
        "params": ["x"]
      },
      "potentia": {
        "template": "Math.pow(§, §)",
        "params": ["base","exp"]
      },
      "logarithmus": {
        "template": "Math.log(§)",
        "params": ["x"]
      },
      "logarithmus10": {
        "template": "Math.log10(§)",
        "params": ["x"]
      },
      "exponens": {
        "template": "Math.exp(§)",
        "params": ["x"]
      },
      "sinus": {
        "template": "Math.sin(§)",
        "params": ["x"]
      },
      "cosinus": {
        "template": "Math.cos(§)",
        "params": ["x"]
      },
      "tangens": {
        "template": "Math.tan(§)",
        "params": ["x"]
      },
      "absolutum": {
        "template": "Math.abs(§)",
        "params": ["x"]
      },
      "signum": {
        "template": "Math.sign(§)",
        "params": ["x"]
      },
      "minimus": {
        "template": "Math.min(§, §)",
        "params": ["a","b"]
      },
      "maximus": {
        "template": "Math.max(§, §)",
        "params": ["a","b"]
      },
      "constringens": {
        "template": "Math.min(Math.max(§, §), §)",
        "params": ["x","lo","hi"]
      },
      "PI": {
        "template": "Math.PI",
        "params": []
      },
      "E": {
        "template": "Math.E",
        "params": []
      },
      "TAU": {
        "template": "(Math.PI * 2)",
        "params": []
      }
    }
    , "innatum": "Math"
  },
  "aleator": {
    "methods": {
      "fractus": {
        "template": "Math.random()",
        "params": []
      },
      "inter": {
        "template": "Math.floor(Math.random() * (§1 - §0 + 1)) + §0",
        "params": ["min","max"]
      },
      "octeti": {
        "template": "crypto.getRandomValues(new Uint8Array(§))",
        "params": ["n"]
      },
      "uuid": {
        "template": "crypto.randomUUID()",
        "params": []
      },
      "selige": {
        "template": "§0[Math.floor(Math.random() * §0.length)]",
        "params": ["lista"]
      },
      "miscita": {
        "template": "(() => { const a = [...§0]; for (let i = a.length - 1; i > 0; i--) { const j = Math.floor(Math.random() * (i + 1)); [a[i], a[j]] = [a[j], a[i]]; } return a; })()",
        "params": ["lista"]
      },
      "semen": {
        "template": "undefined /* JS Math.random is not seedable */",
        "params": ["n"]
      }
    }
    , "innatum": "crypto"
  },
  "solum": {
    "methods": {
      "legens": {
        "template": "fs.createReadStream(§)",
        "params": ["path"]
      },
      "leget": {
        "template": "fs.promises.readFile(§, 'utf-8')",
        "params": ["path"]
      },
      "lege": {
        "template": "fs.readFileSync(§, 'utf-8')",
        "params": ["path"]
      },
      "ausculta": {
        "template": "readline.createInterface({ input: process.stdin })",
        "params": []
      },
      "hauri": {
        "template": "fs.readFileSync(0, 'utf-8')",
        "params": []
      },
      "scribens": {
        "template": "fs.createWriteStream(§)",
        "params": ["path"]
      },
      "scribet": {
        "template": "fs.promises.writeFile(§, §)",
        "params": ["path","data"]
      },
      "inscribe": {
        "template": "fs.writeFileSync(§, §)",
        "params": ["path","data"]
      },
      "apponet": {
        "template": "fs.promises.appendFile(§, §)",
        "params": ["path","data"]
      },
      "appone": {
        "template": "fs.appendFileSync(§, §)",
        "params": ["path","data"]
      },
      "exstat": {
        "template": "fs.existsSync(§)",
        "params": ["path"]
      },
      "inspice": {
        "template": "fs.promises.stat(§)",
        "params": ["path"]
      },
      "dele": {
        "template": "fs.promises.unlink(§)",
        "params": ["path"]
      },
      "duplica": {
        "template": "fs.promises.copyFile(§, §)",
        "params": ["src","dest"]
      },
      "move": {
        "template": "fs.promises.rename(§, §)",
        "params": ["src","dest"]
      },
      "trunca": {
        "template": "fs.promises.truncate(§, §)",
        "params": ["path","size"]
      },
      "tange": {
        "template": "fs.promises.utimes(§, Date.now(), Date.now())",
        "params": ["path"]
      },
      "crea": {
        "template": "fs.promises.mkdir(§, { recursive: true })",
        "params": ["path"]
      },
      "elenca": {
        "template": "fs.promises.readdir(§)",
        "params": ["path"]
      },
      "ambula": {
        "template": "glob.stream(§ + '/**/*')",
        "params": ["path"]
      },
      "vacua": {
        "template": "fs.promises.rmdir(§)",
        "params": ["path"]
      },
      "deleArborem": {
        "template": "fs.promises.rm(§, { recursive: true })",
        "params": ["path"]
      },
      "iunge": {
        "template": "path.join(...§)",
        "params": ["parts"]
      },
      "dir": {
        "template": "path.dirname(§)",
        "params": ["path"]
      },
      "basis": {
        "template": "path.basename(§)",
        "params": ["path"]
      },
      "extensio": {
        "template": "path.extname(§)",
        "params": ["path"]
      },
      "resolve": {
        "template": "path.resolve(§)",
        "params": ["path"]
      },
      "domus": {
        "template": "os.homedir()",
        "params": []
      }
    }
    , "innatum": "fs"
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
        "method": "set"
      },
      "accipe": {
        "method": "get"
      },
      "accipeAut": {
        "template": "(§0.get(§1) ?? §2)",
        "params": ["ego","k","def"]
      },
      "habet": {
        "method": "has"
      },
      "dele": {
        "method": "delete"
      },
      "longitudo": {
        "template": "§.size",
        "params": ["ego"]
      },
      "vacua": {
        "template": "§.size === 0",
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
        "method": "entries"
      },
      "selecta": {
        "template": "new Map([...§0].filter(([k]) => [§1].flat().includes(k)))",
        "params": ["ego","claves"]
      },
      "omissa": {
        "template": "new Map([...§0].filter(([k]) => ![§1].flat().includes(k)))",
        "params": ["ego","claves"]
      },
      "conflata": {
        "template": "new Map([...§0, ...§1])",
        "params": ["ego","alia"]
      },
      "inversa": {
        "template": "new Map([...§].map(([k, v]) => [v, k]))",
        "params": ["ego"]
      },
      "mappaValores": {
        "template": "new Map([...§0].map(([k, v]) => [k, (§1)(v)]))",
        "params": ["ego","fn"]
      },
      "mappaClaves": {
        "template": "new Map([...§0].map(([k, v]) => [(§1)(k), v]))",
        "params": ["ego","fn"]
      },
      "inLista": {
        "template": "[...§]",
        "params": ["ego"]
      },
      "inObjectum": {
        "template": "Object.fromEntries(§)",
        "params": ["ego"]
      }
    }
    , "innatum": "Map"
  },
  "numerus": {
    "methods": {
      "absolutum": {
        "template": "Math.abs(§)",
        "params": ["ego"]
      },
      "signum": {
        "template": "Math.sign(§)",
        "params": ["ego"]
      },
      "minimus": {
        "template": "Math.min(§, §)",
        "params": ["ego","other"]
      },
      "maximus": {
        "template": "Math.max(§, §)",
        "params": ["ego","other"]
      }
    }
    , "innatum": "number"
  },
  "fractus": {
    "methods": {
      "absolutum": {
        "template": "Math.abs(§)",
        "params": ["ego"]
      },
      "signum": {
        "template": "Math.sign(§)",
        "params": ["ego"]
      },
      "minimus": {
        "template": "Math.min(§, §)",
        "params": ["ego","other"]
      },
      "maximus": {
        "template": "Math.max(§, §)",
        "params": ["ego","other"]
      },
      "rotunda": {
        "template": "Math.round(§)",
        "params": ["ego"]
      },
      "pavimentum": {
        "template": "Math.floor(§)",
        "params": ["ego"]
      },
      "tectum": {
        "template": "Math.ceil(§)",
        "params": ["ego"]
      },
      "trunca": {
        "template": "Math.trunc(§)",
        "params": ["ego"]
      }
    }
    , "innatum": "number"
  },
  "textus": {
    "methods": {
      "longitudo": {
        "template": "§.length",
        "params": ["ego"]
      },
      "sectio": {
        "method": "slice"
      },
      "continet": {
        "method": "includes"
      },
      "initium": {
        "method": "startsWith"
      },
      "finis": {
        "method": "endsWith"
      },
      "maiuscula": {
        "method": "toUpperCase"
      },
      "minuscula": {
        "method": "toLowerCase"
      },
      "recide": {
        "method": "trim"
      },
      "divide": {
        "method": "split"
      },
      "muta": {
        "method": "replaceAll"
      }
    }
    , "innatum": "string"
  },
  "copia": {
    "methods": {
      "adde": {
        "method": "add"
      },
      "habet": {
        "method": "has"
      },
      "dele": {
        "method": "delete"
      },
      "longitudo": {
        "template": "§.size",
        "params": ["ego"]
      },
      "vacua": {
        "template": "§.size === 0",
        "params": ["ego"]
      },
      "purga": {
        "method": "clear"
      },
      "valores": {
        "method": "values"
      },
      "perambula": {
        "method": "forEach"
      },
      "unio": {
        "template": "new Set([...§0, ...§1])",
        "params": ["ego","alia"]
      },
      "intersectio": {
        "template": "new Set([...§0].filter(x => §1.has(x)))",
        "params": ["ego","alia"]
      },
      "differentia": {
        "template": "new Set([...§0].filter(x => !§1.has(x)))",
        "params": ["ego","alia"]
      },
      "symmetrica": {
        "template": "new Set([...[...§0].filter(x => !§1.has(x)), ...[...§1].filter(x => !§0.has(x))])",
        "params": ["ego","alia"]
      },
      "subcopia": {
        "template": "[...§0].every(x => §1.has(x))",
        "params": ["ego","alia"]
      },
      "supercopia": {
        "template": "[...§1].every(x => §0.has(x))",
        "params": ["ego","alia"]
      },
      "inLista": {
        "template": "[...§]",
        "params": ["ego"]
      }
    }
    , "innatum": "Set"
  },
  "lista": {
    "methods": {
      "adde": {
        "method": "push"
      },
      "addita": {
        "template": "[...§, §]",
        "params": ["ego","elem"]
      },
      "praepone": {
        "method": "unshift"
      },
      "praeposita": {
        "template": "[§1, ...§0]",
        "params": ["ego","elem"]
      },
      "remove": {
        "method": "pop"
      },
      "remota": {
        "template": "§.slice(0, -1)",
        "params": ["ego"]
      },
      "decapita": {
        "method": "shift"
      },
      "decapitata": {
        "template": "§.slice(1)",
        "params": ["ego"]
      },
      "purga": {
        "template": "§.length = 0",
        "params": ["ego"]
      },
      "primus": {
        "template": "§[0]",
        "params": ["ego"]
      },
      "ultimus": {
        "template": "§.at(-1)",
        "params": ["ego"]
      },
      "accipe": {
        "template": "§[§]",
        "params": ["ego","idx"]
      },
      "longitudo": {
        "template": "§.length",
        "params": ["ego"]
      },
      "vacua": {
        "template": "§.length === 0",
        "params": ["ego"]
      },
      "continet": {
        "method": "includes"
      },
      "indiceDe": {
        "method": "indexOf"
      },
      "inveni": {
        "method": "find"
      },
      "inveniIndicem": {
        "method": "findIndex"
      },
      "omnes": {
        "method": "every"
      },
      "aliquis": {
        "method": "some"
      },
      "filtrata": {
        "method": "filter"
      },
      "mappata": {
        "method": "map"
      },
      "explanata": {
        "method": "flatMap"
      },
      "plana": {
        "method": "flat"
      },
      "inversa": {
        "template": "[...§].reverse()",
        "params": ["ego"]
      },
      "ordinata": {
        "template": "[...§].sort()",
        "params": ["ego"]
      },
      "sectio": {
        "method": "slice"
      },
      "prima": {
        "template": "§.slice(0, §)",
        "params": ["ego","n"]
      },
      "ultima": {
        "template": "§.slice(-§)",
        "params": ["ego","n"]
      },
      "omissa": {
        "template": "§.slice(§)",
        "params": ["ego","n"]
      },
      "reducta": {
        "method": "reduce"
      },
      "filtra": {
        "template": "(() => { for (let i = §.length - 1; i >= 0; i--) { if (!(§)(§[i])) §.splice(i, 1); } })()",
        "params": ["ego","pred"]
      },
      "ordina": {
        "template": "§.sort()",
        "params": ["ego"]
      },
      "inverte": {
        "method": "reverse"
      },
      "perambula": {
        "method": "forEach"
      },
      "coniunge": {
        "method": "join"
      },
      "summa": {
        "template": "§.reduce((a, b) => a + b, 0)",
        "params": ["ego"]
      },
      "medium": {
        "template": "(§0.reduce((a, b) => a + b, 0) / §0.length)",
        "params": ["ego"]
      },
      "minimus": {
        "template": "Math.min(...§)",
        "params": ["ego"]
      },
      "maximus": {
        "template": "Math.max(...§)",
        "params": ["ego"]
      },
      "minimusPer": {
        "template": "§.reduce((min, x) => (§)(x) < (§)(min) ? x : min)",
        "params": ["ego","fn"]
      },
      "maximusPer": {
        "template": "§.reduce((max, x) => (§)(x) > (§)(max) ? x : max)",
        "params": ["ego","fn"]
      },
      "numera": {
        "template": "§.filter(§).length",
        "params": ["ego","pred"]
      },
      "congrega": {
        "template": "Object.groupBy(§, §)",
        "params": ["ego","fn"]
      },
      "unica": {
        "template": "[...new Set(§)]",
        "params": ["ego"]
      },
      "planaOmnia": {
        "template": "§.flat(Infinity)",
        "params": ["ego"]
      },
      "fragmenta": {
        "template": "Array.from({ length: Math.ceil(§.length / §) }, (_, i) => §.slice(i * §, i * § + §))",
        "params": ["ego","n"]
      },
      "densa": {
        "template": "§.filter(Boolean)",
        "params": ["ego"]
      },
      "partire": {
        "template": "§.reduce(([t, f], x) => (§)(x) ? [[...t, x], f] : [t, [...f, x]], [[], []])",
        "params": ["ego","pred"]
      },
      "miscita": {
        "template": "(() => { const a = [...§]; for (let i = a.length - 1; i > 0; i--) { const j = Math.floor(Math.random() * (i + 1)); [a[i], a[j]] = [a[j], a[i]]; } return a; })()",
        "params": ["ego"]
      },
      "specimen": {
        "template": "§0[Math.floor(Math.random() * §0.length)]",
        "params": ["ego"]
      },
      "specimina": {
        "template": "(() => { const a = [...§]; for (let i = a.length - 1; i > 0; i--) { const j = Math.floor(Math.random() * (i + 1)); [a[i], a[j]] = [a[j], a[i]]; } return a.slice(0, §); })()",
        "params": ["ego","n"]
      }
    }
    , "innatum": "Array"
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
