// Package solum provides filesystem operations for the Faber HAL.
//
// Etymology: "solum" - ground, floor, base. Local/ground I/O.
package solum

import (
	"os"
	"path/filepath"
	"strings"
	"time"
)

// SolumStatus holds file metadata returned by Describe/Describet.
type SolumStatus struct {
	Modus         int   // permission bits (e.g., 0o755)
	Nexus         int   // hard link count
	Possessor     int   // owner uid
	Grex          int   // group gid
	Magnitudo     int64 // size in bytes
	Modificatum   int64 // mtime (ms since epoch)
	EstDirectorii bool  // is directory
	EstVinculum   bool  // is symlink
}

// =============================================================================
// READING - Text
// =============================================================================

// Lege reads entire file as text.
func Lege(via string) (string, error) {
	data, err := os.ReadFile(via)
	if err != nil {
		return "", err
	}
	return string(data), nil
}

// Leget reads entire file as text (async variant).
func Leget(via string) (string, error) {
	return Lege(via)
}

// =============================================================================
// READING - Bytes
// =============================================================================

// Hauri draws entire file as bytes.
func Hauri(via string) ([]byte, error) {
	return os.ReadFile(via)
}

// Hauriet draws entire file as bytes (async variant).
func Hauriet(via string) ([]byte, error) {
	return Hauri(via)
}

// =============================================================================
// READING - Lines
// =============================================================================

// Carpe plucks lines from file.
func Carpe(via string) ([]string, error) {
	data, err := os.ReadFile(via)
	if err != nil {
		return nil, err
	}
	content := string(data)
	if content == "" {
		return []string{}, nil
	}
	lines := strings.Split(content, "\n")
	// Remove trailing empty line if file ends with newline
	if len(lines) > 0 && lines[len(lines)-1] == "" {
		lines = lines[:len(lines)-1]
	}
	return lines, nil
}

// Carpiet plucks lines from file (async variant).
func Carpiet(via string) ([]string, error) {
	return Carpe(via)
}

// =============================================================================
// WRITING - Text
// =============================================================================

// Scribe writes text to file, overwrites existing.
func Scribe(via string, data string) error {
	return os.WriteFile(via, []byte(data), 0644)
}

// Scribet writes text to file, overwrites existing (async variant).
func Scribet(via string, data string) error {
	return Scribe(via, data)
}

// =============================================================================
// WRITING - Bytes
// =============================================================================

// Funde pours bytes to file, overwrites existing.
func Funde(via string, data []byte) error {
	return os.WriteFile(via, data, 0644)
}

// Fundet pours bytes to file, overwrites existing (async variant).
func Fundet(via string, data []byte) error {
	return Funde(via, data)
}

// =============================================================================
// WRITING - Append
// =============================================================================

// Appone appends text to file.
func Appone(via string, data string) error {
	f, err := os.OpenFile(via, os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0644)
	if err != nil {
		return err
	}
	defer f.Close()
	_, err = f.WriteString(data)
	return err
}

// Apponet appends text to file (async variant).
func Apponet(via string, data string) error {
	return Appone(via, data)
}

// =============================================================================
// FILE INFO - Existence
// =============================================================================

// Exstat checks if path exists.
func Exstat(via string) bool {
	_, err := os.Stat(via)
	return err == nil
}

// Exstabit checks if path exists (async variant).
func Exstabit(via string) bool {
	return Exstat(via)
}

// =============================================================================
// FILE INFO - Details
// =============================================================================

// Describe gets file details using lstat (does not follow symlinks).
func Describe(via string) (SolumStatus, error) {
	info, err := os.Lstat(via)
	if err != nil {
		return SolumStatus{}, err
	}
	return infoToStatus(info), nil
}

// Describet gets file details (async variant).
func Describet(via string) (SolumStatus, error) {
	return Describe(via)
}

func infoToStatus(info os.FileInfo) SolumStatus {
	status := SolumStatus{
		Modus:         int(info.Mode().Perm()),
		Magnitudo:     info.Size(),
		Modificatum:   info.ModTime().UnixMilli(),
		EstDirectorii: info.IsDir(),
		EstVinculum:   info.Mode()&os.ModeSymlink != 0,
	}
	// Nexus, Possessor, Grex require platform-specific syscall
	// Left as zero values for portability
	return status
}

// =============================================================================
// FILE INFO - Symlinks
// =============================================================================

// Sequere follows symlink to get target path.
func Sequere(via string) (string, error) {
	return os.Readlink(via)
}

// Sequetur follows symlink to get target path (async variant).
func Sequetur(via string) (string, error) {
	return Sequere(via)
}

// =============================================================================
// FILE OPERATIONS - Delete
// =============================================================================

// Dele deletes a file.
func Dele(via string) error {
	return os.Remove(via)
}

// Delet deletes a file (async variant).
func Delet(via string) error {
	return Dele(via)
}

// =============================================================================
// FILE OPERATIONS - Copy
// =============================================================================

// Exscribe copies a file from source to destination.
func Exscribe(fons string, destinatio string) error {
	data, err := os.ReadFile(fons)
	if err != nil {
		return err
	}
	info, err := os.Stat(fons)
	if err != nil {
		return err
	}
	return os.WriteFile(destinatio, data, info.Mode().Perm())
}

// Exscribet copies a file (async variant).
func Exscribet(fons string, destinatio string) error {
	return Exscribe(fons, destinatio)
}

// =============================================================================
// FILE OPERATIONS - Rename/Move
// =============================================================================

// Renomina renames or moves a file.
func Renomina(fons string, destinatio string) error {
	return os.Rename(fons, destinatio)
}

// Renominabit renames or moves a file (async variant).
func Renominabit(fons string, destinatio string) error {
	return Renomina(fons, destinatio)
}

// =============================================================================
// FILE OPERATIONS - Touch
// =============================================================================

// Tange touches a file - creates if missing, updates mtime if exists.
func Tange(via string) error {
	if _, err := os.Stat(via); os.IsNotExist(err) {
		f, err := os.Create(via)
		if err != nil {
			return err
		}
		return f.Close()
	}
	now := time.Now()
	return os.Chtimes(via, now, now)
}

// Tanget touches a file (async variant).
func Tanget(via string) error {
	return Tange(via)
}

// =============================================================================
// DIRECTORY OPERATIONS - Create
// =============================================================================

// Crea creates a directory (recursive).
func Crea(via string) error {
	return os.MkdirAll(via, 0755)
}

// Creabit creates a directory (async variant).
func Creabit(via string) error {
	return Crea(via)
}

// =============================================================================
// DIRECTORY OPERATIONS - List
// =============================================================================

// Enumera lists directory contents.
func Enumera(via string) ([]string, error) {
	entries, err := os.ReadDir(via)
	if err != nil {
		return nil, err
	}
	names := make([]string, len(entries))
	for i, entry := range entries {
		names[i] = entry.Name()
	}
	return names, nil
}

// Enumerabit lists directory contents (async variant).
func Enumerabit(via string) ([]string, error) {
	return Enumera(via)
}

// =============================================================================
// DIRECTORY OPERATIONS - Prune/Remove
// =============================================================================

// Amputa prunes a directory tree (recursive delete).
func Amputa(via string) error {
	return os.RemoveAll(via)
}

// Amputabit prunes a directory tree (async variant).
func Amputabit(via string) error {
	return Amputa(via)
}

// =============================================================================
// PATH UTILITIES
// =============================================================================

// Iunge joins path segments.
func Iunge(partes []string) string {
	return filepath.Join(partes...)
}

// Directorium gets directory part of path.
func Directorium(via string) string {
	return filepath.Dir(via)
}

// Basis gets filename part of path.
func Basis(via string) string {
	return filepath.Base(via)
}

// Extensio gets file extension (includes dot).
func Extensio(via string) string {
	return filepath.Ext(via)
}

// Absolve resolves to absolute path.
func Absolve(via string) (string, error) {
	return filepath.Abs(via)
}

// Domus gets user's home directory.
func Domus() (string, error) {
	return os.UserHomeDir()
}

// Temporarium gets system temp directory.
func Temporarium() string {
	return os.TempDir()
}
