package main

import (
	"fmt"
	"io"
	"os"

	"subsidia"
)

func main() {
	if len(os.Args) < 2 {
		printUsage()
		os.Exit(1)
	}

	cmd := os.Args[1]
	switch cmd {
	case "to-fg", "fg":
		convertToFG()
	case "to-faber", "faber":
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
	fmt.Println("  glyph-go to-fg [file]     Convert Faber to Faber Glyph")
	fmt.Println("  glyph-go to-faber [file]  Convert Faber Glyph to Faber")
	fmt.Println("  glyph-go help             Show this help")
	fmt.Println()
	fmt.Println("If no file is specified, reads from stdin.")
}

func convertToFG() {
	source, filename := readInput()
	defer func() {
		if r := recover(); r != nil {
			fmt.Fprintln(os.Stderr, subsidia.FormatError(r, source, filename))
			os.Exit(1)
		}
	}()

	tokens := subsidia.Prepare(Lex(source, filename))
	ast := subsidia.Parse(tokens, filename)
	output := EmitFG(ast)
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

	tokens := subsidia.Prepare(LexFG(source, filename))
	ast := subsidia.Parse(tokens, filename)
	output := EmitFaber(ast)
	fmt.Print(output)
}

func readInput() (string, string) {
	if len(os.Args) >= 3 {
		filename := os.Args[2]
		data, err := os.ReadFile(filename)
		if err != nil {
			fmt.Fprintf(os.Stderr, "Error reading file: %v\n", err)
			os.Exit(1)
		}
		return string(data), filename
	}

	data, err := io.ReadAll(os.Stdin)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error reading stdin: %v\n", err)
		os.Exit(1)
	}
	return string(data), "<stdin>"
}
