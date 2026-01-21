package main

import (
	"fmt"
	"io"
	"os"

	"subsidia"
)

var stdinFilename = "<stdin>"

func main() {
	if len(os.Args) < 2 {
		printUsage()
		os.Exit(1)
	}

	cmd := os.Args[1]

	// Parse flags
	for i := 2; i < len(os.Args); i++ {
		if os.Args[i] == "--stdin-filename" && i+1 < len(os.Args) {
			stdinFilename = os.Args[i+1]
			i++
		}
	}

	switch cmd {
	case "encode":
		convertToGlyph()
	case "decode":
		convertToFaber()
	case "help", "-h", "--help":
		printUsage()
	default:
		fmt.Fprintf(os.Stderr, "Unknown command: %s\n", cmd)
		printUsage()
		os.Exit(1)
	}
}

func printUsage() {
	fmt.Println("glyph-go: Faber Glyph format converter")
	fmt.Println()
	fmt.Println("Usage:")
	fmt.Println("  <source> | glyph-go encode [options]  Convert Faber to Glyph")
	fmt.Println("  <source> | glyph-go decode [options]  Convert Glyph to Faber")
	fmt.Println("  glyph-go help                         Show this help")
	fmt.Println()
	fmt.Println("Options:")
	fmt.Println("  --stdin-filename <f>   Filename for error messages (default: <stdin>)")
}

func convertToGlyph() {
	source, filename := readInput()
	defer func() {
		if r := recover(); r != nil {
			fmt.Fprintln(os.Stderr, subsidia.FormatError(r, source, filename))
			os.Exit(1)
		}
	}()

	tokens := subsidia.Prepare(Lex(source, filename))
	ast := subsidia.Parse(tokens, filename)
	output := EmitGlyph(ast)
	fmt.Print(output)
}

func convertToFaber() {
	source, filename := readInput()
	defer func() {
		if r := recover(); r != nil {
			fmt.Fprintln(os.Stderr, subsidia.FormatError(r, source, filename))
			os.Exit(1)
		}
	}()

	tokens := subsidia.Prepare(LexGlyph(source, filename))
	ast := subsidia.Parse(tokens, filename)
	output := EmitFaber(ast)
	fmt.Print(output)
}

func readInput() (string, string) {
	data, err := io.ReadAll(os.Stdin)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error reading stdin: %v\n", err)
		os.Exit(1)
	}
	return string(data), stdinFilename
}
