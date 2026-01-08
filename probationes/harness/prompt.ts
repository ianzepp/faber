// Prompt construction for different task types

import type { Task, Example } from './types'

export function buildVerificationPrompt(originalPrompt: string, drafterResponse: string): string {
  return `${originalPrompt}

---

A previous model produced this response:

\`\`\`
${drafterResponse}
\`\`\`

Review this code. If it is correct, output it unchanged. If it contains errors, provide the corrected version.

Output only the code, no explanation.`
}

export function buildPrompt(
  task: Task,
  examples: Example[],
  nShot: number,
  dialect: string,
  context: string = 'minimal'
): string {
  const selectedExamples = examples.slice(0, nShot)

  let prompt = buildSystemPrompt(dialect, context)

  if (nShot > 0) {
    prompt += '\n\n## Examples\n\n'
    for (const ex of selectedExamples) {
      prompt += formatExample(ex) + '\n\n'
    }
  }

  prompt += '\n\n## Task\n\n'
  prompt += formatTask(task)

  return prompt
}

function buildSystemPrompt(dialect: string, context: string = 'minimal'): string {
  if (dialect === 'latin') {
    const base = `You are learning a new programming language called Faber. It uses Latin-inspired keywords but follows familiar programming patterns.

`

    if (context === 'examples-only') {
      return `You are learning a new programming language called Faber by studying examples.

Your task is to translate between TypeScript and Faber, or to understand Faber code.`
    }

    if (context === 'grammar-only') {
      return `You are learning a new programming language called Faber. Here is its formal grammar in EBNF:

## Types
typeAnnotation := IDENTIFIER typeParams? '?'? arrayBrackets*
typeParams := '<' typeParameter (',' typeParameter)* '>'
// Array types: numerus[] or lista<numerus> (both valid)

## Variables
varDecl := ('varia' | 'fixum') typeAnnotation? IDENTIFIER ('=' expression)?

## Functions
funcDecl := 'functio' IDENTIFIER '(' paramList ')' returnClause? blockStmt
paramList := (parameter (',' parameter)*)?
parameter := typeAnnotation IDENTIFIER ('vel' expression)?
returnClause := '->' typeAnnotation
lambdaExpr := 'pro' IDENTIFIER ':' expression

## Control Flow
ifStmt := 'si' expression blockStmt ('sin' expression blockStmt)* ('secus' blockStmt)?
// sin = else-if (NOT else), secus = else
whileStmt := 'dum' expression blockStmt
forStmt := 'ex' expression 'pro' IDENTIFIER blockStmt  // ex items pro item { } (no fixum/varia)
returnStmt := 'redde' expression?

## Error Handling
tryStmt := 'tempta' blockStmt ('cape' IDENTIFIER blockStmt)?
throwStmt := 'iace' expression

## Classes
genusDecl := 'genus' IDENTIFIER '{' genusMember* '}'
fieldDecl := typeAnnotation IDENTIFIER
methodDecl := 'functio' IDENTIFIER '(' paramList ')' returnClause? blockStmt?

## Operators
or := and ('aut' and | '||' and)*
and := equality ('et' equality | '&&' equality)*
unary := ('!' | '-' | 'non') unary | call
// Comparison: == != < > <= >= (no === in Faber, use ==)

## Keyword Mappings
Types: textus=string, numerus=number, bivalens=boolean, nihil=null, vacuum=void
Decl: fixum=const, varia=let, functio=function, genus=class
Flow: si/sin/secus=if/else-if/else, dum=while, ex...pro=for-of, redde=return
Bool: verum=true, falsum=false, et=and, aut=or, non=not
Error: tempta=try, cape=catch, iace=throw
Output: scribe=console.log
Object: ego=this, novum=new

## Collection Methods (called as methods, not properties)
Array: arr.longitudo() for length, arr.adde(x) for push, arr.mappata(fn), arr.filtrata(fn)
Math: pavimentum(x) for Math.floor, tectum(x) for Math.ceil, absolutum(x) for Math.abs

Your task is to translate between TypeScript and Faber, or to understand Faber code.`
    }

    if (context === 'minimal') {
      return base + `Key vocabulary:
- fixum = const (fixum x = 5)
- varia = let (varia x = 5)
- functio = function, with type-first params: functio name(numerus x) -> numerus
- redde = return
- si/secus = if/else (no parentheses: si x > 0 { })
- dum = while
- ex...pro = for...of (ex items pro item { })
- scribe = console.log (no parentheses needed: scribe x)
- verum/falsum = true/false
- nihil = null
- numerus = number
- textus = string
- bivalens = boolean

IMPORTANT syntax rules:
- Type-first parameters: functio f(numerus x) NOT functio f(x: numerus)
- Type-first declarations: fixum textus name NOT fixum name: textus
- For-of loops: ex collection pro item { } (collection first, then binding)

Your task is to translate between TypeScript and Faber, or to understand Faber code.`
    }

    if (context === 'basic') {
      return base + `## Quick Reference

### Types
textus (string), numerus (int), fractus (float), bivalens (bool), nihil (null), vacuum (void)
lista<T> (array), tabula<K,V> (map), copia<T> (set), promissum<T> (promise)

### Literals
verum (true), falsum (false), nihil (null), ego (this/self)

### Keywords

**Declarations:** fixum (const), varia (let), functio (function), genus (class), pactum (interface), typus (type), ordo (enum)

**Control flow:** si/sin/secus (if/else-if/else), dum (while), ex...pro (for-of), de...pro (for-in), elige (switch), custodi (guard), rumpe (break), perge (continue)

**Functions:** redde (return), futura (async), cede (await), cursor (generator), pro x: expr (lambda)

**Error handling:** tempta (try), cape (catch), demum (finally), iace (throw), mori (panic), adfirma (assert)

**Output:** scribe (console.log), vide (console.debug), mone (console.warn) - parentheses optional

**Operators:** et (&&), aut (||), non (!), vel (??), est (instanceof), qua (type cast as)

### IMPORTANT Syntax Rules
- Type-first parameters: functio f(numerus x) NOT functio f(x: numerus)
- Type-first declarations: fixum textus name NOT fixum name: textus
- For-of loops: ex collection pro item { } (collection first, then binding)
- No parentheses around if/while conditions: si x > 0 { } NOT si (x > 0) { }

Your task is to translate between TypeScript and Faber, or to understand Faber code.`
    }

    if (context === 'complete') {
      return base + `## Complete Grammar Reference

### Types
textus (string), numerus (int), fractus (float), decimus (decimal), magnus (bigint), bivalens (bool)
nihil (null), vacuum (void), numquam (never), ignotum (unknown), octeti (bytes), objectum (object)
lista<T> (array), tabula<K,V> (map), copia<T> (set), promissum<T> (promise), cursor<T> (iterator), unio<A,B> (union)

### Return Type Verbs (encode async/generator semantics)
fit "it becomes" - sync single value
fiet "it will become" - async single value (Promise)
fiunt "they become" - sync generator
fient "they will become" - async generator

### Block Syntax Patterns
ex...pro - iterate values: ex items pro item { }
ex...fiet - async iterate: ex items fiet item { }
de...pro - iterate keys: de table pro key { }
cura...fit - resource scope: cura expr fit name { }
tempta...cape - error handling: tempta { } cape err { }
custodi - guard clauses: custodi { si expr { } }
elige - switch: elige expr { casu val { } ceterum { } }
discerne - pattern match: discerne expr { casu Variant pro x { } }

### Declarations (IMPORTANT: type-first syntax)
fixum x = 5 - immutable binding (inferred type)
fixum textus name = "Alice" - with explicit type (type before name)
varia x = 5 - mutable binding
functio name(textus arg) -> textus { redde "result" } - type before param name
genus Class { textus field } - type before field name
pactum Interface { functio method() -> vacuum }
typus Alias = textus
ordo Enum { A, B, C }

### Control Flow (no parentheses around conditions)
si x > 0 { } secus { }
si x > 0 { } sin x < 0 { } secus { }
dum x > 0 { x = x - 1 }
ex [1,2,3] pro n { scribe n } - collection first, then binding
de object pro key { scribe key }
custodi { si x == nihil reddit nihil }
elige status { casu "ok" { } ceterum { } }

### Error Handling
tempta { riskyOp() } cape err { scribe err } demum { cleanup() }
iace "error message"
mori "panic message"
adfirma x > 0, "must be positive"

### Operators
Arithmetic: + - * / %
Comparison: == != === !== < > <= >=
Logical: et (&&), aut (||), non (!), vel (??)
Special: est (instanceof), qua (type cast), novum (new), cede (await)

### Collection Methods
lista: adde(x), remove(), primus, ultimus, longitudo, mappata(fn), filtrata(fn), reducta(fn,init), inveni(fn), continet(x)
tabula: pone(k,v), accipe(k), habet(k), dele(k), claves(), valores()

### CRITICAL Syntax Rules (NOT TypeScript)
- Type-first parameters: functio f(numerus x) NOT functio f(x: numerus)
- Type-first declarations: fixum textus name NOT fixum name: textus
- For-of loops: ex collection pro item { } (collection first)
- No parentheses: si x > 0 { } NOT si (x > 0) { }
- Output: scribe x (parentheses optional)

Your task is to translate between TypeScript and Faber, or to understand Faber code.`
    }
  }

  // TODO: English and symbols dialects
  return ''
}

function formatExample(ex: Example): string {
  return `TypeScript:
\`\`\`typescript
${ex.typescript}
\`\`\`

Faber:
\`\`\`faber
${ex.faber}
\`\`\`

Note: ${ex.explanation}`
}

function formatTask(task: Task): string {
  const goalLine = task.goal ? `Goal: ${task.goal}\n\n` : ''

  switch (task.type) {
    case 'translate_ts_to_faber':
      return `${goalLine}Translate this TypeScript to Faber:

\`\`\`typescript
${task.input}
\`\`\`

Provide only the Faber code, no explanation.`

    case 'translate_faber_to_ts':
      const faberCode = task.input_faber || ''

      return `${goalLine}Translate this Faber to TypeScript:

\`\`\`faber
${faberCode}
\`\`\`

Provide only the TypeScript code, no explanation.`

    case 'predict_output':
      return `${goalLine}What does this Faber code output?

\`\`\`faber
${task.input_faber}
\`\`\`

Provide only the output, no explanation.`

    case 'complete_code':
      const template = task.input || task.input_faber || ''
      return `${goalLine}Fill in the blank (___) in this Faber code:

\`\`\`faber
${template}
\`\`\`

Provide only the missing keyword or expression that replaces ___, no explanation.`

    case 'write_typescript':
      return `${goalLine}

Write the TypeScript code. Provide only the code, no explanation.`

    case 'write_faber':
      return `${goalLine}

Write the Faber code. Provide only the code, no explanation.`

    default:
      return ''
  }
}
