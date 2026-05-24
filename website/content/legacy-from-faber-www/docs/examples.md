---
title: Examples
description: Faber code examples organized by feature
section: docs
order: 2
---

# Examples

Faber code examples from the `exempla/` directory, organized by feature category.

## ab

### ab

```faber
# Ab expression - collection filtering DSL
# 'ab' provides declarative filtering with optional transforms

incipit {
    # Sample data - users with boolean properties
    fixum users = [
        { nomen: "Marcus", activus: verum, aetas: 25 },
        { nomen: "Julia", activus: falsum, aetas: 30 },
        { nomen: "Gaius", activus: verum, aetas: 17 }
    ]

    fixum items = [
        { valor: 10, visibilis: verum },
        { valor: 20, visibilis: falsum },
        { valor: 30, visibilis: verum },
        { valor: 40, visibilis: verum }
    ]

    # Boolean property shorthand - filter where property is true
    fixum active = ab users activus
    scribe active

    # Negated filter - filter where property is false
    fixum inactive = ab users non activus
    scribe inactive

    # Filter with prima transform (first N elements)
    fixum top2 = ab items visibilis, prima 2
    scribe top2

    # Filter with ultima transform (last N elements)
    fixum last2 = ab items visibilis, ultima 2
    scribe last2

    # Filter with summa transform (sum of results)
    fixum prices = [
        { pretium: 100, validum: verum },
        { pretium: 200, validum: verum },
        { pretium: 50, validum: falsum }
    ]
    fixum validPrices = ab prices validum
    scribe validPrices

    # Multiple transforms chained
    fixum result = ab items visibilis, prima 3, ultima 2
    scribe result

    # Without filter - just apply transforms to collection
    fixum nums = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
    fixum firstFive = ab nums, prima 5
    scribe firstFive

    # Chain transforms without filter
    fixum sumFirst = ab nums, prima 5, summa
    scribe sumFirst

    # Complex source - member expression
    fixum data = { users: users }
    fixum dataActive = ab data.users activus
    scribe dataActive
}
```

## abstractus

### abstractus

```faber
# TODO
incipit {}
```

## ad

### ad

```faber
# Ad statement - endpoint dispatch
# NOTE: Currently parsed but codegen not yet implemented
# See consilia/futura/ad.md for future implementation details

incipit {
    # Basic endpoint dispatch with sync handler
    ad "fasciculus:lege" ("hello.txt") -> textus pro content {
        scribe scriptum("File contents: §", content)
    }

    # Async endpoint dispatch
    @ futura
    ad "caelum:pete" ("https://api.example.com/data") -> objectum pro response {
        fixum data = cede response.json()
        scribe data
    }

    # Multiple arguments
    ad "processus:executa" ("ls", "-la") -> textus pro output {
        scribe output
    }

    # Complex dispatch with error handling
    #ad "rete:connecta" ("127.0.0.1", 8080) -> rete.Socket pro socket {
    #    scribe "Connected successfully"
    #    socket.claude()
    #} cape err {
    #    scribe scriptum("Connection failed: §", err.nuntius)
    #}

    # Endpoint dispatch without handler (fire-and-forget)
    ad "notificatio:mitte" ("User logged in", userId)
}
```

## adfirma

### adfirma

```faber
# Basic adfirma (assert) statements
#
# adfirma <condition>
# adfirma <condition>, "error message"

incipit {
    fixum x = 10

    # Simple assertion without message
    adfirma x > 0

    # Assertion with custom error message
    adfirma x == 10, "x must equal 10"

    # Multiple assertions
    fixum name = "Marcus"
    adfirma name == "Marcus"
    adfirma name != "", "name must not be empty"

    # Boolean assertions
    fixum active = verum
    adfirma active
    adfirma active == verum, "must be active"
}
```

### in-functione

```faber
# Assertions in functions for preconditions and postconditions
#
# adfirma <condition>
# adfirma <condition>, "error message"

# Precondition: validate input at function start
functio divide(numerus a, numerus b) -> numerus {
    adfirma b != 0, "divisor must not be zero"
    redde a / b
}

# Multiple preconditions
functio calculateAge(numerus birthYear, numerus currentYear) -> numerus {
    adfirma birthYear > 0, "birth year must be positive"
    adfirma currentYear >= birthYear, "current year must be >= birth year"
    redde currentYear - birthYear
}

# Postcondition: validate result before returning
functio absoluteValue(numerus n) -> numerus {
    varia result = n
    si n < 0 {
        result = -n
    }
    adfirma result >= 0, "result must be non-negative"
    redde result
}

incipit {
    fixum quotient = divide(20, 4)
    adfirma quotient == 5

    fixum age = calculateAge(1990, 2024)
    adfirma age == 34

    fixum abs = absoluteValue(-42)
    adfirma abs == 42
}
```

## annotatio

### annotatio

```faber
# Stdlib Annotations Example
#
# Demonstrates the four annotation types used to define standard library
# methods with cross-target codegen support.
#
# These annotations are primarily for stdlib authors. User code typically
# does not need them, but library authors may find them useful for providing
# optimized implementations across multiple targets.
#
# NOTE: This file demonstrates annotation syntax only. The functions are
# declared with @ externa since they have no bodies - actual implementations
# come from the target language runtime or @ subsidia files.

# =============================================================================
# @ innatum - Native Type Mapping
# =============================================================================
# Maps a Faber genus to native types in each target language.

@ innatum ts "Array", py "list", zig "Lista", rs "Vec", cpp "std::vector"
genus ExemplarLista { }

@ innatum ts "Map", py "dict", zig "Tabula", rs "HashMap"
genus ExemplarTabula { }

# =============================================================================
# @ subsidia - External Implementation Files
# =============================================================================
# Specifies implementation files for targets where inline codegen is complex.

@ innatum ts "Array", zig "Lista"
@ subsidia zig "subsidia/zig/lista.zig"
genus ExemplarListaExterna { }

# =============================================================================
# @ radix - Morphology Declaration
# =============================================================================
# Declares the verb stem and valid conjugation forms.

# Full morphology: imperative (mutates) + participle (returns new)
@ radix filtr, imperativus, perfectum
@ externa
functio filtraExemplar()

# Imperative only (mutate-only operation)
@ radix purg, imperativus
@ externa
functio purgaExemplar()

# Participle only (always returns new)
@ radix mapp, perfectum
@ externa
functio mappataExemplar()

# =============================================================================
# @ verte - Codegen Transformation
# =============================================================================
# Defines how method calls compile to target code.

# Simple rename: method name maps directly
@ radix add, imperativus
@ verte ts "push"
@ verte py "append"
@ verte rs "push"
@ verte cpp "push_back"
@ verte zig (ego, elem, alloc) -> "§.appende(§, §)"
@ externa
functio addeExemplar()

# Template form: § placeholders filled positionally
@ verte ts (ego, elem) -> "[...§, §]"
@ verte py (ego, elem) -> "[*§, §]"
@ verte zig (ego, elem, alloc) -> "§.addita(§, §)"
@ externa
functio additaExemplar()

# Properties (no morphology - read-only accessors)
@ verte ts (ego) -> "§.length"
@ verte py (ego) -> "len(§)"
@ verte zig (ego) -> "§.longitudo()"
@ externa
functio longitudoExemplar() -> numerus

# =============================================================================
# Combined Example: Full Method Definition
# =============================================================================

@ innatum ts "Array", py "list", zig "Lista"
@ subsidia zig "subsidia/zig/lista.zig"
genus ExemplarListaCompleta { }

@ radix ordin, imperativus, perfectum
@ verte ts "sort"
@ verte py "sort"
@ verte zig (ego) -> "§.ordina()"
@ externa
functio ordinaExemplar()

@ verte ts (ego) -> "[...§].sort()"
@ verte py (ego) -> "sorted(§)"
@ verte zig (ego, alloc) -> "§.ordinata(§)"
@ externa
functio ordinataExemplar()
```

### grammatica-nova

```faber
# New Annotation Grammar Example
#
# Demonstrates the unified keyword-dispatch grammar for annotations:
#   @ <keyword> [args...]  - declaration metadata
#   § <keyword> [args...]  - file/module structure
#
# See: https://github.com/ianzepp/faber/issues/293

# =============================================================================
# importa - Module Imports (NEW SYNTAX per issue #304)
# =============================================================================
# Syntax: importa ex "<path>" privata|publica <name> [ut <alias>]
# Syntax: importa ex "<path>" privata|publica * ut <alias>  (wildcard)

importa ex "./types" privata Numerus
importa ex "./types" privata Textus
importa ex "./utils" privata auxilium ut aux
importa ex "./utils" privata formata
importa ex "./constants" privata * ut constants
importa ex "./config" privata * ut cfg

# =============================================================================
# § sectio - File Sections
# =============================================================================

§ sectio "types"

# =============================================================================
# @ publica / @ privata - Visibility
# =============================================================================

@ publica
genus PublicaRes {}

@ privata
genus PrivataRes {}

# =============================================================================
# @ futura - Async Functions
# =============================================================================

@ publica
@ futura
functio fetchData(textus url) -> textus {
    redde ""
}

# =============================================================================
# @ cli - CLI Program Declaration
# =============================================================================
# Keyword: cli
# Grammar: @ cli <string>

@ cli "exemplar"
@ versio "1.0.0"
@ descriptio "An example CLI program"
incipit argumenta args {}

# =============================================================================
# @ optio - CLI Options
# =============================================================================
# Keyword: optio
# Grammar: @ optio [type] <binding> [brevis <s>] [longum <s>] [descriptio <s>]

@ optio bivalens verbose brevis "v" longum "verbose" descriptio "Enable verbose output"
@ optio textus output brevis "o" longum "output" descriptio "Output file path"
@ optio numerus count longum "count" descriptio "Number of iterations"

# =============================================================================
# @ operandus - CLI Positional Arguments
# =============================================================================
# Keyword: operandus
# Grammar: @ operandus [ceteri] <type> <binding> [descriptio <s>]

@ operandus textus input descriptio "Input file"
@ operandus ceteri textus files descriptio "Additional files"

# =============================================================================
# @ imperium / @ alias - CLI Subcommands
# =============================================================================
# Keyword: imperium
# Grammar: @ imperium <string>
#
# Keyword: alias
# Grammar: @ alias <string>

@ imperium "build"
@ alias "b"
@ descriptio "Build the project"
@ optio bivalens release longum "release" descriptio "Build in release mode"
functio build(si bivalens release) -> vacuum {}

@ imperium "test"
@ alias "t"
@ descriptio "Run tests"
@ optio textus filter brevis "f" longum "filter" descriptio "Filter tests by name"
functio test(si textus filter) -> vacuum {}

# =============================================================================
# @ nomen - Named Blocks (for config)
# =============================================================================
# Keyword: nomen
# Grammar: @ nomen <string>

@ indentum 4
@ tabulae falsum
@ latitudo 100
@ nomen "forma"
fac {}

@ ordinatio "alphabetica"
@ separaGroups verum
@ nomen "importa"
fac {}

@ bracchiae "nova-linea"
@ methodiSeparatio 1
@ nomen "genus"
fac {}

# =============================================================================
# @ innatum / @ verte / @ radix / @ subsidia - Stdlib Annotations
# =============================================================================
# These follow the same keyword-dispatch pattern

@ innatum ts "Map", py "dict", rs "HashMap"
genus Tabula {}

@ radix sort, imperativus, perfectum
@ verte ts "sort"
@ verte py "sort"
@ externa
functio sorta()

# =============================================================================
# @ externa - External Declaration
# =============================================================================

@ externa
functio nativeFunction(numerus x) -> numerus
```

## ante

### ante

```faber
# TODO
incipit {}
```

## assignatio

### assignatio

```faber
# Assignment expressions
# Simple and compound assignment operators

incipit {
    varia numerus x = 10

    # Simple assignment
    x = 20
    scribe x  # 20

    # Compound assignments
    x += 5
    scribe x  # 25

    x -= 10
    scribe x  # 15

    x *= 2
    scribe x  # 30

    x /= 3
    scribe x  # 10

    # String concatenation
    varia textus s = "hello"
    s += " world"
    scribe s  # hello world
}
```

## aut

### aut

```faber
# TODO
incipit {}
```

## binarius

### binarius

```faber
# ============================================================================
# Binary Expressions
# ============================================================================
#
# Binary expressions combine two operands with an operator. This file covers
# arithmetic, comparison, logical, bitwise, and nullish coalescing operations.
#
# ----------------------------------------------------------------------------
# PRECEDENCE (lowest to highest)
# ----------------------------------------------------------------------------
#
# The precedence chain determines parsing order. Lower precedence binds last:
#
#   assignment < ternary < or < and < equality < comparison
#   < bitwiseOr < bitwiseXor < bitwiseAnd < shift < range
#   < additive < multiplicative < unary < call < primary
#
# ----------------------------------------------------------------------------
# GRAMMAR: Assignment
# ----------------------------------------------------------------------------
#
#   assignment := ternary (('=' | '+=' | '-=' | '*=' | '/=' | '&=' | '|=') assignment)?
#
# Assignment is right-associative: a = b = c parses as a = (b = c).
# Compound assignment operators combine operation with assignment.
#
# ----------------------------------------------------------------------------
# GRAMMAR: Ternary
# ----------------------------------------------------------------------------
#
#   ternary := or (('?' expression ':' | 'sic' expression 'secus') ternary)?
#
# Two styles supported (do not mix within one expression):
#   - Symbolic: condition ? consequent : alternate
#   - Latin: condition sic consequent secus alternate
#
# Latin 'sic' means "thus/so", 'secus' means "otherwise".
#
# ----------------------------------------------------------------------------
# GRAMMAR: Logical Or
# ----------------------------------------------------------------------------
#
#   or := and (('||' | 'aut') and)* | and ('vel' and)*
#
# PREFER: 'aut' over '||' for logical or.
# PREFER: 'vel' for nullish coalescing (replaces JavaScript's '??').
#
# 'aut' and 'vel' cannot be mixed without parentheses (same restriction as
# JavaScript's || and ?? to prevent precedence confusion).
#
# Latin: 'aut' = "or" (exclusive sense), 'vel' = "or" (either/or, nullable).
#
# ----------------------------------------------------------------------------
# GRAMMAR: Logical And
# ----------------------------------------------------------------------------
#
#   and := equality ('&&' equality | 'et' equality)*
#
# PREFER: 'et' over '&&' for logical and.
#
# Latin: 'et' = "and".
#
# ----------------------------------------------------------------------------
# GRAMMAR: Equality
# ----------------------------------------------------------------------------
#
#   equality := comparison (('==' | '!=' | '===' | '!==' | 'est' | 'non' 'est') comparison)*
#
# Use '===' and '!==' for value equality (strict).
# Use 'est' for type checking (instanceof/typeof).
# Use 'non est' for negative type checking.
# Use 'nihil x' or 'nonnihil x' as unary prefix for null checks.
#
# Latin: 'est' = "is" (type), 'non' = "not".
#
# ----------------------------------------------------------------------------
# GRAMMAR: Comparison
# ----------------------------------------------------------------------------
#
#   comparison := bitwiseOr (('<' | '>' | '<=' | '>=') bitwiseOr)*
#
# Standard comparison operators. Chains left-to-right: a < b < c is valid.
#
# ----------------------------------------------------------------------------
# GRAMMAR: Bitwise Operators
# ----------------------------------------------------------------------------
#
#   bitwiseOr  := bitwiseXor ('|' bitwiseXor)*
#   bitwiseXor := bitwiseAnd ('^' bitwiseAnd)*
#   bitwiseAnd := range ('&' range)*
#
# Bitwise precedence is ABOVE comparison (unlike C), so:
#   flags & MASK == 0  parses as  (flags & MASK) == 0
#
# This matches programmer intent and avoids subtle bugs.
#
# Bit shift uses postfix keywords (see parseQuaExpression level):
#   x sinistratum n  → x << n (shift left)
#   x dextratum n    → x >> n (shift right)
#
# WHY: Latin keywords avoid ambiguity with nested generics like lista<lista<T>>.
#
# ----------------------------------------------------------------------------
# GRAMMAR: Range
# ----------------------------------------------------------------------------
#
#   range := additive (('..' | 'ante' | 'usque') additive ('per' additive)?)?
#
# Range operators for numeric iteration:
#   - '..' and 'ante': exclusive end (0..10 yields 0-9)
#   - 'usque': inclusive end (0 usque 10 yields 0-10)
#   - 'per': step value (0..10 per 2 yields 0, 2, 4, 6, 8)
#
# Latin: 'ante' = "before", 'usque' = "up to", 'per' = "by/through".
#
# ----------------------------------------------------------------------------
# GRAMMAR: Additive
# ----------------------------------------------------------------------------
#
#   additive := multiplicative (('+' | '-') multiplicative)*
#
# Standard addition and subtraction.
#
# ----------------------------------------------------------------------------
# GRAMMAR: Multiplicative
# ----------------------------------------------------------------------------
#
#   multiplicative := unary (('*' | '/' | '%') unary)*
#
# Standard multiplication, division, and modulo.
#
# ============================================================================
# LLM GUIDANCE
# ============================================================================
#
# ALWAYS prefer Latin keywords over symbols:
#   - 'et' over '&&'
#   - 'aut' over '||'
#   - 'vel' over '??' (nullish coalescing)
#   - 'non' over '!' (see unary.fab)
#   - 'sic...secus' over '?:' (ternary, when clarity helps)
#
# NEVER use JavaScript/TypeScript patterns:
#   - '??' does not exist — use 'vel'
#   - '&&' and '||' work but 'et' and 'aut' are preferred
#
# ============================================================================
# EXAMPLES
# ============================================================================

incipit {
    # --------------------------------------------------------------------------
    # Arithmetic operators: + - * / %
    # --------------------------------------------------------------------------

    fixum sum = 10 + 5
    scribe sum

    fixum diff = 10 - 5
    scribe diff

    fixum prod = 10 * 5
    scribe prod

    fixum quot = 10 / 5
    scribe quot

    fixum rem = 10 % 3
    scribe rem

    # --------------------------------------------------------------------------
    # Compound assignment: += -= *= /= &= |=
    # --------------------------------------------------------------------------

    varia counter = 0
    counter += 10
    counter -= 3
    counter *= 2
    scribe counter

    # --------------------------------------------------------------------------
    # Comparison operators: < > <= >= == != === !==
    # --------------------------------------------------------------------------

    fixum isEqual = 10 == 10
    scribe isEqual

    fixum isNotEqual = 10 != 5
    scribe isNotEqual

    fixum isStrictEqual = 10 === 10
    scribe isStrictEqual

    fixum isLess = 5 < 10
    scribe isLess

    fixum isGreater = 10 > 5
    scribe isGreater

    fixum isLessOrEqual = 5 <= 5
    scribe isLessOrEqual

    fixum isGreaterOrEqual = 10 >= 10
    scribe isGreaterOrEqual

    # Comparison chaining
    fixum inRange = 0 < 5 et 5 < 10
    scribe inRange

    # --------------------------------------------------------------------------
    # Logical operators: et (and), aut (or), non (not)
    # --------------------------------------------------------------------------
    # PREFER: 'et' over '&&', 'aut' over '||'

    fixum both = verum et verum
    scribe both

    fixum either = falsum aut verum
    scribe either

    fixum neither = falsum et falsum
    scribe neither

    # Short-circuit evaluation works as expected
    fixum shortCircuit = falsum et expensiveCheck()
    scribe shortCircuit

    # --------------------------------------------------------------------------
    # Nullish coalescing: vel (replaces ??)
    # --------------------------------------------------------------------------
    # PREFER: 'vel' — the '??' operator does not exist in Faber

    fixum si textus maybeName = nihil
    fixum resolvedName = maybeName vel "default"
    scribe resolvedName

    # Chain multiple fallbacks
    fixum si textus first = nihil
    fixum si textus second = nihil
    fixum textus third = "fallback"
    fixum fallbackResult = first vel second vel third
    scribe fallbackResult

    # --------------------------------------------------------------------------
    # Ternary: condition sic then secus else
    # --------------------------------------------------------------------------
    # Latin form for ternary expressions.

    fixum age = 25
    fixum ageCategory = age >= 18 sic "adult" secus "minor"
    scribe ageCategory

    # Nested ternary (right-associative)
    fixum score = 85
    fixum grade = score >= 90 sic "A" secus score >= 80 sic "B" secus score >= 70 sic "C" secus "F"
    scribe grade

    # --------------------------------------------------------------------------
    # Bitwise operators: & | ^
    # --------------------------------------------------------------------------
    # Note: Bitwise has higher precedence than comparison (unlike C)

    fixum flags = 0b1010
    fixum mask = 0b1100

    fixum bitwiseAnd = flags & mask
    scribe bitwiseAnd

    fixum bitwiseOr = flags | mask
    scribe bitwiseOr

    fixum bitwiseXor = flags ^ mask
    scribe bitwiseXor

    # --------------------------------------------------------------------------
    # Bit shift: sinistratum (<<), dextratum (>>)
    # --------------------------------------------------------------------------
    # Uses postfix keywords to avoid ambiguity with nested generics

    fixum leftShift = 1 sinistratum 4
    scribe leftShift

    fixum rightShift = 16 dextratum 2
    scribe rightShift

    # Precedence: (flags & mask) == 0, not flags & (mask == 0)
    # Using explicit parens to avoid TS literal comparison warning
    fixum checkMask = (flags & mask) == 0
    scribe checkMask
}

# --------------------------------------------------------------------------
# Helper function for short-circuit demonstration
# --------------------------------------------------------------------------

functio expensiveCheck() -> bivalens {
    scribe "This should not print if short-circuited"
    redde verum
}
```

## cede

### cede

```faber
# TODO
incipit {}
```

## ceteri

### ceteri

```faber
# TODO
incipit {}
```

## clausa

### clausa

```faber
# Lambda expressions (closures)
# Pattern: clausura param: expr
# Pattern: clausura param -> <type>: expr

incipit {
    # Simple lambda with single parameter
    fixum double = clausura numerus x: x * 2
    scribe double(5)

    # Multi-parameter lambda
    fixum add = clausura numerus a, numerus b -> numerus: a + b
    scribe add(3, 4)

    # Lambda with array map
    fixum numbers = [1, 2, 3]
    fixum doubled = numbers.map(clausura numerus x: x * 2)
    scribe doubled

    # Lambda with array filter
    fixum evens = numbers.filter(clausura numerus x: x % 2 == 0)
    scribe evens

    # Lambda stored and reused
    fixum isPositive = clausura numerus n -> bivalens: n > 0
    scribe isPositive(10)
    scribe isPositive(-5)
}
```

## cli

### main

```faber
# CLI Framework Example
#
# Demonstrates @ cli, @ imperium, @ imperia annotations.
# Compile: faber compile main.fab -o cli.ts
# Run: bun cli.ts --help

importa ex "./commands/greet" privata * ut greetModule

@ cli "example"
@ versio "1.0.0"
@ descriptio "Example CLI application"
@ imperia "greet" ex greetModule
incipit {}

@ imperium "version"
@ alias "v"
functio version() -> vacuum {
    scribe "Example CLI v1.0.0"
}
```

## commands

### greet

```faber
# Greet Commands Module
#
# Mounted at "greet" by main.fab.
# Commands: greet hello, greet goodbye

@ descriptio "Greeting commands"
incipit {}

@ imperium "hello"
functio hello(textus name) -> vacuum {
    scribe "Hello, §!", name
}

@ imperium "goodbye"
@ alias "bye"
functio goodbye(textus name, si bivalens formal ut f) -> vacuum {
    si f {
        scribe "Farewell, §.", name
    }
    secus {
        scribe "Bye, §!", name
    }
}
```

## conversio

### conversio

```faber
# Conversio - Type conversion operators
#
# Demonstrates numeratum, fractatum, textatum, and bivalentum operators
# for converting between primitive types.

incipit {
    # ==========================================================================
    # numeratum - Convert to integer
    # ==========================================================================

    # Basic string to number
    fixum n1 = "42" numeratum
    scribe n1  # 42

    # With fallback for invalid input
    fixum n2 = "invalid" numeratum vel 0
    scribe n2  # 0

    # With type parameter for sized integers
    fixum n3 = "255" numeratum<i32>
    scribe n3  # 255

    # With radix for hex/octal/binary parsing
    fixum hex = "ff" numeratum<i32, Hex>
    scribe hex  # 255

    fixum bin = "1010" numeratum<i32, Bin>
    scribe bin  # 10

    fixum oct = "755" numeratum<i32, Oct>
    scribe oct  # 493

    # ==========================================================================
    # fractatum - Convert to float
    # ==========================================================================

    fixum f1 = "3.14159" fractatum
    scribe f1  # 3.14159

    fixum f2 = "invalid" fractatum vel 0.0
    scribe f2  # 0.0

    # ==========================================================================
    # textatum - Convert to string (infallible)
    # ==========================================================================

    fixum s1 = 42 textatum
    scribe s1  # "42"

    fixum s2 = 3.14 textatum
    scribe s2  # "3.14"

    fixum s3 = verum textatum
    scribe s3  # "true"

    # ==========================================================================
    # bivalentum - Convert to boolean (truthiness)
    # ==========================================================================

    # Numbers: 0 is falsum, non-zero is verum
    fixum b1 = 0 bivalentum
    fixum b2 = 42 bivalentum
    scribe b1, b2  # false, true

    # Strings: empty is falsum, non-empty is verum
    fixum b3 = "" bivalentum
    fixum b4 = "hello" bivalentum
    scribe b3, b4  # false, true

    # ==========================================================================
    # Chaining conversions
    # ==========================================================================

    # Parse then format back
    fixum roundtrip = "42" numeratum textatum
    scribe roundtrip  # "42"

    # Parse with fallback, then check truthiness
    fixum hasValue = ("123" numeratum vel 0) bivalentum
    scribe hasValue  # true
}
```

## cura

### nidificatus

```faber
# Nested allocator scopes
#
# cura arena <outer> { cura arena <inner> { } }
#
# Allocator scopes can nest. Inner scopes free before outer scopes.

incipit {
    cura arena fixum outer {
        varia textus[] a = ["one"]

        cura arena fixum inner {
            varia textus[] b = ["two"]
            scribe "Inner:", b
        }
        # inner freed here

        a.appende("three")
        scribe "Outer:", a
    }
    # outer freed here
}
```

### cura

```faber
# Arena allocator scope
#
# cura arena <identifier> { <body> }
#
# Arena allocators provide fast allocation with bulk deallocation.
# All memory is freed when the scope exits.
# On GC targets (TS, Python), allocator blocks are ignored.

incipit {
    cura arena fixum mem {
        # All allocations in this block use the arena
        varia textus[] items = ["hello", "world"]
        scribe items
    }
    # Arena freed, all allocations released

    # Page allocator variant
    cura page fixum pageMem {
        scribe "Using page allocator"
    }
}
```

## custodi

### validatio

```faber
# Input validation patterns with custodi
#
# Use custodi to group related precondition checks.
# Each guard should return early or throw on invalid input.

functio processAge(numerus age) -> textus {
    custodi {
        si age < 0 {
            redde "Invalid: negative age"
        }
        si age > 150 {
            redde "Invalid: age too high"
        }
    }

    si age < 18 {
        redde "Minor"
    }
    secus {
        redde "Adult"
    }
}

functio createUser(textus name, textus email, numerus age, curator alloc) -> textus {
    custodi {
        si nihil name aut name == "" {
            redde "Error: name required"
        }
        si nihil email aut email == "" {
            redde "Error: email required"
        }
        si age < 13 {
            redde "Error: must be 13 or older"
        }
        si age > 120 {
            redde "Error: invalid age"
        }
    }

    redde scriptum("User created: §", name)
}

# Guards can throw instead of returning
functio sqrt(numerus n) -> numerus {
    custodi {
        si n < 0 {
            iace "Cannot compute square root of negative number"
        }
    }

    redde n
}

incipit ergo cura arena {
    scribe processAge(-5)
    scribe processAge(200)
    scribe processAge(25)
    scribe processAge(12)

    scribe createUser("Marcus", "marcus@roma.com", 30)
    scribe createUser("", "test@test.com", 25)
    scribe createUser("Julia", "julia@roma.com", 10)

    scribe sqrt(16)
}
```

### custodi

```faber
# Basic custodi (guard clause) statement
#
# custodi { si <condition> { <early-return> } }
#
# Groups early-exit checks at function start to separate
# validation from main logic.

functio divide(numerus a, numerus b) -> numerus {
    custodi {
        si b == 0 {
            redde 0
        }
    }

    redde a / b
}

functio processValue(numerus x) -> numerus {
    custodi {
        si x < 0 {
            redde -1
        }
        si x > 100 {
            redde -1
        }
    }

    # Main logic, clearly separated from guards
    redde x * 2
}

functio clamp(numerus value, numerus min, numerus max) -> numerus {
    custodi {
        si value < min {
            redde min
        }
        si value > max {
            redde max
        }
    }

    redde value
}

incipit {
    scribe divide(10, 2)
    scribe divide(10, 0)

    scribe processValue(50)
    scribe processValue(-10)
    scribe processValue(150)

    scribe clamp(5, 0, 10)
    scribe clamp(-5, 0, 10)
    scribe clamp(15, 0, 10)
}
```

## demum

### demum

```faber
# TODO
incipit {}
```

## destructura

### destructura

```faber
# Array destructuring patterns
#
# fixum [a, b, c] = array         -- destructure into immutable bindings
# varia [x, y, z] = array         -- destructure into mutable bindings
# fixum [first, ceteri rest] = arr -- with rest pattern
# fixum [_, second, _] = arr      -- skip elements with underscore

incipit {
    # Basic array destructuring
    fixum numbers = [1, 2, 3]
    fixum [a, b, c] = numbers

    scribe a
    scribe b
    scribe c

    # Destructure inline array literal
    fixum [first, second, third] = [10, 20, 30]

    scribe first
    scribe second
    scribe third

    # Mutable destructuring with varia
    fixum coords = [100, 200]
    varia [x, y] = coords

    scribe x
    scribe y

    x = x + 50
    y = y + 50

    scribe x
    scribe y

    # Partial destructuring (fewer variables than elements)
    fixum values = [1, 2, 3, 4, 5]
    fixum [one, two] = values

    scribe one
    scribe two

    # Rest pattern with ceteri
    fixum items = [1, 2, 3, 4, 5]
    fixum [head, ceteri tail] = items

    scribe head
    scribe tail

    # Skip elements with underscore
    fixum triple = [10, 20, 30]
    fixum [_, middle, _] = triple

    scribe middle

    # Nested arrays
    fixum matrix = [[1, 2], [3, 4]]
    fixum [row1, row2] = matrix

    scribe row1
    scribe row2
}
```

### objectum

```faber
# Object destructuring patterns
#
# ex obj fixum field1, field2     -- extract fields into immutable bindings
# ex obj varia field1, field2     -- extract into mutable bindings
# ex obj fixum field ut alias     -- extract with alias (rename)
# ex obj fixum field, ceteri rest -- extract with rest pattern

incipit {
    # Basic field extraction
    fixum person = { name: "Marcus", age: 30, city: "Roma" }
    ex person fixum name, age

    scribe name
    scribe age

    # Extract with alias using 'ut'
    fixum user = { name: "Julia", email: "julia@roma.com" }
    ex user fixum name ut userName, email ut userEmail

    scribe userName
    scribe userEmail

    # Mutable destructuring with varia
    fixum data = { count: 100, active: verum }
    ex data varia count, active

    scribe count
    scribe active

    count = 200
    active = falsum

    scribe count
    scribe active

    # Mixed alias and regular fields
    fixum config = { host: "localhost", port: 8080, secure: verum }
    ex config fixum host, port ut serverPort, secure

    scribe host
    scribe serverPort
    scribe secure

    # Rest pattern with ceteri
    fixum fullUser = { id: 1, name: "Gaius", email: "g@roma.com", role: "admin" }
    ex fullUser fixum id, ceteri details

    scribe id
    scribe details

    # Destructure from nested access
    fixum response = { data: { user: { name: "Claudia", verified: verum } } }
    ex response.data.user fixum name ut nestedName, verified

    scribe nestedName
    scribe verified

    # Single field extraction
    fixum settings = { theme: "dark", lang: "la" }
    ex settings fixum theme

    scribe theme
}
```

## discerne

### discerne

```faber
# Pattern matching with discerne (discriminate/distinguish)
#
# discerne <value> {
#     casu <Variant> { <body> }
#     casu <Variant> ut <alias> { <body> }
#     casu <Variant> fixum <bindings> { <body> }
# }

# Define discretio (tagged union) types
discretio Status {
    Active,
    Inactive,
    Pending
}

discretio Event {
    Click { numerus x, numerus y },
    Keypress { textus key },
    Quit
}

# Functions demonstrating discerne
functio describe_status(Status s) -> textus {
    discerne s {
        casu Active { redde "active" }
        casu Inactive { redde "inactive" }
        casu Pending { redde "pending" }
    }
}

functio handle_event(Event e) -> nihil {
    discerne e {
        casu Click fixum x, y {
            scribe "Clicked at:", x, y
        }
        casu Keypress fixum key {
            scribe "Key:", key
        }
        casu Quit {
            scribe "quit"
        }
    }
}

incipit {
    scribe "discerne patterns defined"
}
```

## discretio

### discretio

```faber
# Basic discretio (discriminated union/tagged union)
#
# discretio Name {
#     Variant1 { type field1, type field2 }
#     Variant2 { type field }
#     Variant3
# }

# Discretio with payload variants
discretio Result {
    Success { textus message }
    Failure { textus error }
}

# Discretio with mixed unit and payload variants
discretio Event {
    Click { numerus x, numerus y }
    Keypress { textus key }
    Quit
}

# Discretio with many fields per variant
discretio Shape {
    Rectangle { numerus x, numerus y, numerus width, numerus height }
    Circle { numerus cx, numerus cy, numerus radius }
    Point { numerus x, numerus y }
}

incipit {
    scribe "Discretio types defined"
}
```

## dum

### dum

```faber
# Basic dum (while) loop with counter
#
# dum <condition> { <body> }

incipit {
    varia numerus counter = 0

    dum counter < 5 {
        scribe counter
        counter = counter + 1
    }

    # Countdown example
    varia numerus countdown = 3

    dum countdown > 0 {
        scribe "Countdown:", countdown
        countdown = countdown - 1
    }

    scribe "Done!"
}
```

### conditio-complexa

```faber
# Dum with compound conditions
#
# dum <cond1> et <cond2> { }   -- both must be true
# dum <cond1> aut <cond2> { }  -- either must be true

incipit {
    # Using "et" (and) - loop while running AND attempts < limit
    varia bivalens running = verum
    varia numerus attempts = 0

    dum running et attempts < 5 {
        scribe "Attempt:", attempts
        attempts = attempts + 1

        si attempts >= 3 {
            running = falsum
        }
    }

    # Using "aut" (or) - loop while either condition holds
    varia numerus a = 5
    varia numerus b = 3

    dum a > 0 aut b > 0 {
        scribe "a:", a, "b:", b
        a = a - 1
        b = b - 1
    }
}
```

### in-functione

```faber
# While loops inside functions

functio factorial(numerus n) -> numerus {
    varia numerus result = 1
    varia numerus current = n

    dum current > 1 {
        result = result * current
        current = current - 1
    }

    redde result
}

functio nextPowerOf2(numerus n) -> numerus {
    varia numerus power = 1

    dum power <= n {
        power = power * 2
    }

    redde power
}

incipit {
    scribe "5! =", factorial(5)
    scribe "10! =", factorial(10)

    scribe "Next power of 2 after 100:", nextPowerOf2(100)
    scribe "Next power of 2 after 1000:", nextPowerOf2(1000)
}
```

## ego

### ego

```faber
# TODO
incipit {}
```

## elige

### elige

```faber
# Basic elige (switch) statement
#
# elige <expr> {
#     casu <value> { <body> }
#     casu <value> { <body> }
#     ceterum { <body> }
# }

incipit {
    # String matching
    fixum status = "active"

    elige status {
        casu "pending" {
            scribe "Waiting..."
        }
        casu "active" {
            scribe "Running"
        }
        casu "done" {
            scribe "Completed"
        }
    }

    # Number matching
    fixum code = 200

    elige code {
        casu 200 {
            scribe "OK"
        }
        casu 404 {
            scribe "Not Found"
        }
        casu 500 {
            scribe "Server Error"
        }
    }

    # Multiple statements per case
    fixum mode = "production"

    elige mode {
        casu "development" {
            scribe "Dev mode enabled"
            scribe "Verbose logging on"
        }
        casu "production" {
            scribe "Production mode"
            scribe "Optimizations enabled"
        }
    }
}
```

### reddit

```faber
# Elige with reddit syntax
#
# 'reddit' is syntactic sugar for 'ergo redde' - a one-liner return.
# Use it when each case simply returns a value.
#
# casu <value> reddit <expression>
# ceterum reddit <expression>

# HTTP status code lookup using reddit
functio getStatusText(numerus code) -> textus {
    elige code {
        casu 200 reddit "OK"
        casu 201 reddit "Created"
        casu 204 reddit "No Content"
        casu 400 reddit "Bad Request"
        casu 401 reddit "Unauthorized"
        casu 403 reddit "Forbidden"
        casu 404 reddit "Not Found"
        casu 500 reddit "Internal Server Error"
        casu 502 reddit "Bad Gateway"
        casu 503 reddit "Service Unavailable"
        ceterum reddit "Unknown Status"
    }
}

# Type mapping using reddit
functio getTypeCode(textus name) -> numerus {
    elige name {
        casu "textus" reddit 1
        casu "numerus" reddit 2
        casu "fractus" reddit 3
        casu "bivalens" reddit 4
        ceterum reddit 0
    }
}

# Mixed reddit and blocks
# Use reddit for simple returns, blocks for complex logic
functio processCode(numerus code) -> textus {
    elige code {
        casu 1 reddit "simple"
        casu 2 {
            scribe "Processing code 2..."
            redde "complex"
        }
        casu 3 reddit "also simple"
        ceterum reddit "default"
    }
}

incipit {
    scribe getStatusText(200)    # OK
    scribe getStatusText(404)    # Not Found
    scribe getStatusText(999)    # Unknown Status

    scribe getTypeCode("textus")   # 1
    scribe getTypeCode("unknown")  # 0

    scribe processCode(1)  # simple
    scribe processCode(2)  # Processing code 2... complex
}
```

### ceterum

```faber
# Elige with default case (ceterum)
#
# elige <expr> {
#     casu <value> { <body> }
#     ceterum { <default> }
# }

incipit ergo cura arena {
    # ceterum handles unmatched cases
    fixum day = "wednesday"

    elige day {
        casu "monday" {
            scribe "Start of week"
        }
        casu "friday" {
            scribe "End of week"
        }
        ceterum {
            scribe "Midweek"
        }
    }

    # ceterum with error handling
    fixum command = "unknown"

    elige command {
        casu "start" {
            scribe "Starting..."
        }
        casu "stop" {
            scribe "Stopping..."
        }
        casu "restart" {
            scribe "Restarting..."
        }
        ceterum {
            scribe "Unknown command"
        }
    }

    # Multiple statements in ceterum
    fixum level = 99

    elige level {
        casu 1 {
            scribe "Beginner"
        }
        casu 2 {
            scribe "Intermediate"
        }
        casu 3 {
            scribe "Advanced"
        }
        ceterum {
            scribe "Custom level"
            scribe "Level: §", level
        }
    }
}
```

### in-functione

```faber
# Elige with early returns in functions
#
# elige <expr> {
#     casu <value> { redde ... }
#     casu <value> { redde ... }
#     ceterum { redde ... }
# }

functio getGreeting(textus language) -> textus {
    elige language {
        casu "latin" {
            redde "Salve"
        }
        casu "english" {
            redde "Hello"
        }
        casu "spanish" {
            redde "Hola"
        }
        casu "french" {
            redde "Bonjour"
        }
    }

    redde "Hi"
}

functio getHttpMessage(numerus code) -> textus {
    elige code {
        casu 200 {
            redde "OK"
        }
        casu 201 {
            redde "Created"
        }
        casu 400 {
            redde "Bad Request"
        }
        casu 404 {
            redde "Not Found"
        }
        casu 500 {
            redde "Internal Server Error"
        }
    }

    redde "Unknown"
}

incipit {
    scribe getGreeting("latin")
    scribe getGreeting("spanish")
    scribe getGreeting("unknown")

    scribe getHttpMessage(200)
    scribe getHttpMessage(404)
    scribe getHttpMessage(999)
}
```

## est

### est

```faber
# Type checking with est, boolean checking with verum/falsum
#
# est = "is" (type check, like instanceof)
# verum x = "x is true" (strict boolean check)
# falsum x = "x is false" (strict boolean check)

incipit {
    # Null checking with est
    fixum si numerus maybeValue = nihil
    fixum isNull = maybeValue est nihil
    scribe isNull

    # Boolean true check with verum prefix
    fixum enabled = verum
    fixum isTrue = verum enabled
    scribe isTrue

    # Boolean false check with falsum prefix
    fixum disabled = falsum
    fixum isFalse = falsum disabled
    scribe isFalse

    # Chained with logical operators
    fixum si textus name = nihil
    fixum needsDefault = name est nihil et verum enabled
    scribe needsDefault

    # Parenthesized for clarity
    fixum bothNull = (maybeValue est nihil) et (name est nihil)
    scribe bothNull
}
```

## et

### et

```faber
# TODO
incipit {}
```

## expressionis

### expressionis

```faber
# Regex literals using sed keyword
# Syntax: sed "pattern" with inline flags like (?i) for case-insensitive

incipit {
    # Simple patterns
    fixum digits = sed "\d+"
    fixum word = sed "\w+"

    # With inline flags (i = case insensitive, m = multiline)
    fixum insensitive = sed "(?i)hello"
    fixum multiline = sed "(?im)^start"

    # Complex patterns
    fixum email = sed "[^@]+@[^@]+"
    fixum paths = sed "/usr/local/.*"

    scribe digits
    scribe word
    scribe insensitive
    scribe multiline
    scribe email
    scribe paths
}
```

## externa

### externa

```faber
# External declarations with @ externa
#
# Mark declarations as externally provided (by runtime, FFI, or linker)
# No initializer or body required
#
# Target output:
#   TypeScript: declare const/function
#   Zig: extern var/fn

# Runtime globals (TypeScript/Bun)
@ externa
fixum ignotum Bun

@ externa
fixum ignotum process

# External functions
@ externa
functio require(textus path) -> ignotum

# Use the external declarations
incipit {
    # Access runtime globals
    fixum args = process.argv qua lista<textus>
    scribe "Args:", args.longitudo()

    # Check runtime
    fixum runtime = Bun.version qua textus
    scribe "Bun version:", runtime
}
```

## fac

### fac

```faber
# Fac statement - explicit scoping with optional do-while and error handling
#
# fac { <body> }                              - basic scope block
# fac { <body> } dum <condition>             - do-while scope
# fac { <body> } cape <err> { <handler> }    - scope with error handling

incipit {
    # Basic scope block (explicit scoping)
    fac {
        fixum x = 42
        scribe x
    }

    # Scope block isolates variables
    fac {
        fixum message = "Hello from fac block"
        scribe message
    }

    # Multiple statements in scope
    fac {
        fixum a = 10
        fixum b = 20
        fixum sum = a + b
        scribe sum
    }

    # Fac with do-while condition (repeats until condition is false)
    varia counter = 0
    fac {
        counter = counter + 1
        scribe scriptum("Iteration §", counter)
    } dum counter < 3

    # Fac with error handling
    fac {
        # This would normally throw an error
        # iace "Test error"
        scribe "Block executed successfully"
    } cape err {
        scribe scriptum("Caught error: §", err.nuntius)
    }

    # Fac with both do-while and error handling
    varia attempts = 0
    fac {
        attempts = attempts + 1
        scribe scriptum("Attempt §", attempts)
        si attempts == 2 {
            iace "Simulated failure"
        }
    } cape err {
        scribe scriptum("Failed on attempt §: §", attempts, err.nuntius)
    } dum attempts < 5
}
```

### cape

```faber
# Fac blocks with error handling (cape)
#
# fac { <body> } cape <error> { <handler> }

incipit {
    # Basic fac with cape for error handling
    fac {
        fixum x = 10
        scribe x
    } cape err {
        scribe err
    }

    # Scope block that might throw
    fac {
        fixum value = 42
        scribe value
    } cape error {
        scribe "Error occurred:"
        scribe error
    }
}
```

## figendum

### figendum

```faber
# TODO
incipit {}
```

## finge

### finge

```faber
# Finge (variant construction) expressions
# Creates discretio variant instances

# Define discretio types
discretio Status {
    Active,
    Inactive,
    Pending
}

discretio Event {
    Click { numerus x, numerus y },
    Keypress { textus key },
    Quit
}

discretio Result {
    Success { textus message },
    Failure { textus error }
}

incipit {
    # Unit variants with explicit type
    fixum Status s1 = finge Active qua Status
    fixum Status s2 = finge Pending qua Status

    # Payload variants with explicit type
    fixum Event e1 = finge Click { x: 100, y: 200 } qua Event
    fixum Event e2 = finge Keypress { key: "Enter" } qua Event
    fixum Event e3 = finge Quit qua Event

    # Result variants
    fixum Result r1 = finge Success { message: "Operation completed" } qua Result
    fixum Result r2 = finge Failure { error: "Something went wrong" } qua Result

    scribe "Finge expressions created"
}
```

## fixum

### fixum

```faber
# TODO
incipit {}
```

## functio

### typicus

```faber
# Functions with typed parameters
#
# functio <name>(type param, type param) -> type { <body> }

# Single typed parameter
functio quadratum(numerus n) -> numerus {
    redde n * n
}

# Multiple typed parameters
functio adde(numerus a, numerus b) -> numerus {
    redde a + b
}

# Mixed types with allocator (curata) for string formatting
functio describe(textus nomen, numerus aetas) curata alloc -> textus {
    redde scriptum("§ habet § annos", nomen, aetas)
}

# Boolean parameter and return
functio nega(bivalens valor) -> bivalens {
    redde non valor
}

# Function with fractus (float) type
functio media(fractus a, fractus b) -> fractus {
    redde (a + b) / 2.0
}

incipit ergo cura arena fixum alloc {
    scribe quadratum(7)

    scribe adde(100, 200)

    scribe describe("Julius", 30)

    scribe nega(verum)
    scribe nega(falsum)

    scribe media(3.0, 7.0)
}
```

### optionalis

```faber
# Optional parameters with si and vel
#
# si marks a parameter as optional
# vel provides a default value
#
# GRAMMAR:
#   parameter := (preposition)? 'si'? type name ('ut' alias)? ('vel' default)?

# Optional parameter without default (receives nihil if omitted)
functio greet(textus nomen, si textus titulus) curata alloc -> textus {
    si titulus est nihil {
        redde scriptum("Salve, §!", nomen)
    }
    redde scriptum("Salve, § §!", titulus, nomen)
}

# Optional parameter with default value
functio paginate(si numerus pagina vel 1, si numerus per_pagina vel 10) curata alloc -> textus {
    redde scriptum("Page § with § items", pagina, per_pagina)
}

# Preposition with optional: de si (borrowed, optional without default)
functio analyze(textus source, de si numerus depth) -> numerus {
    si depth est nihil {
        redde source.longitudo()
    }
    redde depth
}

# Mixed required and optional parameters
functio createUser(textus nomen, si numerus aetas vel 0, si bivalens activus vel verum) curata alloc -> textus {
    redde scriptum("User: §, age: §, active: §", nomen, aetas, activus)
}

incipit ergo cura arena fixum alloc {
    # Without optional arg
    scribe greet("Marcus")

    # With optional arg
    scribe greet("Marcus", "Dominus")

    # Default pagination
    scribe paginate()

    # Custom pagination
    scribe paginate(2, 25)

    # Partial defaults
    scribe paginate(5)

    # With borrowed optional
    scribe analyze("code")
    scribe analyze("code", 5)

    # Mixed args
    scribe createUser("Julia")
    scribe createUser("Julia", 25)
    scribe createUser("Julia", 25, falsum)
}
```

### recursio

```faber
# Recursive functions
#
# Functions that call themselves with a base case

# Factorial: n! = n * (n-1)!
functio factorial(numerus n) -> numerus {
    si n <= 1 {
        redde 1
    }
    redde n * factorial(n - 1)
}

# Fibonacci: fib(n) = fib(n-1) + fib(n-2)
functio fibonacci(numerus n) -> numerus {
    si n <= 0 {
        redde 0
    }
    si n == 1 {
        redde 1
    }
    redde fibonacci(n - 1) + fibonacci(n - 2)
}

# Sum from 1 to n
functio summatio(numerus n) -> numerus {
    si n <= 0 {
        redde 0
    }
    redde n + summatio(n - 1)
}

incipit {
    # Factorial examples
    scribe factorial(0)
    scribe factorial(1)
    scribe factorial(5)
    scribe factorial(10)

    # Fibonacci examples
    scribe fibonacci(0)
    scribe fibonacci(1)
    scribe fibonacci(10)

    # Sum examples
    scribe summatio(5)
    scribe summatio(10)
}
```

### functio

```faber
# Basic function declarations
#
# functio <name>() { <body> }
# functio <name>() -> <type> { <body> }

# Function with no parameters, no return
functio saluta() {
    scribe "Salve, Mundus!"
}

# Function with parameter, no explicit return type
functio dic(textus verbum) {
    scribe verbum
}

# Function with return type
functio nomen() -> textus {
    redde "Marcus Aurelius"
}

# Function with parameter and return type
functio duplica(numerus n) -> numerus {
    redde n * 2
}

incipit {
    saluta()

    dic("Bonum diem!")

    fixum rex = nomen()
    scribe rex

    scribe duplica(21)
}
```

## futura

### futura

```faber
# TODO
incipit {}
```

## futurum

### futurum

```faber
# TODO
incipit {}
```

## generis

### generis

```faber
# TODO
incipit {}
```

## genus

### methodi

```faber
# Genus with methods using ego (self) reference
#
# genus <Name> {
#     <type> <property>
#     functio <method>() -> <type> { ... ego.<property> ... }
# }

genus Rectangle {
    numerus width: 1
    numerus height: 1

    functio area() -> numerus {
        redde ego.width * ego.height
    }

    functio perimeter() -> numerus {
        redde 2 * (ego.width + ego.height)
    }

    functio isSquare() -> bivalens {
        redde ego.width == ego.height
    }
}

genus Counter {
    numerus count: 0

    functio increment() {
        ego.count = ego.count + 1
    }

    functio getValue() -> numerus {
        redde ego.count
    }
}

incipit {
    # Methods that return values
    fixum rect = novum Rectangle {
        width: 10,
        height: 5
    }

    scribe rect.area()
    scribe rect.perimeter()
    scribe rect.isSquare()

    # Methods that modify state
    varia counter = novum Counter

    scribe counter.getValue()

    counter.increment()
    scribe counter.getValue()

    counter.increment()
    counter.increment()
    scribe counter.getValue()
}
```

### genus

```faber
# Basic genus (class/struct) with properties
#
# genus <Name> {
#     <type> <property>
#     <type> <property>: <default>
# }

genus Punctum {
    numerus x
    numerus y
}

genus Persona {
    textus nomen
    numerus aetas: 0
    bivalens activus: verum
}

incipit {
    # Instantiate with all required fields
    fixum p = novum Punctum {
        x: 10,
        y: 20
    }

    scribe p.x
    scribe p.y

    # Instantiate with required + optional defaults
    fixum marcus = novum Persona {
        nomen: "Marcus"
    }

    scribe marcus.nomen
    scribe marcus.aetas
    scribe marcus.activus

    # Override defaults
    fixum julia = novum Persona {
        nomen: "Julia",
        aetas: 25,
        activus: falsum
    }

    scribe julia.nomen
    scribe julia.aetas
    scribe julia.activus
}
```

### creo

```faber
# Genus with constructor hook (creo)
#
# genus <Name> {
#     <type> <property>: <default>
#     functio creo() { ... }
# }
#
# creo() runs after defaults and overrides are merged.
# Use for validation, clamping, or derived initialization.

genus BoundedValue {
    numerus value: 0

    functio creo() {
        si ego.value < 0 {
            ego.value = 0
        }

        si ego.value > 100 {
            ego.value = 100
        }
    }

    functio getValue() -> numerus {
        redde ego.value
    }
}

genus Circle {
    numerus radius: 1
    numerus diameter: 0
    numerus area: 0

    functio creo() {
        ego.diameter = ego.radius * 2
        ego.area = 3.14159 * ego.radius * ego.radius
    }
}

incipit {
    # Validation in creo
    fixum normal = novum BoundedValue {
        value: 50
    }

    scribe normal.getValue()

    fixum clamped = novum BoundedValue {
        value: 200
    }

    scribe clamped.getValue()

    # Derived initialization in creo
    fixum c = novum Circle {
        radius: 5
    }

    scribe c.radius
    scribe c.diameter
    scribe c.area
}
```

## hal

### json

```faber
# HAL Example: JSON Encoding/Decoding
#
# Demonstrates JSON serialization using the json HAL module.

importa ex "../../norma/hal/consolum" privata consolum
importa ex "../../norma/hal/json" privata json

incipit {
    consolum.fundeLineam("=== JSON Demo ===\n")

    # Serialize objects
    consolum.fundeLineam("--- Serialization ---")
    fixum obj = { "name": "Alice", "age": 30, "active": verum }
    fixum compact = json.solve(obj)
    consolum.fundeLineam(scriptum("Compact: §", compact))

    fixum pretty = json.solvePulchre(obj, 2)
    consolum.fundeLineam(scriptum("Pretty:\n§", pretty))

    # Parse JSON
    consolum.fundeLineam("\n--- Parsing ---")
    fixum parsed = json.pange("{\"x\": 100, \"y\": 200}")
    consolum.fundeLineam(scriptum("Parsed: §", parsed))

    # Safe parse (returns nihil on error)
    fixum good = json.pangeTuto("{\"valid\": true}")
    consolum.fundeLineam(scriptum("Good JSON: §", good))

    fixum bad = json.pangeTuto("not valid json")
    consolum.fundeLineam(scriptum("Bad JSON: §", bad))

    # Type checking
    consolum.fundeLineam("\n--- Type Checking ---")
    consolum.fundeLineam(scriptum("estTextus('hello'): §", json.estTextus("hello")))
    consolum.fundeLineam(scriptum("estNumerus(42): §", json.estNumerus(42)))
    consolum.fundeLineam(scriptum("estNihil(null): §", json.estNihil(nihil)))

    # Value extraction with defaults
    consolum.fundeLineam("\n--- Value Extraction ---")
    fixum data = { "name": "Bob", "count": 42 }
    fixum name = json.cape(data, "name")
    fixum count = json.cape(data, "count")
    consolum.fundeLineam(scriptum("name: §", json.utTextus(name, "unknown")))
    consolum.fundeLineam(scriptum("count: §", json.utNumerus(count, 0)))

    # Path access for nested data
    consolum.fundeLineam("\n--- Path Access ---")
    fixum nested = { "user": { "profile": { "email": "alice@example.com" } } }
    fixum email = json.capeVia(nested, "user.profile.email")
    consolum.fundeLineam(scriptum("email: §", email))

    consolum.fundeLineam("\n=== Demo Complete ===")
}
```

### consolum

```faber
# HAL Example: Console I/O
#
# Demonstrates console input/output using the consolum HAL module.

importa ex "../../norma/hal/consolum" privata consolum

incipit {
    consolum.fundeLineam("=== Console I/O Demo ===\n")

    # Basic text output
    consolum.fundeTextum("Text without newline... ")
    consolum.fundeLineam("and text with newline")

    # Check if running in a terminal
    si consolum.estTerminale() {
        consolum.fundeLineam("✓ stdin is a TTY (terminal)")
    }
    secus {
        consolum.fundeLineam("✗ stdin is NOT a TTY (piped/redirected)")
    }

    si consolum.estTerminaleOutput() {
        consolum.fundeLineam("✓ stdout is a TTY (terminal)")
    }
    secus {
        consolum.fundeLineam("✗ stdout is NOT a TTY (piped/redirected)")
    }

    # Write to stderr
    consolum.fundeLineam("\n--- Writing to stderr ---")
    consolum.errorLineam("This is an error message on stderr")
    consolum.errorTextum("Error without newline... ")
    consolum.errorLineam("and another error line")

    # Byte output
    consolum.fundeLineam("\n--- Byte output ---")
    consolum.fundeLineam("Writing raw bytes: [72, 101, 108, 108, 111] = 'Hello'")
    # TODO: Need octeti literal syntax to demonstrate fundeOctetos
    # consolum.fundeOctetos([72, 101, 108, 108, 111])

    consolum.fundeLineam("\n=== Demo Complete ===")
    consolum.fundeLineam("(Note: stdin reading functions like hauriLineam() require interactive input)")
}
```

### processus

```faber
# HAL Example: Process Management
#
# Demonstrates process control using the processus HAL module.

importa ex "../../norma/hal/consolum" privata consolum
importa ex "../../norma/hal/processus" privata processus

incipit {
    consolum.fundeLineam("=== Process Management Demo ===\n")

    # Current working directory
    fixum dir = processus.cwd()
    consolum.fundeLineam(scriptum("Current directory: §", dir))

    # Process ID
    fixum pid = processus.pid()
    consolum.fundeLineam(scriptum("Process ID: §", pid))

    # Command line arguments
    fixum args = processus.argv()
    consolum.fundeLineam(scriptum("Arguments: §", args))

    # Environment variables
    fixum path = processus.envVel("PATH", "(not set)")
    consolum.fundeLineam(scriptum("PATH: §", path))

    fixum home = processus.envVel("HOME", "(not set)")
    consolum.fundeLineam(scriptum("HOME: §", home))

    # Execute a simple command
    consolum.fundeLineam("\n--- Executing: echo 'Hello from subprocess' ---")
    fixum output = processus.exsequi("echo 'Hello from subprocess'")
    consolum.fundeLineam(scriptum("Output: §", output))

    # Execute command and check exit code
    consolum.fundeLineam("\n--- Checking exit code: true ---")
    fixum exitCode = processus.exsequiCodem("true")
    consolum.fundeLineam(scriptum("Exit code: §", exitCode))

    consolum.fundeLineam("\n=== Demo Complete ===")
}
```

### aleator

```faber
# HAL Example: Entropy / Random Numbers
#
# Demonstrates random number generation using the aleator HAL module.

importa ex "../../norma/hal/consolum" privata consolum
importa ex "../../norma/hal/aleator" privata aleator

incipit {
    consolum.fundeLineam("=== Entropy / Random Demo ===\n")

    # Basic random numbers
    consolum.fundeLineam("--- Basic Generation ---")
    fixum f = aleator.fractus()
    consolum.fundeLineam(scriptum("Random float [0,1): §", f))

    fixum n = aleator.inter(1, 100)
    consolum.fundeLineam(scriptum("Random int [1,100]: §", n))

    # Cryptographic
    consolum.fundeLineam("\n--- Cryptographic ---")
    fixum id = aleator.uuid()
    consolum.fundeLineam(scriptum("UUID v4: §", id))

    fixum bytes = aleator.octeti(8)
    consolum.fundeLineam(scriptum("Random 8 bytes: §", bytes))

    # Seeded (reproducible) random
    consolum.fundeLineam("\n--- Seeded Random (reproducible) ---")
    aleator.semen(42)
    consolum.fundeLineam(scriptum("Seed 42, first:  §", aleator.inter(1, 100)))
    consolum.fundeLineam(scriptum("Seed 42, second: §", aleator.inter(1, 100)))

    aleator.semen(42)
    consolum.fundeLineam(scriptum("Seed 42 again:   §", aleator.inter(1, 100)))
    consolum.fundeLineam("(should match first)")

    aleator.semen(0)  # Reset to true random
    consolum.fundeLineam(scriptum("Reset, random:   §", aleator.inter(1, 100)))

    consolum.fundeLineam("\n=== Demo Complete ===")
}
```

### yaml

```faber
# HAL Example: YAML Encoding/Decoding
#
# Demonstrates YAML serialization using the yaml HAL module.

importa ex "../../norma/hal/consolum" privata consolum
importa ex "../../norma/hal/yaml" privata yaml

incipit {
    consolum.fundeLineam("=== YAML Demo ===\n")

    # Serialize objects
    consolum.fundeLineam("--- Serialization ---")
    fixum config = {
        "database": {
            "host": "localhost",
            "port": 5432,
            "name": "myapp"
        },
        "debug": verum
    }

    fixum serialized = yaml.solve(config)
    consolum.fundeLineam("Serialized:")
    consolum.fundeLineam(serialized)

    # Parse YAML
    consolum.fundeLineam("--- Parsing ---")
    fixum yamlText = "name: Alice\nage: 30\nactive: true"
    fixum parsed = yaml.pange(yamlText)
    consolum.fundeLineam(scriptum("Parsed: §", parsed))

    # Safe parse (returns nihil on error)
    fixum good = yaml.pangeTuto("valid: yaml")
    consolum.fundeLineam(scriptum("Good YAML: §", good))

    fixum bad = yaml.pangeTuto("[unclosed")
    consolum.fundeLineam(scriptum("Bad YAML: §", bad))

    # Multi-document YAML
    consolum.fundeLineam("\n--- Multi-Document ---")
    fixum multiYaml = "a: 1\n---\nb: 2\n---\nc: 3"
    fixum lista<quidlibet> docs = yaml.pangeMulti(multiYaml)
    consolum.fundeLineam(scriptum("Document count: §", docs.longitudo()))
    consolum.fundeLineam(scriptum("First doc: §", docs[0]))
    consolum.fundeLineam(scriptum("Last doc: §", docs[2]))

    # Type checking
    consolum.fundeLineam("\n--- Type Checking ---")
    consolum.fundeLineam(scriptum("estTextus('hello'): §", yaml.estTextus("hello")))
    consolum.fundeLineam(scriptum("estNumerus(42): §", yaml.estNumerus(42)))

    consolum.fundeLineam("\n=== Demo Complete ===")
}
```

## iace

### iace

```faber
# Basic iace (throw) statement
#
# iace <expression>

incipit {
    # Throw with string literal
    tempta {
        iace "Something went wrong"
    }
    cape err {
        scribe "Caught:", err
    }

    # Throw with formatted message
    fixum code = 404
    tempta {
        iace scriptum("Error code: §", code)
    }
    cape err {
        scribe "Caught:", err
    }

    # Throw from conditional
    fixum value = -5
    tempta {
        si value < 0 {
            iace "Value must be non-negative"
        }
        scribe "Value is valid"
    }
    cape err {
        scribe "Validation failed:", err
    }
}
```

## implet

### implet

```faber
# TODO
incipit {}
```

## importa

### importa

```faber
# Import statements (importa)
#
# importa ex <source> privata <name>           - named imports
# importa ex <source> privata <name> ut <alias> - import with alias
# importa ex <source> privata * ut <alias>      - wildcard import with alias

# Named imports
importa ex "lodash" privata map
importa ex "@hono/hono" privata Hono
importa ex "@hono/hono" privata Context
importa ex "utils" privata helper ut h
importa ex "db" privata connect
importa ex "db" privata query ut q
importa ex "db" privata close

# Wildcard import (alias required for TypeScript target)
importa ex "@std/crypto" privata * ut crypto

# Multiple imports from different sources
importa ex "@oak/oak" privata Application
importa ex "std/path" privata join
importa ex "std/path" privata resolve

# Relative imports (commented out - modules don't exist in exempla)
# importa ex "./utils" privata helper
# importa ex "../shared/utils" privata formatter

# Many named items
importa ex "helpers" privata a
importa ex "helpers" privata b
importa ex "helpers" privata c
importa ex "helpers" privata d
importa ex "helpers" privata e
importa ex "helpers" privata f

# Multiple aliases
importa ex "mod" privata foo ut f
importa ex "mod" privata bar ut b
importa ex "mod" privata baz ut z

incipit {
    scribe "Import statements are declarations at module scope"
}
```

### auxilia

```faber
# Example utility module for local import testing

# A simple greeting function (needs allocator for string formatting)
functio greet(textus name) curata alloc -> textus {
    redde scriptum("Hello, §!", name)
}

# A constant value
fixum numerus ANSWER = 42

# A genus for testing
genus Point {
    numerus x
    numerus y
}
```

## incipiet

### incipiet

```faber
# Async program entry point with incipiet
#
# incipiet creates an async entry point
# Use when your program needs to await async operations
#
# Syntax:
#   incipiet { <async body> }
#   incipiet ergo <statement>
#   incipiet reddit <expression>

# Helper async function
@ futura
functio fetchData() -> textus {
    # Simulates async operation
    redde "data loaded"
}

@ futura
functio processData(textus data) -> numerus {
    redde data.longitudo()
}

# Async entry point
incipiet {
    scribe "Starting async program..."

    # Await async functions with cede
    fixum data = cede fetchData()
    scribe "Received:", data

    fixum length = cede processData(data)
    scribe "Length:", length

    scribe "Program complete"
}
```

## incipit

### incipit

```faber
# Basic incipit (entry point)
#
# incipit { <body> }

incipit {
    scribe "Salve, Munde!"
}
```

### functionibus

```faber
# Entry point with functions defined outside
#
# Functions declared outside incipit become module-level declarations.
# The incipit block calls them as needed.

functio greet(textus name) curata alloc -> textus {
    redde scriptum("Salve, §!", name)
}

functio add(numerus a, numerus b) -> numerus {
    redde a + b
}

incipit ergo cura arena fixum alloc {
    scribe greet("Marcus")
    scribe "Sum:", add(3, 5)
}
```

## innatum

### innatum

```faber
# Native type construction with innatum
#
# innatum constructs proper native instances of builtin types
# Unlike qua (type assertion), innatum creates the native representation
#
# Syntax:
#   {} innatum tabula<K, V>  -> new Map() in TS, HashMap in Rust, etc.
#   [] innatum lista<T>      -> proper array/vector initialization

incipit ergo cura arena fixum alloc {
    # Empty tabula (Map)
    varia cache = {} innatum tabula<textus, numerus>
    cache["foo"] = 42
    cache["bar"] = 99
    scribe cache["foo"]  # 42

    # Empty lista (Array)
    varia items = [] innatum lista<textus>
    items.appende("alpha")
    items.appende("beta")
    scribe items.longitudo()  # 2

    # Non-empty tabula with initial values
    fixum scores = { alice: 95, bob: 87 } innatum tabula<textus, numerus>
    scribe scores["alice"]  # 95

    # Non-empty lista
    fixum nums = [1, 2, 3] innatum lista<numerus>
    scribe nums.primus()  # 1

    # Why innatum is needed:
    # Without innatum, {} is just an object literal, not a Map
    # varia broken = {} qua tabula<textus, numerus>
    # broken["key"] = 1  # Error: Map methods don't exist

    # With innatum, we get the native Map instance
    varia working = {} innatum tabula<textus, numerus>
    working["key"] = 1  # Works: properly constructed Map
}
```

## inter

### inter

```faber
# Set membership operator: inter

fixum status = "active"
fixum age = 21

# Basic inter with string array
si status inter ["pending", "active", "paused"] {
    scribe "valid status"
}

# inter with numeric array
si age inter [18, 21, 65] {
    scribe "milestone age"
}

incipit {
    scribe "inter operator examples"
}
```

## intra

### intra

```faber
# Range containment operator: intra

fixum age = 25

# Basic intra with .. operator (exclusive end)
si age intra 0..100 {
    scribe "age is in valid range"
}

# intra with usque (inclusive end)
si age intra 18 usque 65 {
    scribe "working age"
}

# intra with ante (explicit exclusive)
si age intra 0 ante 18 {
    scribe "minor"
}

incipit {
    scribe "intra operator examples"
}
```

## itera

### nidificatus

```faber
# Nested itera ex loops

incipit {
    # Nested array iteration
    fixum rows = [1, 2, 3]
    fixum cols = ["A", "B", "C"]

    itera ex rows fixum row {
        itera ex cols fixum col {
            scribe row, col
        }
    }

    # Multiplication table
    itera pro 1..4 fixum i {
        itera pro 1..4 fixum j {
            scribe i, "*", j, "=", i * j
        }
    }

    # Nested ranges
    itera pro 0..3 fixum x {
        itera pro 0..3 fixum y {
            scribe x, y
        }
    }
}
```

### ex

```faber
# Iterating over arrays with itera ex
#
# itera ex <collection> fixum <item> { <body> }

incipit {
    # Iterate over number array
    fixum numbers = [1, 2, 3, 4, 5]

    itera ex numbers fixum n {
        scribe n
    }

    # Iterate over string array
    fixum names = ["Marcus", "Julia", "Claudia"]

    itera ex names fixum name {
        scribe name
    }

    # Process items
    fixum values = [10, 20, 30]

    itera ex values fixum v {
        fixum doubled = v * 2
        scribe doubled
    }
}
```

### cursor-iteratio

```faber
# Iterating over generator function returns with itera ex
#
# This demonstrates that @ cursor functions produce iterable results
# that can be consumed with itera ex loops

# Multi-value sync function that yields values via cede
@ cursor
functio rangeSync(numerus n) -> numerus {
    itera pro 0..n fixum i {
        cede i
    }
}

# Multi-value async function that yields values via cede
@ futura
@ cursor
functio rangeAsync(numerus n) -> numerus {
    itera pro 0..n fixum i {
        cede i
    }
}

incipit {
    # Iterate over sync generator function results
    scribe "Sync cursor iteration:"
    itera ex rangeSync(3) fixum num {
        scribe "  num: {num}"
    }

    # Collect all results from generator function
    varia syncResults = []
    itera ex rangeSync(5) fixum num {
        syncResults.appende(num * 2)
    }
    scribe "Sync collected:"
    scribe syncResults

    # Note: Async iteration would require async context
    # itera ex rangeAsync(3) fixum num {
    #     scribe num
    # }
}
```

### de

```faber
# Basic itera de (for-in) key iteration
#
# itera de <object> fixum <key> { <body> }
# itera de <array> fixum <index> { <body> }

incipit {
    # Iterate over object keys
    fixum persona = { nomen: "Marcus", aetas: 30, urbs: "Roma" }

    itera de persona fixum clavis {
        scribe clavis
    }

    # Access values using the key
    itera de persona fixum clavis {
        scribe clavis, persona[clavis]
    }

    # Iterate over array indices
    fixum numeri = [10, 20, 30]

    itera de numeri fixum index {
        scribe "Index", index, numeri[index]
    }

    # With objects
    fixum data = { alpha: 1, beta: 2 }
    itera de data fixum k {
        scribe k
    }
}
```

### intervallum-gradus

```faber
# Ranges with step using per
#
# itera pro <start>..<end> per <step> fixum <item> { }
# itera pro <start> usque <end> per <step> fixum <item> { }

incipit {
    # Step by 2 (exclusive: 0, 2, 4, 6, 8)
    itera pro 0..10 per 2 fixum i {
        scribe i
    }

    # Step by 2 (inclusive: 0, 2, 4, 6, 8, 10)
    itera pro 0 usque 10 per 2 fixum i {
        scribe i
    }

    # Step by 3
    itera pro 0..15 per 3 fixum i {
        scribe i
    }

    # Countdown with negative step
    itera pro 10..0 per -1 fixum i {
        scribe i
    }

    # Countdown by 2
    itera pro 10..0 per -2 fixum i {
        scribe i
    }
}
```

*2 more examples in this category*

## lege

### lege

```faber
# TODO
incipit {}
```

## lista

### lista

```faber
# Array literal expressions: empty, typed, nested, and spread

incipit {
    # Empty array
    fixum empty = []

    # Number array
    fixum numbers = [1, 2, 3, 4, 5]

    # String array
    fixum names = ["Marcus", "Julia", "Gaius"]

    # Boolean array
    fixum flags = [verum, falsum, verum]

    # Nested arrays
    fixum matrix = [[1, 2], [3, 4], [5, 6]]

    # Spread operator: combine arrays
    fixum first = [1, 2, 3]
    fixum second = [4, 5, 6]
    fixum combined = [sparge first, sparge second]

    # Spread with additional elements
    fixum extended = [0, sparge first, 99]

    scribe empty
    scribe numbers
    scribe names
    scribe flags
    scribe matrix
    scribe combined
    scribe extended
}
```

## literalis

### literalis

```faber
# Literal expressions: numbers, strings, booleans, null, templates

incipit {
    # Numbers
    fixum integer = 42
    fixum decimal = 3.14
    fixum negative = -100

    # Strings
    fixum greeting = "hello"
    fixum single = 'single quotes'

    # Booleans
    fixum yes = verum
    fixum no = falsum

    # Null
    fixum nothing = nihil

    # Template literals
    fixum name = "Mundus"
    fixum message = `Hello ${name}`

    scribe integer
    scribe decimal
    scribe negative
    scribe greeting
    scribe single
    scribe yes
    scribe no
    scribe nothing
    scribe message
}
```

## membrum

### membrum

```faber
# Member access expressions: dot, bracket, chained, optional

incipit {
    # Object with dot access
    fixum point = { x: 10, y: 20 }
    scribe point.x
    scribe point.y

    # Array with bracket access
    fixum numbers = [1, 2, 3]
    scribe numbers[0]
    scribe numbers[2]

    # Object with bracket access (string key)
    fixum config = { name: "test", value: 42 }
    scribe config["name"]
    scribe config["value"]

    # Chained member access
    fixum nested = { outer: { inner: { deep: "found" } } }
    scribe nested.outer.inner.deep

    # Mixed dot and bracket access
    fixum data = { items: ["first", "second", "third"] }
    scribe data.items[0]
    scribe data.items[2]

    # Optional chaining
    fixum maybe = { present: { value: 100 } }
    scribe maybe?.present?.value

    # Optional chaining with nihil
    fixum empty = nihil
    scribe empty?.missing
}
```

## misc

### salve-munde

```faber
# Salve, Munde!

scribe "Salve, Munde!"
```

## mone

### mone

```faber
# TODO
incipit {}
```

## mori

### mori

```faber
# Fatal errors with mori (panic)
#
# mori is for unrecoverable errors (panic/fatal)
# iace is for recoverable errors (throw/catch)
#
# Difference:
#   iace - can be caught with cape
#   mori - terminates program immediately

functio divide(numerus a, numerus b) -> fractus {
    # Use mori for invariant violations
    si b == 0 {
        mori "Division by zero is a fatal error"
    }
    redde a / b
}

functio accessArray(lista<numerus> items, numerus index) -> numerus {
    # Bounds check with panic
    si index < 0 aut index >= items.longitudo() {
        mori "Index out of bounds"
    }
    redde items[index]
}

incipit {
    # Normal operation
    fixum result = divide(10, 2)
    scribe "Result:", result  # 5.0

    # This would panic (uncomment to test):
    # fixum bad = divide(10, 0)  # Fatal: Division by zero

    # Array access
    fixum nums = [1, 2, 3]
    fixum val = accessArray(nums, 1)
    scribe "Value:", val  # 2

    # This would panic (uncomment to test):
    # fixum bad = accessArray(nums, 10)  # Fatal: Index out of bounds
}
```

## morphologia

### morphologia

```faber
# Morphologia - Verb Conjugation Dispatch Demo
#
# Demonstrates how Latin verb conjugations determine method semantics:
# - Imperative (-a, -e, -i): mutates in place
# - Perfect participle (-ata, -ita, -sa): returns new collection
#
# Run with: bun run faber compile exempla/morphologia.fab

incipit {
    varia items = [1, 2, 3, 4, 5] qua numerus[]

    # ==========================================================================
    # FILTER: filtr-
    # ==========================================================================

    # filtrata (participle) - returns new filtered list
    fixum evens = items.filtrata(clausura numerus x: x % 2 == 0)
    scribe "filtrata (returns new):", evens

    # TODO (type errors) - filtra (imperative) - mutates in place
    # varia nums = [1, 2, 3, 4, 5]
    # nums.filtra(clausura x: x > 2)
    # scribe "filtra (mutates):", nums

    # ==========================================================================
    # ADD: add-
    # ==========================================================================

    # addita (participle) - returns new list with element added
    fixum extended = items.addita(6)
    scribe "addita (returns new):", extended

    # appende (imperative) - mutates in place
    varia list = [1, 2, 3]
    list.appende(4)
    scribe "appende (mutates):", list

    # ==========================================================================
    # REVERSE: invert-/invers-
    # ==========================================================================

    # inversa (participle) - returns new reversed list
    fixum reversed = items.inversa()
    scribe "inversa (returns new):", reversed

    # inverte (imperative) - mutates in place
    varia toReverse = [1, 2, 3]
    toReverse.inverte()
    scribe "inverte (mutates):", toReverse

    # ==========================================================================
    # SORT: ordin-
    # ==========================================================================

    # ordinata (participle) - returns new sorted list
    fixum sorted = items.ordinata()
    scribe "ordinata (returns new):", sorted

    # NOTE: ordina (imperative) conflicts with keyword - use ordinata for now

    # ==========================================================================
    # MAP: mapp-
    # ==========================================================================

    # mappata (participle) - returns new mapped list
    fixum doubled = items.mappata(clausura numerus x: x * 2)
    scribe "mappata (returns new):", doubled

    # ==========================================================================
    # Original list unchanged by participle operations
    # ==========================================================================
    scribe "original items:", items
}
```

## nexum

### nexum

```faber
# TODO
incipit {}
```

## novum

### novum

```faber
# Novum (instantiation) expressions
# Creates new instances of genus types

genus Point {
    numerus x
    numerus y
}

genus Person {
    textus name
    numerus age: 0
}

incipit {
    # Instantiate with all fields
    fixum p1 = novum Point { x: 10, y: 20 }
    scribe p1.x
    scribe p1.y

    # With default value
    fixum person = novum Person { name: "Marcus" }
    scribe person.name
    scribe person.age
}
```

## objectum

### objectum

```faber
# Object literal expressions

incipit {
    # Empty object
    fixum empty = {}
    scribe empty

    # Simple object with numeric values
    fixum point = { x: 10, y: 20 }
    scribe point

    # String keys
    fixum value = 42
    fixum withStringKey = { "key": value }
    scribe withStringKey

    # Properties from variables
    fixum name = "Marcus"
    fixum age = 30
    fixum person = { name: name, age: age }
    scribe person

    # Nested objects
    fixum nested = { outer: { inner: 1 } }
    scribe nested

    # Spread operator
    fixum base = { a: 1, b: 2 }
    scribe base
    fixum extended = { sparge base, c: 3 }
    scribe extended
}
```

## omitte

### omitte

```faber
# TODO
incipit {}
```

## omnia

### omnia

```faber
# TODO
incipit {}
```

## optionalis

### optionalis

```faber
# Optional chaining with ?.
#
# Safely access nested properties that might be null/undefined
#
# Syntax:
#   object?.property
#   object?[index]

genus Address {
    textus city
    si textus state
}

genus User {
    textus name
    si Address address
}

incipit {
    # User with full address
    fixum alice = novum User {
        name: "Alice",
        address: novum Address { city: "Roma", state: "Italia" }
    }

    # Optional chaining prevents errors when properties might be nihil
    fixum city = alice?.address?.city
    scribe city  # "Roma"

    fixum state = alice?.address?.state
    scribe state  # "Italia"

    # User with no address
    fixum bob = novum User { name: "Bob" }
    fixum bobCity = bob?.address?.city
    scribe bobCity  # nihil (no error thrown)

    # Chaining through multiple levels
    fixum bobState = bob?.address?.state
    scribe bobState  # nihil

    # Optional array access
    fixum items = ["a", "b", "c"]
    scribe items?[0]  # "a"
    scribe items?[10] # undefined/nihil (out of bounds)

    # Combining with vel for defaults
    fixum defaultCity = bob?.address?.city vel "Unknown"
    scribe defaultCity  # "Unknown"
}
```

## ordo

### ordo

```faber
# Basic ordo (enum) declaration
#
# ordo Name { Member1, Member2, Member3 }
# ordo Name { Member1 = value1, Member2 = value2 }

ordo Color { rubrum, viridis, caeruleum }

ordo Status { pendens = 0, actum = 1, finitum = 2 }

incipit {
    # Using enum values
    fixum color = Color.rubrum
    fixum status = Status.actum

    # Switch on enum
    elige color {
        casu Color.rubrum {
            scribe "Red"
        }
        casu Color.viridis {
            scribe "Green"
        }
        casu Color.caeruleum {
            scribe "Blue"
        }
    }

    # Switch on enum with numeric values
    elige status {
        casu Status.pendens {
            scribe "Pending"
        }
        casu Status.actum {
            scribe "Active"
        }
        casu Status.finitum {
            scribe "Finished"
        }
    }
}
```

## pactum

### pactum

```faber
# Basic pactum (interface) definition and implementation
#
# pactum <Name> { functio <method>(<params>) -> <returnType> }
# genus <Name> implet <Pactum> { <implementation> }

pactum Drawable {
    functio draw() -> vacuum
}

genus Circle implet Drawable {
    numerus radius: 10

    functio draw() {
        scribe "Drawing circle with radius:", ego.radius
    }
}

genus Square implet Drawable {
    numerus side: 5

    functio draw() {
        scribe "Drawing square with side:", ego.side
    }
}

incipit {
    fixum circle = novum Circle { radius: 25 }
    fixum square = novum Square { side: 15 }

    circle.draw()
    square.draw()
}
```

## per

### per

```faber
# TODO
incipit {}
```

## perge

### perge

```faber
# Continue (perge) in loops
#
# perge skips to the next iteration of the innermost loop

incipit ergo cura arena {
    # Skip even numbers
    varia i = 0

    dum i < 10 {
        i = i + 1

        si i % 2 == 0 {
            perge
        }

        scribe i
    }

    # Continue in nested loop (affects inner only)
    varia outer = 0

    dum outer < 3 {
        varia inner = 0

        dum inner < 5 {
            inner = inner + 1

            si inner == 3 {
                perge
            }

            scribe "outer=§, inner=§", outer, inner
        }

        outer = outer + 1
    }
}
```

## postpara

### postpara

```faber
# TODO
incipit {}
```

## praefixum

### praefixum

```faber
# Compile-time evaluation with praefixum
#
# praefixum evaluates expressions at compile time
# Values are computed during compilation, not runtime
#
# Syntax:
#   praefixum(expression)
#   praefixum { block }

incipit {
    # Compile-time constant expression
    fixum myPi = praefixum(3.14159)
    fixum myTau = praefixum(2 * 3.14159)
    scribe "myPi:", myPi
    scribe "myTau:", myTau

    # Compile-time computation
    fixum DAYS_IN_YEAR = praefixum(365)
    fixum HOURS_IN_YEAR = praefixum(365 * 24)
    fixum SECONDS_IN_YEAR = praefixum(365 * 24 * 60 * 60)

    scribe "Days:", DAYS_IN_YEAR
    scribe "Hours:", HOURS_IN_YEAR
    scribe "Seconds:", SECONDS_IN_YEAR

    # Compile-time block
    fixum STATUS_CODES = praefixum({
        ok: 200,
        created: 201,
        notFound: 404,
        error: 500
    })

    scribe "OK code:", STATUS_CODES.ok
}
```

## praepara

### praepara

```faber
# TODO
incipit {}
```

## privatus

### privatus

```faber
# TODO
incipit {}
```

## proba

### proba

```faber
# Basic proba (test) statements
#
# proba "name" { body }

# Simple test with single assertion
proba "one plus one equals two" {
    adfirma 1 + 1 == 2
}

# Test with multiple assertions
proba "validates arithmetic" {
    adfirma 2 + 2 == 4
    adfirma 10 - 3 == 7
    adfirma 3 * 4 == 12
}

# Test with variables
proba "string concatenation works" {
    fixum greeting = "hello"
    fixum name = "world"
    fixum result = scriptum("§ §", greeting, name)
    adfirma result == "hello world"
}

# Test boolean conditions
proba "comparison operators" {
    fixum x = 10
    adfirma x > 5
    adfirma x < 20
    adfirma x >= 10
    adfirma x <= 10
}

# Test with negation
proba "negated assertions" {
    adfirma non falsum
    adfirma non (1 == 2)
}

# Test with complex logical assertion
proba "complex assertion" {
    fixum x = 50
    adfirma x > 0 et x < 100
}
```

### modificatores

```faber
# Test modifiers: omitte (skip) and futurum (todo)
#
# proba omitte "reason" "name" { body }
# proba futurum "reason" "name" { body }

# Skip a test with reason and name
proba omitte "blocked by issue #42" "database connection test" {
    adfirma falsum
}

# Todo test with reason and name
proba futurum "needs async support" "async file operations" {
    adfirma verum
}

# Regular test alongside modifiers
proba "this test runs normally" {
    adfirma 1 + 1 == 2
}

# Multiple skipped tests
proba omitte "flaky on CI" "network timeout test" {
    adfirma falsum
}

proba omitte "platform specific" "windows-only behavior" {
    adfirma falsum
}

# Multiple todo tests
proba futurum "needs new API" "graphql mutations" {
    adfirma verum
}

proba futurum "depends on feature X" "caching layer" {
    adfirma verum
}

# Skipped test with complex body
proba omitte "external service down" "api integration" {
    fixum status = 500
    adfirma status == 200
}

# Todo test with setup
proba futurum "needs database fixtures" "user creation flow" {
    varia userId = 0
    adfirma userId > 0
}
```

## protectus

### protectus

```faber
# TODO
incipit {}
```

## publicus

### publicus

```faber
# TODO
incipit {}
```

## qua

### qua

```faber
# Type casting with qua: converts values between types

functio getData() -> lista<numerus> {
    redde [1, 2, 3]
}

functio getResponse() -> objectum {
    redde { body: "body" }
}

functio getValue() -> numerus {
    redde 42
}

incipit {
    # Cast to string
    fixum data = 42
    fixum asText = data qua textus
    scribe asText

    # Cast to number
    fixum input = "100"
    fixum asNum = input qua numerus
    scribe asNum

    # Cast to boolean
    fixum value = 1
    fixum asBool = value qua bivalens
    scribe asBool

    # Cast to nullable type
    fixum num = 10
    fixum maybe = num qua si numerus
    scribe maybe

    # Cast to array type
    fixum raw = getData()
    fixum items = raw qua lista<textus>
    scribe items

    # Cast with member access
    fixum response = getResponse()
    fixum body = response.body qua textus
    scribe body

    # Cast call result directly
    fixum result = getValue() qua textus
    scribe result

    # Cast in parenthesized expression for chaining
    fixum len = (data qua textus).length
    scribe len
}
```

## redde

### redde

```faber
# Basic redde (return) statements
#
# redde <expression>   -- return a value
# redde                -- void return

functio add(numerus a, numerus b) -> numerus {
    redde a + b
}

functio greet(textus name) -> textus {
    redde "Hello, " + name
}

functio getFortyTwo() -> numerus {
    redde 42
}

functio doNothing() -> vacuum {
    redde
}

functio earlyExit(numerus x) -> numerus {
    si x < 0 {
        redde 0
    }
    redde x * 2
}

incipit {
    scribe add(10, 20)
    scribe greet("World")
    scribe getFortyTwo()
    doNothing()
    scribe earlyExit(-5)
    scribe earlyExit(10)
}
```

## rivus-cli-annotated

### cli

```faber
# ═══════════════════════════════════════════════════════════════════════════════
# RIVUS - Bootstrap Faber Compiler CLI
# ═══════════════════════════════════════════════════════════════════════════════
#
# Multi-target compiler for bootstrap testing.
# Compiles Faber source to TypeScript, Go, or other targets.
#
# Usage:
#   rivus emit hello.fab                 # compile to stdout (default: ts)
#   rivus emit hello.fab -t go           # compile to Go
#   rivus emit hello.fab -o hello.ts     # compile to file
#   rivus build main.fab -o dist/        # build with dependencies
#   rivus parse hello.fab                # dump AST as JSON
#   rivus check hello.fab                # validate without codegen
#
# ═══════════════════════════════════════════════════════════════════════════════

# Commands
importa ex "./cli/commands/emit" privata emitCommand
importa ex "./cli/commands/build" privata buildCommand
importa ex "./cli/commands/parse" privata parseCommand
importa ex "./cli/commands/check" privata checkCommand
importa ex "./cli/commands/test" privata testCommand
importa ex "./cli/commands/fetch" privata mandatumArcessere
importa ex "./cli/commands/run" privata mandatumCurrere

@ cli "rivus"
@ versio "0.1.0"
@ descriptio "Bootstrap Faber compiler - compiles Faber source to TypeScript or Go"
incipit {}

# ─────────────────────────────────────────────────────────────────────────────
# version - show version
# ─────────────────────────────────────────────────────────────────────────────

@ imperium "version"
@ alias "v"
functio version() -> vacuum {
    scribe "rivus v0.1.0"
}

# ─────────────────────────────────────────────────────────────────────────────
# emit - compile single file to target language
# ─────────────────────────────────────────────────────────────────────────────

@ imperium "emit"
@ alias "compile"
@ descriptio "Emit code (warns on errors unless --strict)"
@ futura
@ optio textus target brevis "t" longum "target" descriptio "Target language: ts (default), go"
@ optio textus output brevis "o" longum "output" descriptio "Output file (default: stdout)"
@ optio textus include brevis "I" longum "include" descriptio "Add import search path"
@ optio textus manifest brevis "m" longum "manifest" descriptio "Project manifest file (faber.fab)"
@ optio textus stdinFilename longum "stdin-filename" descriptio "Filename for error messages when reading stdin"
@ optio bivalens json longum "json" descriptio "Output errors as JSON"
@ optio bivalens dryRun longum "dry-run" descriptio "Check without emitting code"
@ optio bivalens stripTests longum "strip-tests" descriptio "Strip probandum/proba test blocks from output"
@ optio bivalens stripComments longum "strip-comments" descriptio "Strip comments from output"
@ optio bivalens strict longum "strict" descriptio "Fail on semantic errors instead of warning"
functio emit(
    si textus input,
    si textus target,
    si textus output,
    si textus include,
    si textus manifest,
    si textus stdinFilename,
    si bivalens json,
    si bivalens dryRun,
    si bivalens stripTests,
    si bivalens stripComments,
    si bivalens strict
) exitus code -> vacuum {
    fixum result = cede emitCommand(input, target, output, include, manifest, stdinFilename, json, dryRun, stripTests, stripComments, strict)
    code = result
}

# ─────────────────────────────────────────────────────────────────────────────
# build - compile entry and dependencies to directory
# ─────────────────────────────────────────────────────────────────────────────

@ imperium "build"
@ alias "aedifica"
@ descriptio "Build entry file and all dependencies to output directory"
@ futura
@ optio textus target brevis "t" longum "target" descriptio "Target language: ts (default), go"
@ optio textus output brevis "o" longum "output" descriptio "Output directory (default: ./dist)"
@ optio textus include brevis "I" longum "include" descriptio "Add import search path"
@ optio textus manifest brevis "m" longum "manifest" descriptio "Project manifest file (faber.fab)"
@ optio bivalens json longum "json" descriptio "Output errors as JSON"
@ optio bivalens dryRun longum "dry-run" descriptio "Check without writing files"
@ optio bivalens listFiles longum "list-files" descriptio "List files that would be compiled"
@ optio bivalens stripComments longum "strip-comments" descriptio "Strip comments from output"
functio build(
    textus input,
    si textus target,
    si textus output,
    si textus include,
    si textus manifest,
    si bivalens json,
    si bivalens dryRun,
    si bivalens listFiles,
    si bivalens stripComments
) exitus code -> vacuum {
    fixum result = cede buildCommand(input, target, output, include, manifest, json, dryRun, listFiles, stripComments)
    code = result
}

# ─────────────────────────────────────────────────────────────────────────────
# parse - lex and parse, dump AST
# ─────────────────────────────────────────────────────────────────────────────

@ imperium "parse"
@ alias "ast"
@ descriptio "Parse source and dump AST as JSON (reads stdin if no input)"
@ futura
@ optio textus output brevis "o" longum "output" descriptio "Output file (default: stdout)"
@ optio textus stdinFilename longum "stdin-filename" descriptio "Filename for error messages when reading stdin"
@ optio bivalens compact longum "compact" descriptio "Output compact JSON (no indentation)"
functio parse(
    si textus input,
    si textus output,
    si textus stdinFilename,
    si bivalens compact
) exitus code -> vacuum {
    fixum result = cede parseCommand(input, output, stdinFilename, compact)
    code = result
}

# ─────────────────────────────────────────────────────────────────────────────
# check - validate without codegen
# ─────────────────────────────────────────────────────────────────────────────

@ imperium "check"
@ alias "proba"
@ descriptio "Validate source without codegen (reads stdin if no input)"
@ futura
@ optio textus output brevis "o" longum "output" descriptio "Write error report to file"
@ optio textus include brevis "I" longum "include" descriptio "Add import search path"
@ optio textus manifest brevis "m" longum "manifest" descriptio "Project manifest file (faber.fab)"
@ optio textus stdinFilename longum "stdin-filename" descriptio "Filename for error messages when reading stdin"
@ optio bivalens json longum "json" descriptio "Output errors as JSON"
@ optio bivalens strict longum "strict" descriptio "Enable all strict checks"
functio check(
    si textus input,
    si textus output,
    si textus include,
    si textus manifest,
    si textus stdinFilename,
    si bivalens json,
    si bivalens strict
) exitus code -> vacuum {
    fixum result = cede checkCommand(input, output, include, manifest, stdinFilename, json, strict)
    code = result
}

# ─────────────────────────────────────────────────────────────────────────────
# test - compile and run tests with standalone harness
# ─────────────────────────────────────────────────────────────────────────────

@ imperium "test"
@ alias "probandum"
@ descriptio "Compile and run tests with standalone harness"
@ futura
@ optio textus target brevis "t" longum "target" descriptio "Target language: ts (default)"
@ optio textus tag longum "tag" descriptio "Run only tests with this tag"
@ optio textus exclude longum "exclude" descriptio "Exclude tests with this tag"
@ optio textus stdinFilename longum "stdin-filename" descriptio "Filename to use for stdin input"
@ optio bivalens json longum "json" descriptio "Output errors as JSON"
@ optio bivalens dryRun longum "dry-run" descriptio "Compile only, don't execute"
@ optio bivalens solo longum "only" descriptio "Run only tests marked @ solum"
functio test(
    textus input,
    si textus target,
    si textus tag,
    si textus exclude,
    si textus stdinFilename,
    si bivalens json,
    si bivalens dryRun,
    si bivalens solo
) exitus code -> vacuum {
    fixum result = cede testCommand(input, target, tag, exclude, stdinFilename, json, dryRun, solo)
    code = result
}

# ─────────────────────────────────────────────────────────────────────────────
# fetch - download and cache GitHub dependencies
# ─────────────────────────────────────────────────────────────────────────────

@ imperium "fetch"
@ descriptio "Download and cache GitHub dependencies from manifest"
@ futura
@ optio textus manifest brevis "m" longum "manifest" descriptio "Project manifest file (faber.fab)"
@ optio bivalens force brevis "f" longum "force" descriptio "Re-fetch even if cached"
@ optio bivalens json longum "json" descriptio "Output as JSON"
functio fetch(
    si textus manifest,
    si bivalens force,
    si bivalens json
) exitus code -> vacuum {
    fixum result = cede mandatumArcessere(manifest, force, json)
    code = result
}

# ─────────────────────────────────────────────────────────────────────────────
# run - execute build scripts from manifest
# ─────────────────────────────────────────────────────────────────────────────

@ imperium "run"
@ descriptio "Execute build scripts registered via § scriptum"
@ futura
@ optio textus manifest brevis "m" longum "manifest" descriptio "Project manifest file (default: faber.fab)"
@ optio bivalens list brevis "l" longum "list" descriptio "List available scripts"
@ optio bivalens json longum "json" descriptio "Output as JSON"
functio run(
    si textus script,
    si textus manifest,
    si bivalens list,
    si bivalens json
) exitus code -> vacuum {
    # TODO: arg forwarding - for now, pass empty list
    varia scriptArgs = [] innatum lista<textus>
    fixum result = cede mandatumCurrere(script, manifest, list, json, scriptArgs)
    code = result
}
```

## rumpe

### rumpe

```faber
# Break (rumpe) in loops
#
# rumpe exits the innermost loop immediately

incipit ergo cura arena {
    # Break when reaching 5
    varia i = 0

    dum i < 10 {
        si i == 5 {
            rumpe
        }
        scribe i
        i = i + 1
    }

    # Break in nested loop (exits inner only)
    varia outer = 0

    dum outer < 3 {
        varia inner = 0

        dum inner < 10 {
            si inner == 2 {
                rumpe
            }
            scribe "outer=§, inner=§", outer, inner
            inner = inner + 1
        }

        outer = outer + 1
    }
}
```

## scribe

### gradus

```faber
# Output statements with different log levels
#
# scribe <expr>  -> console.log (standard output)
# vide <expr>    -> console.debug (debug output)
# mone <expr>    -> console.warn (warning output)

incipit {
    fixum status = "running"
    fixum count = 42

    # Standard output (console.log)
    scribe "Application started"
    scribe "Status:", status

    # Debug output (console.debug)
    vide "Debug: entering main loop"
    vide "Debug: count =", count

    # Warning output (console.warn)
    mone "Warning: deprecated feature used"
    mone "Warning: count exceeds threshold:", count
}
```

### scribe

```faber
# Basic scribe (print) statements
#
# scribe <expr>
# scribe <expr>, <expr>, ...

incipit {
    # Simple string output
    scribe "Hello, world!"

    # Variable output
    fixum nomen = "Marcus"
    scribe nomen

    # Multiple arguments
    fixum aetas = 30
    scribe "Name:", nomen
    scribe "Age:", aetas

    # Expressions
    fixum x = 10
    fixum y = 20
    scribe "Sum:", x + y

    # Multiple values in one statement
    scribe "Coordinates:", x, y
}
```

## scriptum

### scriptum

```faber
# Format string expressions using scriptum()

incipit ergo cura arena {
    fixum name = "Marcus"
    fixum age = 30

    # Single placeholder
    fixum greeting = scriptum("Salve, §!", name)
    scribe greeting

    # Multiple placeholders
    fixum info = scriptum("§ is § years old", name, age)
    scribe info

    # With expression
    fixum calc = scriptum("10 + 20 = §", 10 + 20)
    scribe calc
}
```

## sed

### sed

```faber
# TODO
incipit {}
```

## si

### nidificatus

```faber
# Nested si conditionals

incipit {
    fixum isLoggedIn = verum
    fixum hasPermission = verum

    si isLoggedIn {
        si hasPermission {
            scribe "Access granted"
        }
        secus {
            scribe "Permission denied"
        }
    }
    secus {
        scribe "Please log in"
    }
}
```

### sin

```faber
# si-sin-secus (poetic if-else-if chain)
#
# Poetic alternative to si/sin/secus:
#   si   = if
#   sin  = else if ("but if")
#   secus = else ("otherwise")

incipit {
    fixum hour = 14

    si hour < 6 {
        scribe "Late night"
    }
    sin hour < 12 {
        scribe "Morning"
    }
    sin hour < 18 {
        scribe "Afternoon"
    }
    sin hour < 22 {
        scribe "Evening"
    }
    secus {
        scribe "Night"
    }
}
```

### ergo

```faber
# One-liner conditionals with ergo
#
# si <condition> ergo <statement>
# si <condition> ergo <statement> secus ergo <statement>
#
# ergo = "therefore, thus" (logical consequence)

incipit {
    fixum x = 10

    # Simple one-liner
    si x > 5 ergo scribe "x is big"

    # One-liner if-else
    fixum age = 25
    si age >= 18 ergo scribe "Adult" secus ergo scribe "Minor"

    # Multiple conditions
    fixum score = 85
    si score >= 90 ergo scribe "A" secus ergo scribe "Not A"

    # Simple validation
    fixum valid = verum
    si valid ergo scribe "OK"
}
```

### secus

```faber
# si-secus (if-else) conditionals
#
# si <condition> { <body> } secus { <body> }
# si <cond1> { } sin <cond2> { } secus { }

incipit {
    # Simple if-else
    fixum score = 85

    si score >= 90 {
        scribe "Grade: A"
    }
    secus {
        scribe "Grade: B or lower"
    }

    # Multiple statements in branches
    fixum temperature = 22

    si temperature > 30 {
        scribe "Hot"
        scribe "Stay hydrated"
    }
    secus {
        scribe "Comfortable"
        scribe "Enjoy the weather"
    }

    # If-else-if chain
    fixum grade = 75

    si grade >= 90 {
        scribe "A - Excellent"
    }
    sin grade >= 80 {
        scribe "B - Good"
    }
    sin grade >= 70 {
        scribe "C - Satisfactory"
    }
    sin grade >= 60 {
        scribe "D - Passing"
    }
    secus {
        scribe "F - Failing"
    }
}
```

### reddit

```faber
# Si with reddit syntax
#
# 'reddit' is syntactic sugar for 'ergo redde' - a one-liner return.
# Use it for early returns and guard clauses.
#
# si <condition> reddit <expression>
# sin <condition> reddit <expression>
# secus reddit <expression>

# Early return pattern
functio classify(numerus x) -> textus {
    si x < 0 reddit "negative"
    si x == 0 reddit "zero"
    redde "positive"
}

# Guard clause pattern
functio divide(numerus a, numerus b) -> si numerus {
    si b == 0 reddit nihil
    redde a / b
}

# Sin/secus chain with reddit
functio grade(numerus score) -> textus {
    si score >= 90 reddit "A"
    sin score >= 80 reddit "B"
    sin score >= 70 reddit "C"
    sin score >= 60 reddit "D"
    secus reddit "F"
}

# Find first in list (early return from loop)
functio findFirst(lista<numerus> items, numerus target) -> si numerus {
    itera ex items fixum item {
        si item == target reddit item
    }
    redde nihil
}

# Check if key exists (early return from iteration)
functio hasKey(tabula<textus, numerus> obj, textus key) -> bivalens {
    itera de obj fixum k {
        si k == key reddit verum
    }
    redde falsum
}

incipit {
    scribe classify(-5)   # negative
    scribe classify(0)    # zero
    scribe classify(10)   # positive

    scribe divide(10, 2)  # 5
    scribe divide(10, 0)  # nihil

    scribe grade(95)  # A
    scribe grade(85)  # B
    scribe grade(55)  # F

    fixum nums = [1, 2, 3, 4, 5]
    scribe findFirst(nums, 3)  # 3
    scribe findFirst(nums, 9)  # nihil
}
```

*2 more examples in this category*

## sparge

### sparge

```faber
# TODO
incipit {}
```

## sub

### sub

```faber
# TODO
incipit {}
```

## tempta

### tempta

```faber
# Basic tempta-cape (try-catch) with iace (throw)
#
# tempta { <body> }
# cape <errorName> { <handler> }
# iace <expression>

incipit {
    # Basic try-catch
    tempta {
        scribe "Attempting operation..."
        iace "Something went wrong"
        scribe "This line never runs"
    }
    cape err {
        scribe "Caught error:", err
    }

    # tempta-cape-demum (try-catch-finally)
    tempta {
        scribe "Opening resource..."
        iace "Failed to open"
    }
    cape err {
        scribe "Error occurred:", err
    }
    demum {
        scribe "Cleanup: always runs"
    }

    # demum without cape
    tempta {
        scribe "Operation succeeds"
    }
    demum {
        scribe "Cleanup runs anyway"
    }

    # Nested tempta blocks
    tempta {
        scribe "Outer try"

        tempta {
            scribe "Inner try"
            iace "Inner error"
        }
        cape inner {
            scribe "Caught inner:", inner
        }

        scribe "Continues after inner catch"
    }
    cape outer {
        scribe "Outer catch:", outer
    }
}
```

### in-functione

```faber
# Error handling in functions
#
# Functions can use tempta-cape to handle errors internally
# or let errors propagate to callers

functio safeDivide(numerus a, numerus b) -> numerus {
    tempta {
        si b == 0 {
            iace "Division by zero"
        }
        redde a / b
    }
    cape err {
        scribe "Error:", err
        redde 0
    }
}

functio validatePositive(numerus value) -> numerus {
    tempta {
        si value < 0 {
            iace "Negative value not allowed"
        }
        redde value * 2
    }
    cape err {
        scribe "Validation failed:", err
        redde 0
    }
}

functio processWithCleanup(textus name) {
    varia resource = "pending"

    tempta {
        scribe "Opening:", name
        resource = name

        si name == "" {
            iace "Empty name"
        }

        scribe "Processing:", resource
    }
    cape err {
        scribe "Error:", err
    }
    demum {
        scribe "Closing:", resource
    }
}

functio withReturnInDemum() -> textus {
    tempta {
        scribe "Starting operation"
        redde "success"
    }
    cape err {
        redde "error"
    }
    demum {
        scribe "Demum runs before return"
    }
}

incipit {
    scribe "Safe divide 10/2:", safeDivide(10, 2)
    scribe "Safe divide 10/0:", safeDivide(10, 0)

    scribe "Validate 5:", validatePositive(5)
    scribe "Validate -3:", validatePositive(-3)

    processWithCleanup("data.txt")
    processWithCleanup("")

    scribe "Result:", withReturnInDemum()
}
```

## ternarius

### ternarius

```faber
# Ternary conditional expressions
#
# Two syntaxes (equivalent):
#   condition ? then : else
#   condition sic then secus else
#
# Latin: sic = "thus/so", secus = "otherwise"

incipit {
    fixum age = 25

    # Latin ternary
    fixum statusLatin = age >= 18 sic "adult" secus "minor"
    scribe statusLatin  # "adult"

    # Latin nested (reads more clearly)
    fixum score = 85
    fixum gradeLatin =
        score >= 90 sic "A" secus
        score >= 80 sic "B" secus
        score >= 70 sic "C" secus "F"
    scribe gradeLatin  # "B"

    # In expressions
    fixum max = age > 30 sic age secus 30
    scribe "Max:", max

    # With nullish values
    varia maybe = nihil qua si textus
    fixum result = nonnihil maybe sic maybe secus "default"
    scribe result  # "default"

    # Prefer sic/secus for clarity
    fixum readable = age >= 21 sic "can drink" secus "cannot drink"
    scribe readable
}
```

## typus

### typus

```faber
# Basic type aliases
#
# typus Name = Type

# Primitive type aliases
typus UserId = numerus
typus Username = textus
typus IsActive = bivalens

# Generic type aliases
typus Names = lista<textus>
typus Scores = lista<numerus>
typus UserCache = tabula<textus, numerus>

# Nullable type alias
typus OptionalName = si textus

incipit {
    # Using the type aliases
    fixum UserId id = 42
    fixum Username name = "Marcus"
    fixum IsActive active = verum

    scribe id
    scribe name
    scribe active

    # Using generic type aliases
    fixum Names friends = ["Gaius", "Lucius", "Titus"]
    scribe friends

    fixum Scores points = [100, 95, 87]
    scribe points
}
```

## unarius

### unarius

```faber
# Unary operators: negation, logical not, and null/sign checks

incipit {
    # Numeric negation
    fixum x = 5
    fixum neg = -x
    scribe neg  # -5

    # Logical negation
    fixum flag = verum
    fixum notFlag = non flag
    scribe notFlag  # falsum

    # Positive/negative checks
    fixum a = 10
    fixum b = -3
    scribe positivum a   # verum
    scribe negativum a   # falsum
    scribe positivum b   # falsum
    scribe negativum b   # verum

    # Null checks
    fixum si textus maybe = nihil
    scribe nulla maybe      # verum (is null)
    scribe nonnulla maybe   # falsum (is not null)

    fixum si textus present = "salve"
    scribe nulla present    # falsum
    scribe nonnulla present # verum
}
```

## usque

### usque

```faber
# TODO
incipit {}
```

## varia

### typicus

```faber
# Typed variable declarations
#
# varia <type> <name> = <expr>   -- typed mutable
# fixum <type> <name> = <expr>   -- typed immutable

incipit {
    # Typed immutable declarations
    fixum numerus age = 30
    fixum textus name = "Marcus"
    fixum bivalens active = verum

    scribe age
    scribe name
    scribe active

    # Typed mutable declarations
    varia numerus count = 0
    varia textus status = "pending"
    varia bivalens running = falsum

    scribe count
    scribe status
    scribe running

    # Reassign mutable typed variables
    count = 100
    status = "complete"
    running = verum

    scribe count
    scribe status
    scribe running

    # Fractional numbers
    fixum fractus pi = 3.14159
    varia fractus rate = 0.05

    scribe pi
    scribe rate

    rate = 0.10
    scribe rate
}
```

### destructura

```faber
# Array destructuring declarations
#
# fixum [a, b, c] = array   -- destructure into immutable bindings
# varia [x, y, z] = array   -- destructure into mutable bindings

incipit {
    # Basic array destructuring
    fixum numbers = [1, 2, 3]
    fixum [a, b, c] = numbers

    scribe a
    scribe b
    scribe c

    # Destructure inline array
    fixum [first, second, third] = [10, 20, 30]

    scribe first
    scribe second
    scribe third

    # Mutable destructuring
    fixum coords = [100, 200]
    varia [x, y] = coords

    scribe x
    scribe y

    x = x + 50
    y = y + 50

    scribe x
    scribe y

    # Destructure with fewer variables (partial)
    fixum values = [1, 2, 3, 4, 5]
    fixum [one, two] = values

    scribe one
    scribe two

    # Nested arrays
    fixum matrix = [[1, 2], [3, 4]]
    fixum [row1, row2] = matrix

    scribe row1
    scribe row2
}
```

### varia

```faber
# Basic varia and fixum declarations
#
# varia <name> = <expr>   -- mutable binding
# fixum <name> = <expr>   -- immutable binding

incipit {
    # Mutable variable with varia
    varia counter = 0
    scribe counter

    counter = 1
    scribe counter

    counter = counter + 10
    scribe counter

    # Immutable variable with fixum
    fixum greeting = "Salve, Mundus!"
    scribe greeting

    # Multiple declarations
    fixum x = 10
    fixum y = 20
    fixum sum = x + y
    scribe sum

    # Mutable reassignment
    varia message = "Hello"
    message = "Goodbye"
    scribe message
}
```

## variandum

### variandum

```faber
# TODO
incipit {}
```

## vel

### vel

```faber
# Nullish coalescing with vel
#
# vel (??) returns right side only if left is nihil
# Different from logical || which triggers on all falsy values
#
# Syntax:
#   value vel default
#   value ?? default

incipit {
    # Basic nullish coalescing
    fixum si textus name = nihil
    fixum display = name vel "Anonymous"
    scribe display  # "Anonymous"

    # With non-null value
    fixum si textus actualName = "Marcus"
    fixum displayName = actualName vel "Anonymous"
    scribe displayName  # "Marcus"

    # Difference from logical or: vel only checks nihil
    fixum zero = 0
    fixum emptyStr = ""
    fixum falseBool = falsum

    # vel preserves falsy non-null values
    scribe zero vel 999        # 0 (not nihil)
    scribe emptyStr vel "def"  # "" (not nihil)
    scribe falseBool vel verum # falsum (not nihil)

    # Symbolic syntax (equivalent)
    fixum result2 = name vel "Default"
    scribe result2

    # Chaining
    fixum si textus a = nihil
    fixum si textus b = nihil
    fixum si textus c = "found"
    fixum first = a vel b vel c vel "none"
    scribe first  # "found"

    # With conversion operators
    fixum input = "abc"
    fixum num = input numeratum vel 0
    scribe num  # 0 (parse failed, uses default)

    fixum validInput = "42"
    fixum validNum = validInput numeratum vel 0
    scribe validNum  # 42
}
```

## vide

### vide

```faber
# TODO
incipit {}
```

## vocatio

### vocatio

```faber
# Call expressions: simple, with arguments, method calls, chaining, spread

functio greet() {
    scribe "Salve!"
}

functio add(numerus a, numerus b) -> numerus {
    redde a + b
}

functio multiply(numerus x, numerus y) -> numerus {
    redde x * y
}

genus Calculator {
    numerus value: 0

    functio setValue(numerus n) -> Calculator {
        ego.value = n
        redde ego
    }

    functio double() -> Calculator {
        ego.value = ego.value * 2
        redde ego
    }

    functio getResult() -> numerus {
        redde ego.value
    }
}

incipit ergo cura arena fixum alloc {
    # Simple call (no arguments)
    greet()

    # Call with arguments
    fixum sum = add(10, 20)
    scribe sum  # 30

    # Multiple arguments
    fixum product = multiply(5, 6)
    scribe product  # 30

    # Method call on object
    varia calc = novum Calculator()
    calc.setValue(10)
    scribe calc.getResult()  # 10

    # Chained method calls
    varia calc2 = novum Calculator()
    fixum result = calc2.setValue(5).double().double().getResult()
    scribe result  # 20

    # Call with spread operator
    fixum numerus[] numbers = [3, 7]
    fixum spreadSum = add(sparge numbers)
    scribe spreadSum  # 10
}
```

