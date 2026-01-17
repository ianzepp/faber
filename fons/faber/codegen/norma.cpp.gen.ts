/**
 * Generated CPP norma registry.
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
        "template": "std::floor(§)",
        "params": ["x"]
      },
      "tectum": {
        "template": "std::ceil(§)",
        "params": ["x"]
      },
      "rotundum": {
        "template": "std::round(§)",
        "params": ["x"]
      },
      "truncatum": {
        "template": "std::trunc(§)",
        "params": ["x"]
      },
      "radix": {
        "template": "std::sqrt(§)",
        "params": ["x"]
      },
      "potentia": {
        "template": "std::pow(§, §)",
        "params": ["base","exp"]
      },
      "logarithmus": {
        "template": "std::log(§)",
        "params": ["x"]
      },
      "logarithmus10": {
        "template": "std::log10(§)",
        "params": ["x"]
      },
      "exponens": {
        "template": "std::exp(§)",
        "params": ["x"]
      },
      "sinus": {
        "template": "std::sin(§)",
        "params": ["x"]
      },
      "cosinus": {
        "template": "std::cos(§)",
        "params": ["x"]
      },
      "tangens": {
        "template": "std::tan(§)",
        "params": ["x"]
      },
      "absolutum": {
        "template": "std::abs(§)",
        "params": ["x"]
      },
      "signum": {
        "template": "((§0 > 0) - (§0 < 0))",
        "params": ["x"]
      },
      "minimus": {
        "template": "std::min(§, §)",
        "params": ["a","b"]
      },
      "maximus": {
        "template": "std::max(§, §)",
        "params": ["a","b"]
      },
      "constringens": {
        "template": "std::clamp(§, §, §)",
        "params": ["x","lo","hi"]
      },
      "PI": {
        "template": "M_PI",
        "params": []
      },
      "E": {
        "template": "M_E",
        "params": []
      },
      "TAU": {
        "template": "(M_PI * 2)",
        "params": []
      }
    }
    , "innatum": "std::cmath"
  },
  "tabula": {
    "methods": {
      "pone": {
        "template": "§0.insert_or_assign(§1, §2)",
        "params": ["ego","k","v"]
      },
      "accipe": {
        "template": "§0.at(§1)",
        "params": ["ego","k"]
      },
      "accipeAut": {
        "template": "(§0.contains(§1) ? §0.at(§1) : §2)",
        "params": ["ego","k","def"]
      },
      "habet": {
        "template": "§0.contains(§1)",
        "params": ["ego","k"]
      },
      "dele": {
        "template": "§0.erase(§1)",
        "params": ["ego","k"]
      },
      "longitudo": {
        "template": "§.size()",
        "params": ["ego"]
      },
      "vacua": {
        "template": "§.empty()",
        "params": ["ego"]
      },
      "purga": {
        "method": "clear"
      },
      "claves": {
        "template": "§ | std::views::keys",
        "params": ["ego"]
      },
      "valores": {
        "template": "§ | std::views::values",
        "params": ["ego"]
      },
      "paria": {
        "template": "§",
        "params": ["ego"]
      },
      "selecta": {
        "template": "faber::tabula_selecta(§0, §1)",
        "params": ["ego","claves"]
      },
      "omissa": {
        "template": "faber::tabula_omissa(§0, §1)",
        "params": ["ego","claves"]
      },
      "conflata": {
        "template": "faber::tabula_conflata(§0, §1)",
        "params": ["ego","alia"]
      },
      "inversa": {
        "template": "faber::tabula_inversa(§)",
        "params": ["ego"]
      },
      "inLista": {
        "template": "faber::tabula_in_lista(§)",
        "params": ["ego"]
      },
    }
    , "innatum": "std::unordered_map"
  },
  "numerus": {
    "methods": {
      "absolutum": {
        "template": "std::abs(§)",
        "params": ["ego"]
      },
      "signum": {
        "template": "((§0 > 0) - (§0 < 0))",
        "params": ["ego"]
      },
      "minimus": {
        "template": "std::min(§, §)",
        "params": ["ego","other"]
      },
      "maximus": {
        "template": "std::max(§, §)",
        "params": ["ego","other"]
      }
    }
    , "innatum": "int64_t"
  },
  "fractus": {
    "methods": {
      "absolutum": {
        "template": "std::abs(§)",
        "params": ["ego"]
      },
      "signum": {
        "template": "std::copysign(1.0, §)",
        "params": ["ego"]
      },
      "minimus": {
        "template": "std::min(§, §)",
        "params": ["ego","other"]
      },
      "maximus": {
        "template": "std::max(§, §)",
        "params": ["ego","other"]
      },
      "rotunda": {
        "template": "static_cast<int64_t>(std::round(§))",
        "params": ["ego"]
      },
      "pavimentum": {
        "template": "static_cast<int64_t>(std::floor(§))",
        "params": ["ego"]
      },
      "tectum": {
        "template": "static_cast<int64_t>(std::ceil(§))",
        "params": ["ego"]
      },
      "trunca": {
        "template": "static_cast<int64_t>(std::trunc(§))",
        "params": ["ego"]
      }
    }
    , "innatum": "double"
  },
  "textus": {
    "methods": {
      "longitudo": {
        "template": "§.length()",
        "params": ["ego"]
      },
      "sectio": {
        "template": "§.substr(§, § - §)",
        "params": ["ego","start","end"]
      },
      "continet": {
        "template": "(§.find(§) != std::string::npos)",
        "params": ["ego","sub"]
      },
      "initium": {
        "template": "(§.rfind(§, 0) == 0)",
        "params": ["ego","prefix"]
      },
      "finis": {
        "template": "(§.size() >= §.size() && §.compare(§.size() - §.size(), §.size(), §) == 0)",
        "params": ["ego","suffix"]
      },
      "maiuscula": {
        "template": "[&]{ auto s = §; std::transform(s.begin(), s.end(), s.begin(), ::toupper); return s; }()",
        "params": ["ego"]
      },
      "minuscula": {
        "template": "[&]{ auto s = §; std::transform(s.begin(), s.end(), s.begin(), ::tolower); return s; }()",
        "params": ["ego"]
      },
      "recide": {
        "template": "[&]{ auto s = §; s.erase(0, s.find_first_not_of(\\\" \\\\t\\\\n\\\\r\\\")); s.erase(s.find_last_not_of(\\\" \\\\t\\\\n\\\\r\\\") + 1); return s; }()",
        "params": ["ego"]
      },
      "divide": {
        "template": "@compileError(\\\"No single-expression split in C++ - use manual loop\\\")",
        "params": ["ego","sep"]
      },
      "muta": {
        "template": "std::regex_replace(§, std::regex(§), §)",
        "params": ["ego","old","new"]
      }
    }
    , "innatum": "std::string"
  },
  "copia": {
    "methods": {
      "adde": {
        "method": "insert"
      },
      "habet": {
        "template": "§0.contains(§1)",
        "params": ["ego","elem"]
      },
      "dele": {
        "template": "§0.erase(§1)",
        "params": ["ego","elem"]
      },
      "longitudo": {
        "template": "§.size()",
        "params": ["ego"]
      },
      "vacua": {
        "template": "§.empty()",
        "params": ["ego"]
      },
      "purga": {
        "method": "clear"
      },
      "valores": {
        "template": "§",
        "params": ["ego"]
      },
      "perambula": {
        "template": "std::ranges::for_each(§0, §1)",
        "params": ["ego","fn"]
      },
      "unio": {
        "template": "faber::copia_unio(§0, §1)",
        "params": ["ego","alia"]
      },
      "intersectio": {
        "template": "faber::copia_intersectio(§0, §1)",
        "params": ["ego","alia"]
      },
      "differentia": {
        "template": "faber::copia_differentia(§0, §1)",
        "params": ["ego","alia"]
      },
      "symmetrica": {
        "template": "faber::copia_symmetrica(§0, §1)",
        "params": ["ego","alia"]
      },
      "subcopia": {
        "template": "faber::copia_subcopia(§0, §1)",
        "params": ["ego","alia"]
      },
      "supercopia": {
        "template": "faber::copia_supercopia(§0, §1)",
        "params": ["ego","alia"]
      },
      "inLista": {
        "template": "faber::copia_in_lista(§)",
        "params": ["ego"]
      }
    }
    , "innatum": "std::unordered_set"
  },
  "lista": {
    "methods": {
      "adde": {
        "method": "push_back"
      },
      "addita": {
        "template": "faber::lista_addita(§0, §1)",
        "params": ["ego","elem"]
      },
      "praepone": {
        "template": "§.insert(§.begin(), §)",
        "params": ["ego","elem"]
      },
      "praeposita": {
        "template": "faber::lista_praeposita(§0, §1)",
        "params": ["ego","elem"]
      },
      "remove": {
        "template": "faber::lista_remove(§0)",
        "params": ["ego"]
      },
      "remota": {
        "template": "std::vector(§0.begin(), §0.end() - 1)",
        "params": ["ego"]
      },
      "decapita": {
        "template": "faber::lista_decapita(§0)",
        "params": ["ego"]
      },
      "decapitata": {
        "template": "std::vector(§0.begin() + 1, §0.end())",
        "params": ["ego"]
      },
      "purga": {
        "method": "clear"
      },
      "primus": {
        "template": "§.front()",
        "params": ["ego"]
      },
      "ultimus": {
        "template": "§.back()",
        "params": ["ego"]
      },
      "accipe": {
        "template": "§.at(§)",
        "params": ["ego","idx"]
      },
      "longitudo": {
        "template": "§.size()",
        "params": ["ego"]
      },
      "vacua": {
        "template": "§.empty()",
        "params": ["ego"]
      },
      "continet": {
        "template": "(std::find(§0.begin(), §0.end(), §1) != §0.end())",
        "params": ["ego","elem"]
      },
      "indiceDe": {
        "template": "faber::lista_indice_de(§0, §1)",
        "params": ["ego","elem"]
      },
      "inveni": {
        "template": "*std::find_if(§0.begin(), §0.end(), §1)",
        "params": ["ego","pred"]
      },
      "inveniIndicem": {
        "template": "faber::lista_inveni_indicem(§0, §1)",
        "params": ["ego","pred"]
      },
      "omnes": {
        "template": "std::ranges::all_of(§, §)",
        "params": ["ego","pred"]
      },
      "aliquis": {
        "template": "std::ranges::any_of(§, §)",
        "params": ["ego","pred"]
      },
      "filtrata": {
        "template": "(§ | std::views::filter(§) | std::ranges::to<std::vector>())",
        "params": ["ego","pred"]
      },
      "mappata": {
        "template": "(§ | std::views::transform(§) | std::ranges::to<std::vector>())",
        "params": ["ego","fn"]
      },
      "explanata": {
        "template": "(§ | std::views::transform(§) | std::views::join | std::ranges::to<std::vector>())",
        "params": ["ego","fn"]
      },
      "plana": {
        "template": "(§ | std::views::join | std::ranges::to<std::vector>())",
        "params": ["ego"]
      },
      "inversa": {
        "template": "faber::lista_inversa(§)",
        "params": ["ego"]
      },
      "ordinata": {
        "template": "faber::lista_ordinata(§)",
        "params": ["ego"]
      },
      "sectio": {
        "template": "std::vector(§0.begin() + §1, §0.begin() + §2)",
        "params": ["ego","start","end"]
      },
      "prima": {
        "template": "(§ | std::views::take(§) | std::ranges::to<std::vector>())",
        "params": ["ego","n"]
      },
      "ultima": {
        "template": "faber::lista_ultima(§0, §1)",
        "params": ["ego","n"]
      },
      "omissa": {
        "template": "(§ | std::views::drop(§) | std::ranges::to<std::vector>())",
        "params": ["ego","n"]
      },
      "reducta": {
        "template": "std::ranges::fold_left(§, §, §)",
        "params": ["ego","fn","init"]
      },
      "filtra": {
        "template": "§0.erase(std::remove_if(§0.begin(), §0.end(), [&](auto& x) { return !(§1)(x); }), §0.end())",
        "params": ["ego","pred"]
      },
      "ordina": {
        "template": "std::ranges::sort(§)",
        "params": ["ego"]
      },
      "inverte": {
        "template": "std::ranges::reverse(§)",
        "params": ["ego"]
      },
      "perambula": {
        "template": "std::ranges::for_each(§, §)",
        "params": ["ego","fn"]
      },
      "coniunge": {
        "template": "faber::lista_coniunge(§0, §1)",
        "params": ["ego","sep"]
      },
      "summa": {
        "template": "std::accumulate(§0.begin(), §0.end(), 0)",
        "params": ["ego"]
      },
      "medium": {
        "template": "(std::accumulate(§0.begin(), §0.end(), 0.0) / §0.size())",
        "params": ["ego"]
      },
      "minimus": {
        "template": "*std::ranges::min_element(§)",
        "params": ["ego"]
      },
      "maximus": {
        "template": "*std::ranges::max_element(§)",
        "params": ["ego"]
      },
      "minimusPer": {
        "template": "*std::ranges::min_element(§, [&](auto& a, auto& b) { return (§)(a) < (§)(b); })",
        "params": ["ego","fn"]
      },
      "maximusPer": {
        "template": "*std::ranges::max_element(§, [&](auto& a, auto& b) { return (§)(a) < (§)(b); })",
        "params": ["ego","fn"]
      },
      "numera": {
        "template": "std::ranges::count_if(§, §)",
        "params": ["ego","pred"]
      },
      "congrega": {
        "template": "faber::lista_congrega(§0, §1)",
        "params": ["ego","fn"]
      },
      "unica": {
        "template": "faber::lista_unica(§)",
        "params": ["ego"]
      },
      "planaOmnia": {
        "template": "(§ | std::views::join | std::ranges::to<std::vector>())",
        "params": ["ego"]
      },
      "fragmenta": {
        "template": "faber::lista_fragmenta(§0, §1)",
        "params": ["ego","n"]
      },
      "densa": {
        "template": "(§ | std::views::filter([](auto& x) { return static_cast<bool>(x); }) | std::ranges::to<std::vector>())",
        "params": ["ego"]
      },
      "partire": {
        "template": "faber::lista_partire(§0, §1)",
        "params": ["ego","pred"]
      },
      "miscita": {
        "template": "faber::lista_miscita(§)",
        "params": ["ego"]
      },
      "specimen": {
        "template": "faber::lista_specimen(§0)",
        "params": ["ego"]
      },
      "specimina": {
        "template": "faber::lista_specimina(§0, §1)",
        "params": ["ego","n"]
      }
    }
    , "innatum": "std::vector"
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
