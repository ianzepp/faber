// solum.go - File System Implementation (Go)
//
// Native Go implementation of the HAL solum (filesystem) interface.
// Uses os, io, and path/filepath packages.

package hal

import (
	"io"
	"io/fs"
	"os"
	"path/filepath"
	"time"
)

// SolumStatus holds file status information
type SolumStatus struct {
	Modus        int64
	Nexus        int64
	Possessor    int64
	Grex         int64
	Magnitudo    int64
	Modificatum  int64
	EstDirectorii bool
	EstVinculum  bool
}

// Solum provides filesystem operations
var Solum = struct {
	// Reading
	Lege        func(path string) string
	LegeOctetos func(path string) []byte

	// Writing
	Scribe        func(path string, data string)
	ScribeOctetos func(path string, data []byte)
	Appone        func(path string, data string)

	// File Info
	Exstat        func(path string) bool
	EstLimae      func(path string) bool
	EstDirectorii func(path string) bool
	Magnitudo     func(path string) int64
	Modificatum   func(path string) int64

	// File Status
	Status       func(path string) SolumStatus
	LegeVinculum func(path string) string

	// File Operations
	Dele  func(path string)
	Copia func(src, dest string)
	Move  func(src, dest string)
	Tange func(path string)

	// Directory Operations
	CreaDir     func(path string)
	Elenca      func(path string) []string
	DeleDir     func(path string)
	DeleArborem func(path string)

	// Path Utilities
	Iunge    func(parts []string) string
	Dir      func(path string) string
	Basis    func(path string) string
	Extensio func(path string) string
	Absolve  func(path string) string
	Domus    func() string
	Temp     func() string
}{
	// =========================================================================
	// READING
	// =========================================================================

	Lege: func(path string) string {
		data, err := os.ReadFile(path)
		if err != nil {
			panic(err)
		}
		return string(data)
	},

	LegeOctetos: func(path string) []byte {
		data, err := os.ReadFile(path)
		if err != nil {
			panic(err)
		}
		return data
	},

	// =========================================================================
	// WRITING
	// =========================================================================

	Scribe: func(path string, data string) {
		err := os.WriteFile(path, []byte(data), 0644)
		if err != nil {
			panic(err)
		}
	},

	ScribeOctetos: func(path string, data []byte) {
		err := os.WriteFile(path, data, 0644)
		if err != nil {
			panic(err)
		}
	},

	Appone: func(path string, data string) {
		f, err := os.OpenFile(path, os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0644)
		if err != nil {
			panic(err)
		}
		defer f.Close()
		if _, err := f.WriteString(data); err != nil {
			panic(err)
		}
	},

	// =========================================================================
	// FILE INFO
	// =========================================================================

	Exstat: func(path string) bool {
		_, err := os.Stat(path)
		return err == nil
	},

	EstLimae: func(path string) bool {
		info, err := os.Stat(path)
		if err != nil {
			return false
		}
		return info.Mode().IsRegular()
	},

	EstDirectorii: func(path string) bool {
		info, err := os.Stat(path)
		if err != nil {
			return false
		}
		return info.IsDir()
	},

	Magnitudo: func(path string) int64 {
		info, err := os.Stat(path)
		if err != nil {
			panic(err)
		}
		return info.Size()
	},

	Modificatum: func(path string) int64 {
		info, err := os.Stat(path)
		if err != nil {
			panic(err)
		}
		return info.ModTime().UnixMilli()
	},

	// =========================================================================
	// FILE STATUS
	// =========================================================================

	Status: func(path string) SolumStatus {
		info, err := os.Lstat(path)
		if err != nil {
			panic(err)
		}
		return SolumStatus{
			Modus:        int64(info.Mode().Perm()),
			Nexus:        1, // Go doesn't expose nlink portably
			Possessor:    0, // Would need syscall for uid
			Grex:         0, // Would need syscall for gid
			Magnitudo:    info.Size(),
			Modificatum:  info.ModTime().UnixMilli(),
			EstDirectorii: info.IsDir(),
			EstVinculum:  info.Mode()&fs.ModeSymlink != 0,
		}
	},

	LegeVinculum: func(path string) string {
		target, err := os.Readlink(path)
		if err != nil {
			panic(err)
		}
		return target
	},

	// =========================================================================
	// FILE OPERATIONS
	// =========================================================================

	Dele: func(path string) {
		if err := os.Remove(path); err != nil {
			panic(err)
		}
	},

	Copia: func(src, dest string) {
		srcFile, err := os.Open(src)
		if err != nil {
			panic(err)
		}
		defer srcFile.Close()

		destFile, err := os.Create(dest)
		if err != nil {
			panic(err)
		}
		defer destFile.Close()

		if _, err := io.Copy(destFile, srcFile); err != nil {
			panic(err)
		}
	},

	Move: func(src, dest string) {
		if err := os.Rename(src, dest); err != nil {
			panic(err)
		}
	},

	Tange: func(path string) {
		now := time.Now()
		if err := os.Chtimes(path, now, now); err != nil {
			// File doesn't exist, create it
			f, err := os.Create(path)
			if err != nil {
				panic(err)
			}
			f.Close()
		}
	},

	// =========================================================================
	// DIRECTORY OPERATIONS
	// =========================================================================

	CreaDir: func(path string) {
		if err := os.MkdirAll(path, 0755); err != nil {
			panic(err)
		}
	},

	Elenca: func(path string) []string {
		entries, err := os.ReadDir(path)
		if err != nil {
			panic(err)
		}
		names := make([]string, len(entries))
		for i, entry := range entries {
			names[i] = entry.Name()
		}
		return names
	},

	DeleDir: func(path string) {
		if err := os.Remove(path); err != nil {
			panic(err)
		}
	},

	DeleArborem: func(path string) {
		if err := os.RemoveAll(path); err != nil {
			panic(err)
		}
	},

	// =========================================================================
	// PATH UTILITIES
	// =========================================================================

	Iunge: func(parts []string) string {
		return filepath.Join(parts...)
	},

	Dir: func(path string) string {
		return filepath.Dir(path)
	},

	Basis: func(path string) string {
		return filepath.Base(path)
	},

	Extensio: func(path string) string {
		return filepath.Ext(path)
	},

	Absolve: func(path string) string {
		abs, err := filepath.Abs(path)
		if err != nil {
			panic(err)
		}
		return abs
	},

	Domus: func() string {
		home, err := os.UserHomeDir()
		if err != nil {
			panic(err)
		}
		return home
	},

	Temp: func() string {
		return os.TempDir()
	},
}
