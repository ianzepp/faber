package nanus

import "subsidia"

// CompileOptions controls compilation behavior.
type CompileOptions struct {
	Filename    string
	InputFormat string // "faber" (default) or "fg" (Faber Glyph)
	Format      string // "ts" (default) or "fg" (Faber Glyph)
}

// CompileResult mirrors nanus compile output.
type CompileResult struct {
	Success bool   `json:"success"`
	Output  string `json:"output"`
	Error   string `json:"error"`
}

// Compile source to the specified output format.
func Compile(source string, options *CompileOptions) (result CompileResult) {
	filename := "<stdin>"
	inputFormat := "faber"
	outputFormat := "ts"
	if options != nil {
		if options.Filename != "" {
			filename = options.Filename
		}
		if options.InputFormat != "" {
			inputFormat = options.InputFormat
		}
		if options.Format != "" {
			outputFormat = options.Format
		}
	}

	defer func() {
		if r := recover(); r != nil {
			result = CompileResult{
				Success: false,
				Error:   subsidia.FormatError(r, source, filename),
			}
		}
	}()

	var tokens []subsidia.Token
	switch inputFormat {
	case "fg":
		tokens = subsidia.Prepare(LexFG(source, filename))
	default:
		tokens = subsidia.Prepare(Lex(source, filename))
	}

	ast := subsidia.Parse(tokens, filename)

	var output string
	switch outputFormat {
	case "fg":
		output = EmitFG(ast)
	default:
		output = EmitTS(ast)
	}

	return CompileResult{Success: true, Output: output}
}
