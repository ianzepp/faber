// consolum.go - Console Device Implementation (Go)
//
// Native Go implementation of the HAL console interface.
// Provides stdin/stdout/stderr operations.

package hal

import (
	"bufio"
	"os"
)

// Consolum provides console I/O operations
var Consolum = struct {
	HauriOctetos       func(magnitudo int) []byte
	HauriLineam        func() string
	FundeOctetos       func(data []byte)
	FundeTextum        func(msg string)
	FundeLineam        func(msg string)
	ErrorOctetos       func(data []byte)
	ErrorTextum        func(msg string)
	ErrorLineam        func(msg string)
	EstTerminale       func() bool
	EstTerminaleOutput func() bool
}{
	// STDIN

	HauriOctetos: func(magnitudo int) []byte {
		buffer := make([]byte, magnitudo)
		n, err := os.Stdin.Read(buffer)
		if err != nil {
			return []byte{}
		}
		return buffer[:n]
	},

	HauriLineam: func() string {
		scanner := bufio.NewScanner(os.Stdin)
		if scanner.Scan() {
			return scanner.Text()
		}
		return ""
	},

	// STDOUT

	FundeOctetos: func(data []byte) {
		os.Stdout.Write(data)
	},

	FundeTextum: func(msg string) {
		os.Stdout.WriteString(msg)
	},

	FundeLineam: func(msg string) {
		os.Stdout.WriteString(msg + "\n")
	},

	// STDERR

	ErrorOctetos: func(data []byte) {
		os.Stderr.Write(data)
	},

	ErrorTextum: func(msg string) {
		os.Stderr.WriteString(msg)
	},

	ErrorLineam: func(msg string) {
		os.Stderr.WriteString(msg + "\n")
	},

	// TTY

	EstTerminale: func() bool {
		fi, err := os.Stdin.Stat()
		if err != nil {
			return false
		}
		return fi.Mode()&os.ModeCharDevice != 0
	},

	EstTerminaleOutput: func() bool {
		fi, err := os.Stdout.Stat()
		if err != nil {
			return false
		}
		return fi.Mode()&os.ModeCharDevice != 0
	},
}
