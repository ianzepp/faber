package main

import (
	"encoding/json"
	"fmt"
	"io"
	"os"

	"subsidia"
)

func main() {
	if len(os.Args) < 2 || os.Args[1] == "-h" || os.Args[1] == "--help" {
		printUsage()
		os.Exit(0)
	}

	command := os.Args[1]
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
			fmt.Fprintln(os.Stderr, subsidia.FormatError(r, string(source), "<stdin>"))
			os.Exit(1)
		}
	}()

	switch command {
	case "lex":
		tokens := Lex(string(source), "<stdin>")
		out, err := json.MarshalIndent(tokens, "", "  ")
		if err != nil {
			fmt.Fprintln(os.Stderr, err.Error())
			os.Exit(1)
		}
		fmt.Println(string(out))
	case "parse":
		tokens := subsidia.Prepare(Lex(string(source), "<stdin>"))
		ast := subsidia.Parse(tokens, "<stdin>")
		out, err := json.MarshalIndent(ast, "", "  ")
		if err != nil {
			fmt.Fprintln(os.Stderr, err.Error())
			os.Exit(1)
		}
		fmt.Println(string(out))
	case "emit":
		tokens := subsidia.Prepare(Lex(string(source), "<stdin>"))
		ast := subsidia.Parse(tokens, "<stdin>")
		fmt.Println(EmitTS(ast))
	}
}

func printUsage() {
	fmt.Println("nanus-go: Faber to TypeScript compiler (stdin/stdout)")
	fmt.Println()
	fmt.Println("Usage: <source> | nanus-go <command>")
	fmt.Println()
	fmt.Println("Commands:")
	fmt.Println("  emit     Compile Faber to TypeScript")
	fmt.Println("  parse    Output AST as JSON")
	fmt.Println("  lex      Output tokens as JSON")
}
