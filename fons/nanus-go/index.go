package nanus

// CompileOptions controls compilation behavior.
type CompileOptions struct {
	Filename string
}

// CompileResult mirrors nanus compile output.
type CompileResult struct {
	Success bool   `json:"success"`
	Output  string `json:"output"`
	Error   string `json:"error"`
}

// Compile Faber source to TypeScript.
func Compile(source string, options *CompileOptions) (result CompileResult) {
	filename := "<stdin>"
	if options != nil && options.Filename != "" {
		filename = options.Filename
	}

	defer func() {
		if r := recover(); r != nil {
			result = CompileResult{
				Success: false,
				Error:   FormatError(r, source, filename),
			}
		}
	}()

	tokens := Prepare(Lex(source, filename))
	ast := Parse(tokens, filename)
	output := Emit(ast)

	return CompileResult{Success: true, Output: output}
}
