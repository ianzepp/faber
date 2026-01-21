"""CLI entry point for nanus-py: Faber microcompiler."""

import argparse
import json
import sys
from dataclasses import asdict

from lexer import lex, prepare
from parser import parse
from emitter_faber import emit_faber
from emitter_py import emit_py
from errors import CompileError, format_error


def main():
    parser = argparse.ArgumentParser(
        prog="nanus-py",
        description="Faber microcompiler (stdin/stdout)",
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    lex_parser = subparsers.add_parser("lex", help="Output tokens as JSON")
    lex_parser.add_argument("-f", "--file", help="Input file (default: stdin)")

    parse_parser = subparsers.add_parser("parse", help="Output AST as JSON")
    parse_parser.add_argument("-f", "--file", help="Input file (default: stdin)")

    emit_parser = subparsers.add_parser("emit", help="Compile Faber to target language")
    emit_parser.add_argument(
        "-t", "--target",
        choices=["fab", "py"],
        default="py",
        help="Output target: fab, py (default: py)",
    )
    emit_parser.add_argument("-f", "--file", help="Input file (default: stdin)")

    args = parser.parse_args()

    if args.file:
        with open(args.file, "r") as f:
            source = f.read()
        filename = args.file
    else:
        source = sys.stdin.read()
        filename = "<stdin>"

    try:
        if args.command == "lex":
            tokens = lex(source, filename)
            output = [token_to_dict(t) for t in tokens]
            print(json.dumps(output, indent=2))

        elif args.command == "parse":
            tokens = prepare(lex(source, filename))
            ast = parse(tokens, filename)
            output = module_to_dict(ast)
            print(json.dumps(output, indent=2))

        elif args.command == "emit":
            tokens = prepare(lex(source, filename))
            ast = parse(tokens, filename)

            if args.target == "fab":
                print(emit_faber(ast))
            elif args.target == "py":
                print(emit_py(ast))

    except CompileError as e:
        print(format_error(e, source, filename), file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)


def token_to_dict(token):
    """Convert a Token to a serializable dict."""
    return {
        "tag": token.tag.value,
        "value": token.valor,
        "locus": {
            "linea": token.locus.linea,
            "columna": token.locus.columna,
            "index": token.locus.index,
        },
    }


def module_to_dict(mod):
    """Convert a Modulus AST to a serializable dict."""
    return {
        "_type": "Modulus",
        "corpus": [stmt_to_dict(s) for s in mod.corpus],
    }


def stmt_to_dict(stmt):
    """Convert a statement to a dict, handling AST node types."""
    if stmt is None:
        return None

    result = {"_type": type(stmt).__name__}

    if hasattr(stmt, "__dataclass_fields__"):
        for field_name in stmt.__dataclass_fields__:
            value = getattr(stmt, field_name)
            result[field_name] = value_to_dict(value)
    else:
        result["_value"] = str(stmt)

    return result


def value_to_dict(value):
    """Recursively convert AST values to dicts."""
    if value is None:
        return None
    if isinstance(value, (str, int, float, bool)):
        return value
    if isinstance(value, list):
        return [value_to_dict(v) for v in value]
    if hasattr(value, "__dataclass_fields__"):
        return stmt_to_dict(value)
    if hasattr(value, "value"):
        return value.value
    return str(value)


if __name__ == "__main__":
    main()
