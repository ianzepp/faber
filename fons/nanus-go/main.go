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

	// Parse flags for emit command
	target := "go"   // default to Go
	pkg := "main"    // default package name
	for i := 2; i < len(os.Args); i++ {
		if os.Args[i] == "-t" && i+1 < len(os.Args) {
			target = os.Args[i+1]
			i++
		} else if os.Args[i] == "-p" && i+1 < len(os.Args) {
			pkg = os.Args[i+1]
			i++
		}
	}

	// Validate target
	if command == "emit" && target != "go" && target != "fab" {
		fmt.Fprintf(os.Stderr, "Unknown target: %s. Valid: go, fab\n", target)
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
		switch target {
		case "go":
			fmt.Println(EmitGo(ast, pkg))
		case "fab":
			fmt.Println(EmitFaber(ast))
		}
	}
}

func printUsage() {
	fmt.Println("nanus-go: Faber microcompiler (stdin/stdout)")
	fmt.Println()
	fmt.Println("Usage: <source> | nanus-go <command> [options]")
	fmt.Println()
	fmt.Println("Commands:")
	fmt.Println("  emit     Compile Faber to target language")
	fmt.Println("  parse    Output AST as JSON")
	fmt.Println("  lex      Output tokens as JSON")
	fmt.Println()
	fmt.Println("Options (emit only):")
	fmt.Println("  -t <target>   Output target: go, fab (default: go)")
	fmt.Println("  -p <package>  Go package name (default: main)")
}
