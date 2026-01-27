"""Lexer for Faber source code."""

from errors import Locus, CompileError
from nodes import Token, TokenTag


KEYWORDS = frozenset([
    # Declarations
    "varia", "fixum", "figendum", "variandum",
    "functio", "genus", "pactum", "ordo", "discretio", "typus",
    "ex", "importa", "ut",
    # Modifiers
    "publica", "privata", "protecta", "generis", "implet", "sub", "abstractus",
    # Control flow
    "si", "sin", "secus", "dum", "fac", "elige", "casu", "ceterum", "discerne", "custodi",
    "de", "itera", "in", "pro", "omnia",
    # Actions
    "redde", "reddit", "rumpe", "perge", "iace", "mori", "tempta", "cape", "demum",
    "scribe", "vide", "mone", "adfirma", "tacet",
    # Expressions
    "cede", "novum", "clausura", "qua", "innatum", "finge",
    "sic", "scriptum",
    # Operators (word-form)
    "et", "aut", "vel", "inter", "intra",
    "non", "nihil", "nonnihil", "positivum", "negativum", "nulla", "nonnulla",
    # Conversion operators
    "numeratum", "fractatum", "textatum", "bivalentum",
    # Literals
    "verum", "falsum", "ego",
    # Entry
    "incipit", "incipiet",
    # Test
    "probandum", "proba",
    # Type
    "usque", "ante",
    # Annotations
    "publicum", "externa",
])

PUNCTUATORS = frozenset("(){}[],.;:@#?!")

OPERATORS = [
    # Multi-char first (greedy match)
    "===", "!==", "==", "!=", "<=", ">=", "&&", "||", "??",
    "+=", "-=", "*=", "/=",
    "->", "..",
    # Single-char
    "+", "-", "*", "/", "%",
    "<", ">", "=",
    "&", "|", "^", "~",
]


def lex(source: str, filename: str = "<stdin>") -> list[Token]:
    """Convert source text into tokens."""
    tokens: list[Token] = []
    pos = 0
    linea = 1
    line_start = 0
    length = len(source)

    def locus() -> Locus:
        return Locus(linea=linea, columna=pos - line_start + 1, index=pos)

    def peek(offset: int = 0) -> str:
        idx = pos + offset
        return source[idx] if idx < length else ""

    def advance() -> str:
        nonlocal pos, linea, line_start
        ch = source[pos]
        pos += 1
        if ch == "\n":
            linea += 1
            line_start = pos
        return ch

    def match(s: str) -> bool:
        nonlocal pos, linea, line_start
        if source[pos:].startswith(s):
            for _ in s:
                advance()
            return True
        return False

    def skip_whitespace():
        nonlocal pos
        while pos < length:
            ch = peek()
            if ch in " \t\r":
                advance()
            elif ch == "\n":
                loc = locus()
                advance()
                tokens.append(Token(TokenTag.NEWLINE, "\n", loc))
            else:
                break

    def read_string(quote: str) -> str:
        result: list[str] = []
        advance()  # skip opening quote
        while pos < length and peek() != quote:
            if peek() == "\\":
                advance()
                esc = advance()
                if esc == "n":
                    result.append("\n")
                elif esc == "t":
                    result.append("\t")
                elif esc == "r":
                    result.append("\r")
                elif esc == "\\":
                    result.append("\\")
                elif esc == '"':
                    result.append('"')
                elif esc == "'":
                    result.append("'")
                else:
                    result.append(esc)
            else:
                result.append(advance())
        advance()  # skip closing quote
        return "".join(result)

    def read_triple_string() -> str:
        advance()  # skip first "
        advance()  # skip second "
        advance()  # skip third "
        if peek() == "\n":
            advance()
        result: list[str] = []
        while pos < length:
            if peek() == '"' and peek(1) == '"' and peek(2) == '"':
                value = "".join(result)
                if value.endswith("\n"):
                    value = value[:-1]
                advance()
                advance()
                advance()
                return value
            result.append(advance())
        return "".join(result)

    def read_number() -> str:
        result: list[str] = []
        while pos < length and is_number_char(peek()):
            result.append(advance())
        return "".join(result)

    def read_identifier() -> str:
        result: list[str] = []
        while pos < length and is_ident_char(peek()):
            result.append(advance())
        return "".join(result)

    def read_comment() -> str:
        result: list[str] = []
        advance()  # skip #
        while pos < length and peek() != "\n":
            result.append(advance())
        return "".join(result)

    while pos < length:
        skip_whitespace()
        if pos >= length:
            break

        loc = locus()
        ch = peek()

        # Comment
        if ch == "#":
            value = read_comment()
            tokens.append(Token(TokenTag.COMMENT, value, loc))
            continue

        # Triple-quoted string
        if ch == '"' and peek(1) == '"' and peek(2) == '"':
            value = read_triple_string()
            tokens.append(Token(TokenTag.TEXTUS, value, loc))
            continue

        # String
        if ch in '"\'':
            value = read_string(ch)
            tokens.append(Token(TokenTag.TEXTUS, value, loc))
            continue

        # Number
        if is_digit(ch):
            value = read_number()
            tokens.append(Token(TokenTag.NUMERUS, value, loc))
            continue

        # Identifier or keyword
        if is_alpha(ch) or ch == "_":
            value = read_identifier()
            tag = TokenTag.KEYWORD if value in KEYWORDS else TokenTag.IDENTIFIER
            tokens.append(Token(tag, value, loc))
            continue

        # Operators (multi-char first)
        matched = False
        for op in OPERATORS:
            if match(op):
                tokens.append(Token(TokenTag.OPERATOR, op, loc))
                matched = True
                break
        if matched:
            continue

        # Section sign (multi-byte UTF-8)
        if match("§"):
            tokens.append(Token(TokenTag.PUNCTUATOR, "§", loc))
            continue

        # Punctuators
        if ch in PUNCTUATORS:
            advance()
            tokens.append(Token(TokenTag.PUNCTUATOR, ch, loc))
            continue

        raise CompileError(f"unexpected character '{ch}'", loc, filename)

    tokens.append(Token(TokenTag.EOF, "", locus()))
    return tokens


def is_digit(ch: str) -> bool:
    return ch.isdigit()


def is_alpha(ch: str) -> bool:
    return ch.isalpha()


def is_ident_char(ch: str) -> bool:
    return ch.isalnum() or ch == "_"


def is_number_char(ch: str) -> bool:
    return ch.isdigit() or ch in "._"


def prepare(tokens: list[Token]) -> list[Token]:
    """Prepare token stream by filtering comments and newlines."""
    return [tok for tok in tokens if tok.tag not in (TokenTag.COMMENT, TokenTag.NEWLINE)]
