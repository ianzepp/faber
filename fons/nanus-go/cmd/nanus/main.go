package main

import (
	"encoding/json"
	"fmt"
	"io"
	"os"
	"strings"

	nanus "nanus-go"
)

func showHelp() {
	fmt.Println("nanus - Minimal Faber compiler")
	fmt.Println("Compiles the Faber subset needed to bootstrap rivus.")
	fmt.Println("")
	fmt.Println("Usage:")
	fmt.Println("  nanus <command> <file> [options]")
	fmt.Println("")
	fmt.Println("Commands:")
	fmt.Println("  emit, compile <file>   Emit .fab file as TypeScript")
	fmt.Println("  parse <file>           Parse and output AST as JSON")
	fmt.Println("  lex <file>             Lex and output tokens as JSON")
	fmt.Println("")
	fmt.Println("Options:")
	fmt.Println("  -o, --output <file>    Output file (default: stdout)")
	fmt.Println("  -h, --help             Show this help")
	fmt.Println("")
	fmt.Println("Reads from stdin if no file specified (or use '-' explicitly).")
}

func main() {
	args := os.Args[1:]
	if len(args) == 0 || contains(args, "-h") || contains(args, "--help") {
		showHelp()
		os.Exit(0)
	}

	var command string
	var input string
	var output string
	filename := "<stdin>"

	for i := 0; i < len(args); i++ {
		arg := args[i]
		switch arg {
		case "-o", "--output":
			if i+1 < len(args) {
				output = args[i+1]
				i++
			}
		case "-h", "--help":
			showHelp()
			os.Exit(0)
		default:
			if strings.HasPrefix(arg, "-") {
				continue
			}
			if command == "" {
				command = arg
			} else if input == "" {
				input = arg
				filename = arg
			}
		}
	}

	validCommands := map[string]struct{}{"emit": {}, "compile": {}, "parse": {}, "lex": {}}
	if _, ok := validCommands[command]; !ok {
		fmt.Fprintf(os.Stderr, "Unknown command: %s\n", firstOr(command, "(none)"))
		fmt.Fprintln(os.Stderr, "Use --help for usage.")
		os.Exit(1)
	}

	source, err := readSource(input)
	if err != nil {
		fmt.Fprintln(os.Stderr, err.Error())
		os.Exit(1)
	}

	defer func() {
		if r := recover(); r != nil {
			fmt.Fprintln(os.Stderr, nanus.FormatError(r, source, filename))
			os.Exit(1)
		}
	}()

	switch command {
	case "lex":
		tokens := nanus.Lex(source, filename)
		out, err := json.MarshalIndent(tokens, "", "  ")
		if err != nil {
			fmt.Fprintln(os.Stderr, err.Error())
			os.Exit(1)
		}
		writeOutput(out, output)
		return
	case "parse":
		tokens := nanus.Prepare(nanus.Lex(source, filename))
		ast := nanus.Parse(tokens, filename)
		out, err := json.MarshalIndent(ast, "", "  ")
		if err != nil {
			fmt.Fprintln(os.Stderr, err.Error())
			os.Exit(1)
		}
		writeOutput(out, output)
		return
	default:
		result := nanus.Compile(source, &nanus.CompileOptions{Filename: filename})
		if !result.Success {
			fmt.Fprintln(os.Stderr, result.Error)
			os.Exit(1)
		}
		writeOutput([]byte(result.Output), output)
		return
	}
}

func readSource(input string) (string, error) {
	if input != "" && input != "-" {
		data, err := os.ReadFile(input)
		if err != nil {
			return "", err
		}
		return string(data), nil
	}

	data, err := io.ReadAll(os.Stdin)
	if err != nil {
		return "", err
	}
	return string(data), nil
}

func writeOutput(data []byte, output string) {
	if output != "" {
		if err := os.WriteFile(output, data, 0644); err != nil {
			fmt.Fprintln(os.Stderr, err.Error())
			os.Exit(1)
		}
		return
	}
	fmt.Println(string(data))
}

func contains(args []string, val string) bool {
	for _, arg := range args {
		if arg == val {
			return true
		}
	}
	return false
}

func firstOr(val string, fallback string) string {
	if val == "" {
		return fallback
	}
	return val
}
