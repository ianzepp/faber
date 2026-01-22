/**
 * Generated TS norma registry.
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
      "appende": {
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
