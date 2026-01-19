package main

import (
	"encoding/json"
	"fmt"
	"io"
	"os"

	nanus "nanus-go"
)

func showHelp() {
	fmt.Println("nanus-go - Minimal Faber compiler (stdin/stdout)")
	fmt.Println("")
	fmt.Println("Usage: <source> | nanus-go <command> [options]")
	fmt.Println("")
	fmt.Println("Commands:")
	fmt.Println("  emit     Compile to output format")
	fmt.Println("  parse    Output AST as JSON")
	fmt.Println("  lex      Output tokens as JSON")
	fmt.Println("")
	fmt.Println("Options:")
	fmt.Println("  -i <format>  Input format: faber (default), fg (Faber Glyph)")
	fmt.Println("  -f <format>  Output format: ts (TypeScript, default), fg (Faber Glyph)")
}

func main() {
	args := os.Args[1:]

	if len(args) == 0 || args[0] == "-h" || args[0] == "--help" {
		showHelp()
		os.Exit(0)
	}

	command := args[0]
	validCommands := map[string]struct{}{"emit": {}, "parse": {}, "lex": {}}
	if _, ok := validCommands[command]; !ok {
		fmt.Fprintf(os.Stderr, "Unknown command: %s\n", command)
		os.Exit(1)
	}

	inputFormat := "faber"
	outputFormat := "ts"
	for i := 1; i < len(args); i++ {
		if args[i] == "-i" && i+1 < len(args) {
			inputFormat = args[i+1]
			i++
		} else if args[i] == "-f" && i+1 < len(args) {
			outputFormat = args[i+1]
			i++
		}
	}

	source, err := io.ReadAll(os.Stdin)
	if err != nil {
		fmt.Fprintln(os.Stderr, err.Error())
		os.Exit(1)
	}

	defer func() {
		if r := recover(); r != nil {
			fmt.Fprintln(os.Stderr, nanus.FormatError(r, string(source), "<stdin>"))
			os.Exit(1)
		}
	}()

	switch command {
	case "lex":
		var tokens []nanus.Token
		if inputFormat == "fg" {
			tokens = nanus.LexFG(string(source), "<stdin>")
		} else {
			tokens = nanus.Lex(string(source), "<stdin>")
		}
		out, err := json.MarshalIndent(tokens, "", "  ")
		if err != nil {
			fmt.Fprintln(os.Stderr, err.Error())
			os.Exit(1)
		}
		fmt.Println(string(out))
	case "parse":
		var tokens []nanus.Token
		if inputFormat == "fg" {
			tokens = nanus.Prepare(nanus.LexFG(string(source), "<stdin>"))
		} else {
			tokens = nanus.Prepare(nanus.Lex(string(source), "<stdin>"))
		}
		ast := nanus.Parse(tokens, "<stdin>")
		out, err := json.MarshalIndent(ast, "", "  ")
		if err != nil {
			fmt.Fprintln(os.Stderr, err.Error())
			os.Exit(1)
		}
		fmt.Println(string(out))
	default:
		result := nanus.Compile(string(source), &nanus.CompileOptions{InputFormat: inputFormat, Format: outputFormat})
		if !result.Success {
			fmt.Fprintln(os.Stderr, result.Error)
			os.Exit(1)
		}
		fmt.Println(result.Output)
	}
}
