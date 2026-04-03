package subsidia

import (
	"errors"
	"fmt"
	"regexp"
	"strings"
)

// CompileError represents a positioned compile error.
type CompileError struct {
	Message  string
	Locus    Locus
	Filename string
}

func (e *CompileError) Error() string {
	return fmt.Sprintf("%s:%d:%d: %s", e.Filename, e.Locus.Linea, e.Locus.Columna, e.Message)
}

// FormatError renders a human-friendly error message with source context.
func FormatError(err interface{}, source string, filename string) string {
	if err == nil {
		return ""
	}

	var errObj error
	switch e := err.(type) {
	case error:
		errObj = e
	default:
		return fmt.Sprint(err)
	}

	var line, col int
	var msg string

	var compErr *CompileError
	if errors.As(errObj, &compErr) {
		line = compErr.Locus.Linea
		col = compErr.Locus.Columna
		msg = strings.TrimPrefix(compErr.Error(), fmt.Sprintf("%s:%d:%d: ", compErr.Filename, line, col))
	} else {
		re := regexp.MustCompile(`^(\d+):(\d+): (.*)$`)
		match := re.FindStringSubmatch(errObj.Error())
		if len(match) != 4 {
			return errObj.Error()
		}
		fmt.Sscanf(match[1], "%d", &line)
		fmt.Sscanf(match[2], "%d", &col)
		msg = match[3]
	}

	lines := strings.Split(source, "\n")
	var srcLine string
	if line-1 >= 0 && line-1 < len(lines) {
		srcLine = lines[line-1]
	}

	pointer := strings.Repeat(" ", maxInt(0, col-1)) + "^"

	return strings.Join([]string{
		fmt.Sprintf("%s:%d:%d: error: %s", filename, line, col, msg),
		"",
		fmt.Sprintf("  %s", srcLine),
		fmt.Sprintf("  %s", pointer),
	}, "\n")
}

func maxInt(a, b int) int {
	if a > b {
		return a
	}
	return b
}
