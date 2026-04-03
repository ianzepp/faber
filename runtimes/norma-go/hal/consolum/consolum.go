// Package consolum provides console I/O operations for the Faber HAL.
//
// Etymology: "consolum" from "con-" (together) + "solum" (base/ground).
// The foundation for all I/O interaction.
package consolum

import (
	"bufio"
	"os"
)

// Shared reader for stdin line operations
var stdinReader = bufio.NewReader(os.Stdin)

// =============================================================================
// STDIN - Bytes
// =============================================================================

// Hauri draws bytes from stdin.
func Hauri(magnitudo int) ([]byte, error) {
	buf := make([]byte, magnitudo)
	n, err := os.Stdin.Read(buf)
	if err != nil {
		return nil, err
	}
	return buf[:n], nil
}

// Hauriet draws bytes from stdin (async variant).
func Hauriet(magnitudo int) ([]byte, error) {
	return Hauri(magnitudo)
}

// =============================================================================
// STDIN - Text
// =============================================================================

// Lege reads a line from stdin (blocks until newline).
func Lege() (string, error) {
	line, err := stdinReader.ReadString('\n')
	if err != nil {
		return line, err
	}
	// Strip trailing newline
	if len(line) > 0 && line[len(line)-1] == '\n' {
		line = line[:len(line)-1]
	}
	// Strip trailing carriage return (Windows)
	if len(line) > 0 && line[len(line)-1] == '\r' {
		line = line[:len(line)-1]
	}
	return line, nil
}

// Leget reads a line from stdin (async variant).
func Leget() (string, error) {
	return Lege()
}

// =============================================================================
// STDOUT - Bytes
// =============================================================================

// Funde pours bytes to stdout.
func Funde(data []byte) error {
	_, err := os.Stdout.Write(data)
	return err
}

// Fundet pours bytes to stdout (async variant).
func Fundet(data []byte) error {
	return Funde(data)
}

// =============================================================================
// STDOUT - Text with Newline
// =============================================================================

// Scribe writes text to stdout with newline.
func Scribe(msg string) error {
	_, err := os.Stdout.WriteString(msg + "\n")
	return err
}

// Scribet writes text to stdout with newline (async variant).
func Scribet(msg string) error {
	return Scribe(msg)
}

// =============================================================================
// STDOUT - Text without Newline
// =============================================================================

// Dic says text to stdout without newline.
func Dic(msg string) error {
	_, err := os.Stdout.WriteString(msg)
	return err
}

// Dicet says text to stdout without newline (async variant).
func Dicet(msg string) error {
	return Dic(msg)
}

// =============================================================================
// STDERR - Warning/Error Output
// =============================================================================

// Mone warns to stderr with newline.
func Mone(msg string) error {
	_, err := os.Stderr.WriteString(msg + "\n")
	return err
}

// Monet warns to stderr with newline (async variant).
func Monet(msg string) error {
	return Mone(msg)
}

// =============================================================================
// DEBUG Output
// =============================================================================

// Vide writes debug output with newline.
// In production, this may be filtered by log level.
func Vide(msg string) error {
	_, err := os.Stderr.WriteString(msg + "\n")
	return err
}

// Videbit writes debug output with newline (async variant).
func Videbit(msg string) error {
	return Vide(msg)
}

// =============================================================================
// TTY Detection
// =============================================================================

// EstTerminale checks if stdin is connected to a terminal.
func EstTerminale() bool {
	info, err := os.Stdin.Stat()
	if err != nil {
		return false
	}
	return info.Mode()&os.ModeCharDevice != 0
}

// EstTerminaleOutput checks if stdout is connected to a terminal.
func EstTerminaleOutput() bool {
	info, err := os.Stdout.Stat()
	if err != nil {
		return false
	}
	return info.Mode()&os.ModeCharDevice != 0
}
