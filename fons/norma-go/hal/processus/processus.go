// Package processus provides process management for the Faber HAL.
//
// Etymology: "processus" from "pro-" (forward) + "cedere" (to go).
// A going forward, a process in motion.
package processus

import (
	"os"
	"os/exec"
	"runtime"
	"syscall"
)

// Subprocessus represents a spawned subprocess handle.
type Subprocessus struct {
	Pid int
	cmd *exec.Cmd
}

// Expiravit waits for the process to exit and returns the exit code.
func (s *Subprocessus) Expiravit() (int, error) {
	err := s.cmd.Wait()
	if err != nil {
		if exitErr, ok := err.(*exec.ExitError); ok {
			return exitErr.ExitCode(), nil
		}
		return -1, err
	}
	return 0, nil
}

// =============================================================================
// SPAWN - Attached
// =============================================================================

// Genera spawns an attached process. Caller can wait via Expiravit().
func Genera(argumenta []string, directorium string, ambitus map[string]string) (*Subprocessus, error) {
	if len(argumenta) == 0 {
		return nil, &exec.Error{Name: "", Err: exec.ErrNotFound}
	}

	cmd := exec.Command(argumenta[0], argumenta[1:]...)

	if directorium != "" {
		cmd.Dir = directorium
	}

	if ambitus != nil {
		cmd.Env = buildEnv(ambitus)
	}

	// Inherit stdio
	cmd.Stdin = os.Stdin
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr

	if err := cmd.Start(); err != nil {
		return nil, err
	}

	return &Subprocessus{
		Pid: cmd.Process.Pid,
		cmd: cmd,
	}, nil
}

// Generabit spawns an attached process (async variant).
func Generabit(argumenta []string, directorium string, ambitus map[string]string) (*Subprocessus, error) {
	return Genera(argumenta, directorium, ambitus)
}

// =============================================================================
// SPAWN - Detached
// =============================================================================

// Dimitte dismisses a process to run independently. Returns PID.
// Process continues after parent exits.
func Dimitte(argumenta []string, directorium string, ambitus map[string]string) (int, error) {
	if len(argumenta) == 0 {
		return 0, &exec.Error{Name: "", Err: exec.ErrNotFound}
	}

	cmd := exec.Command(argumenta[0], argumenta[1:]...)

	if directorium != "" {
		cmd.Dir = directorium
	}

	if ambitus != nil {
		cmd.Env = buildEnv(ambitus)
	}

	// Detach: no stdio, new process group
	cmd.Stdin = nil
	cmd.Stdout = nil
	cmd.Stderr = nil
	cmd.SysProcAttr = detachAttrs()

	if err := cmd.Start(); err != nil {
		return 0, err
	}

	// Release so we don't wait for it
	cmd.Process.Release()

	return cmd.Process.Pid, nil
}

// Dimittet dismisses a process to run independently (async variant).
func Dimittet(argumenta []string, directorium string, ambitus map[string]string) (int, error) {
	return Dimitte(argumenta, directorium, ambitus)
}

// =============================================================================
// SHELL EXECUTION
// =============================================================================

// Exsequi executes a shell command and returns stdout.
func Exsequi(imperium string) (string, error) {
	shell, flag := shellCmd()
	cmd := exec.Command(shell, flag, imperium)
	output, err := cmd.Output()
	if err != nil {
		return string(output), err
	}
	return string(output), nil
}

// Exsequetur executes a shell command (async variant).
func Exsequetur(imperium string) (string, error) {
	return Exsequi(imperium)
}

// =============================================================================
// ENVIRONMENT - Read
// =============================================================================

// Lege reads an environment variable. Returns empty string if not set.
func Lege(nomen string) string {
	return os.Getenv(nomen)
}

// =============================================================================
// ENVIRONMENT - Write
// =============================================================================

// Scribe writes an environment variable.
func Scribe(nomen string, valor string) error {
	return os.Setenv(nomen, valor)
}

// =============================================================================
// PROCESS INFO - Working Directory
// =============================================================================

// Sedes gets current working directory.
func Sedes() (string, error) {
	return os.Getwd()
}

// =============================================================================
// PROCESS INFO - Change Directory
// =============================================================================

// Muta changes the current working directory.
func Muta(via string) error {
	return os.Chdir(via)
}

// =============================================================================
// PROCESS INFO - Identity
// =============================================================================

// Identitas gets the process ID.
func Identitas() int {
	return os.Getpid()
}

// =============================================================================
// PROCESS INFO - Arguments
// =============================================================================

// Argumenta gets command line arguments (excludes program name).
func Argumenta() []string {
	if len(os.Args) <= 1 {
		return []string{}
	}
	return os.Args[1:]
}

// =============================================================================
// EXIT
// =============================================================================

// Exi exits the process with the given code. Does not return.
func Exi(code int) {
	os.Exit(code)
}

// =============================================================================
// HELPERS
// =============================================================================

func buildEnv(ambitus map[string]string) []string {
	// Start with current environment
	env := os.Environ()
	// Append/override with provided values
	for k, v := range ambitus {
		env = append(env, k+"="+v)
	}
	return env
}

func shellCmd() (string, string) {
	if runtime.GOOS == "windows" {
		return "cmd", "/c"
	}
	return "sh", "-c"
}

func detachAttrs() *syscall.SysProcAttr {
	// Platform-specific detach attributes
	// On Unix: Setpgid creates new process group
	// On Windows: different handling needed
	if runtime.GOOS == "windows" {
		return &syscall.SysProcAttr{}
	}
	return &syscall.SysProcAttr{
		Setpgid: true,
	}
}
