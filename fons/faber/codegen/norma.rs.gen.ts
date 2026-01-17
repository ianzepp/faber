/**
 * Generated RS norma registry.
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
        "template": "§.floor()",
        "params": ["x"]
      },
      "tectum": {
        "template": "§.ceil()",
        "params": ["x"]
      },
      "rotundum": {
        "template": "§.round()",
        "params": ["x"]
      },
      "truncatum": {
        "template": "§.trunc()",
        "params": ["x"]
      },
      "radix": {
        "template": "§.sqrt()",
        "params": ["x"]
      },
      "potentia": {
        "template": "§.powf(§)",
        "params": ["base","exp"]
      },
      "logarithmus": {
        "template": "§.ln()",
        "params": ["x"]
      },
      "logarithmus10": {
        "template": "§.log10()",
        "params": ["x"]
      },
      "exponens": {
        "template": "§.exp()",
        "params": ["x"]
      },
      "sinus": {
        "template": "§.sin()",
        "params": ["x"]
      },
      "cosinus": {
        "template": "§.cos()",
        "params": ["x"]
      },
      "tangens": {
        "template": "§.tan()",
        "params": ["x"]
      },
      "absolutum": {
        "template": "§.abs()",
        "params": ["x"]
      },
      "signum": {
        "template": "§.signum()",
        "params": ["x"]
      },
      "minimus": {
        "template": "§.min(§)",
        "params": ["a","b"]
      },
      "maximus": {
        "template": "§.max(§)",
        "params": ["a","b"]
      },
      "constringens": {
        "template": "§.clamp(§, §)",
        "params": ["x","lo","hi"]
      },
      "PI": {
        "template": "std::f64::consts::PI",
        "params": []
      },
      "E": {
        "template": "std::f64::consts::E",
        "params": []
      },
      "TAU": {
        "template": "std::f64::consts::TAU",
        "params": []
      }
    }
    , "innatum": "f64"
  },
  "tabula": {
    "methods": {
      "pone": {
        "template": "§0.insert(§1, §2)",
        "params": ["ego","k","v"]
      },
      "accipe": {
        "template": "§0.get(&§1)",
        "params": ["ego","k"]
      },
      "accipeAut": {
        "template": "§0.get(&§1).cloned().unwrap_or(§2)",
        "params": ["ego","k","def"]
      },
      "habet": {
        "template": "§0.contains_key(&§1)",
        "params": ["ego","k"]
      },
      "dele": {
        "template": "§0.remove(&§1)",
        "params": ["ego","k"]
      },
      "longitudo": {
        "template": "§.len()",
        "params": ["ego"]
      },
      "vacua": {
        "template": "§.is_empty()",
        "params": ["ego"]
      },
      "purga": {
        "method": "clear"
      },
      "claves": {
        "template": "§.keys()",
        "params": ["ego"]
      },
      "valores": {
        "template": "§.values()",
        "params": ["ego"]
      },
      "paria": {
        "template": "§.iter()",
        "params": ["ego"]
      },
      "selecta": {
        "template": "faber::tabula_selecta(&§0, &§1)",
        "params": ["ego","claves"]
      },
      "omissa": {
        "template": "faber::tabula_omissa(&§0, &§1)",
        "params": ["ego","claves"]
      },
      "conflata": {
        "template": "faber::tabula_conflata(&§0, &§1)",
        "params": ["ego","alia"]
      },
      "inversa": {
        "template": "faber::tabula_inversa(&§)",
        "params": ["ego"]
      },
      "inLista": {
        "template": "faber::tabula_in_lista(&§)",
        "params": ["ego"]
      },
    }
    , "innatum": "HashMap"
  },
  "numerus": {
    "methods": {
      "absolutum": {
        "template": "§.abs()",
        "params": ["ego"]
      },
      "signum": {
        "template": "§.signum()",
        "params": ["ego"]
      },
      "minimus": {
        "template": "std::cmp::min(§, §)",
        "params": ["ego","other"]
      },
      "maximus": {
        "template": "std::cmp::max(§, §)",
        "params": ["ego","other"]
      }
    }
    , "innatum": "i64"
  },
  "fractus": {
    "methods": {
      "absolutum": {
        "template": "§.abs()",
        "params": ["ego"]
      },
      "signum": {
        "template": "§.signum()",
        "params": ["ego"]
      },
      "minimus": {
        "template": "§.min(§)",
        "params": ["ego","other"]
      },
      "maximus": {
        "template": "§.max(§)",
        "params": ["ego","other"]
      },
      "rotunda": {
        "template": "§.round() as i64",
        "params": ["ego"]
      },
      "pavimentum": {
        "template": "§.floor() as i64",
        "params": ["ego"]
      },
      "tectum": {
        "template": "§.ceil() as i64",
        "params": ["ego"]
      },
      "trunca": {
        "template": "§.trunc() as i64",
        "params": ["ego"]
      }
    }
    , "innatum": "f64"
  },
  "textus": {
    "methods": {
      "longitudo": {
        "template": "§.len()",
        "params": ["ego"]
      },
      "sectio": {
        "template": "&§[§..§]",
        "params": ["ego","start","end"]
      },
      "continet": {
        "method": "contains"
      },
      "initium": {
        "method": "starts_with"
      },
      "finis": {
        "method": "ends_with"
      },
      "maiuscula": {
        "method": "to_uppercase"
      },
      "minuscula": {
        "method": "to_lowercase"
      },
      "recide": {
        "method": "trim"
      },
      "divide": {
        "template": "§.split(§).collect::<Vec<_>>()",
        "params": ["ego","sep"]
      },
      "muta": {
        "method": "replace"
      }
    }
    , "innatum": "&str"
  },
  "copia": {
    "methods": {
      "adde": {
        "method": "insert"
      },
      "habet": {
        "template": "§0.contains(&§1)",
        "params": ["ego","elem"]
      },
      "dele": {
        "template": "§0.remove(&§1)",
        "params": ["ego","elem"]
      },
      "longitudo": {
        "template": "§.len()",
        "params": ["ego"]
      },
      "vacua": {
        "template": "§.is_empty()",
        "params": ["ego"]
      },
      "purga": {
        "method": "clear"
      },
      "valores": {
        "template": "§.iter()",
        "params": ["ego"]
      },
      "perambula": {
        "template": "§0.iter().for_each(§1)",
        "params": ["ego","fn"]
      },
      "unio": {
        "template": "faber::copia_unio(&§0, &§1)",
        "params": ["ego","alia"]
      },
      "intersectio": {
        "template": "faber::copia_intersectio(&§0, &§1)",
        "params": ["ego","alia"]
      },
      "differentia": {
        "template": "faber::copia_differentia(&§0, &§1)",
        "params": ["ego","alia"]
      },
      "symmetrica": {
        "template": "faber::copia_symmetrica(&§0, &§1)",
        "params": ["ego","alia"]
      },
      "subcopia": {
        "template": "§0.is_subset(&§1)",
        "params": ["ego","alia"]
      },
      "supercopia": {
        "template": "§0.is_superset(&§1)",
        "params": ["ego","alia"]
      },
      "inLista": {
        "template": "faber::copia_in_lista(&§)",
        "params": ["ego"]
      }
    }
    , "innatum": "HashSet"
  },
  "lista": {
    "methods": {
      "adde": {
        "method": "push"
      },
      "addita": {
        "template": "faber::lista_addita(&§0, §1)",
        "params": ["ego","elem"]
      },
      "praepone": {
        "template": "§.insert(0, §)",
        "params": ["ego","elem"]
      },
      "praeposita": {
        "template": "faber::lista_praeposita(&§0, §1)",
        "params": ["ego","elem"]
      },
      "remove": {
        "template": "§.pop()",
        "params": ["ego"]
      },
      "remota": {
        "template": "§0[..§0.len().saturating_sub(1)].to_vec()",
        "params": ["ego"]
      },
      "decapita": {
        "template": "§.remove(0)",
        "params": ["ego"]
      },
      "decapitata": {
        "template": "§[1..].to_vec()",
        "params": ["ego"]
      },
      "purga": {
        "method": "clear"
      },
      "primus": {
        "template": "§.first()",
        "params": ["ego"]
      },
      "ultimus": {
        "template": "§.last()",
        "params": ["ego"]
      },
      "accipe": {
        "template": "§.get(§)",
        "params": ["ego","idx"]
      },
      "longitudo": {
        "template": "§.len()",
        "params": ["ego"]
      },
      "vacua": {
        "template": "§.is_empty()",
        "params": ["ego"]
      },
      "continet": {
        "template": "§0.contains(&§1)",
        "params": ["ego","elem"]
      },
      "indiceDe": {
        "template": "§0.iter().position(|e| e == &§1)",
        "params": ["ego","elem"]
      },
      "inveni": {
        "template": "§0.iter().find(§1)",
        "params": ["ego","pred"]
      },
      "inveniIndicem": {
        "template": "§0.iter().position(§1)",
        "params": ["ego","pred"]
      },
      "omnes": {
        "template": "§.iter().all(§)",
        "params": ["ego","pred"]
      },
      "aliquis": {
        "template": "§.iter().any(§)",
        "params": ["ego","pred"]
      },
      "filtrata": {
        "template": "§.iter().filter(§).cloned().collect::<Vec<_>>()",
        "params": ["ego","pred"]
      },
      "mappata": {
        "template": "§.iter().map(§).collect::<Vec<_>>()",
        "params": ["ego","fn"]
      },
      "explanata": {
        "template": "§.iter().flat_map(§).collect::<Vec<_>>()",
        "params": ["ego","fn"]
      },
      "plana": {
        "template": "§.iter().flatten().cloned().collect::<Vec<_>>()",
        "params": ["ego"]
      },
      "inversa": {
        "template": "§.iter().rev().cloned().collect::<Vec<_>>()",
        "params": ["ego"]
      },
      "ordinata": {
        "template": "faber::lista_ordinata(&§)",
        "params": ["ego"]
      },
      "sectio": {
        "template": "§[§..§].to_vec()",
        "params": ["ego","start","end"]
      },
      "prima": {
        "template": "§.iter().take(§).cloned().collect::<Vec<_>>()",
        "params": ["ego","n"]
      },
      "ultima": {
        "template": "faber::lista_ultima(&§0, §1)",
        "params": ["ego","n"]
      },
      "omissa": {
        "template": "§.iter().skip(§).cloned().collect::<Vec<_>>()",
        "params": ["ego","n"]
      },
      "reducta": {
        "template": "§.iter().fold(§, §)",
        "params": ["ego","fn","init"]
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
        "template": "§.iter().for_each(§)",
        "params": ["ego","fn"]
      },
      "coniunge": {
        "template": "§.join(§)",
        "params": ["ego","sep"]
      },
      "summa": {
        "template": "§.iter().sum::<i64>()",
        "params": ["ego"]
      },
      "medium": {
        "template": "(§0.iter().sum::<i64>() as f64 / §0.len() as f64)",
        "params": ["ego"]
      },
      "minimus": {
        "template": "§.iter().min()",
        "params": ["ego"]
      },
      "maximus": {
        "template": "§.iter().max()",
        "params": ["ego"]
      },
      "minimusPer": {
        "template": "§.iter().min_by_key(§)",
        "params": ["ego","fn"]
      },
      "maximusPer": {
        "template": "§.iter().max_by_key(§)",
        "params": ["ego","fn"]
      },
      "numera": {
        "template": "§.iter().filter(§).count()",
        "params": ["ego","pred"]
      },
      "congrega": {
        "template": "faber::lista_congrega(&§0, §1)",
        "params": ["ego","fn"]
      },
      "unica": {
        "template": "faber::lista_unica(&§)",
        "params": ["ego"]
      },
      "fragmenta": {
        "template": "§.chunks(§).map(|c| c.to_vec()).collect::<Vec<_>>()",
        "params": ["ego","n"]
      },
      "partire": {
        "template": "faber::lista_partire(&§0, §1)",
        "params": ["ego","pred"]
      },
    }
    , "innatum": "Vec"
  }
};

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
