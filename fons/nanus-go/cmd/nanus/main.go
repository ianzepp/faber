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
	fmt.Println("Usage: <source> | nanus-go <command>")
	fmt.Println("")
	fmt.Println("Commands:")
	fmt.Println("  emit     Compile to TypeScript")
	fmt.Println("  parse    Output AST as JSON")
	fmt.Println("  lex      Output tokens as JSON")
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
		tokens := nanus.Lex(string(source), "<stdin>")
		out, err := json.MarshalIndent(tokens, "", "  ")
		if err != nil {
			fmt.Fprintln(os.Stderr, err.Error())
			os.Exit(1)
		}
		fmt.Println(string(out))
	case "parse":
		tokens := nanus.Prepare(nanus.Lex(string(source), "<stdin>"))
		ast := nanus.Parse(tokens, "<stdin>")
		out, err := json.MarshalIndent(ast, "", "  ")
		if err != nil {
			fmt.Fprintln(os.Stderr, err.Error())
			os.Exit(1)
		}
		fmt.Println(string(out))
	default:
		result := nanus.Compile(string(source), nil)
		if !result.Success {
			fmt.Fprintln(os.Stderr, result.Error)
			os.Exit(1)
		}
		fmt.Println(result.Output)
	}
}
