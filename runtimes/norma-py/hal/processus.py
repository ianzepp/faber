"""processus.py - Process Device Implementation

Native Python implementation of the HAL process interface.
Uses subprocess for sync, asyncio.subprocess for async.

Spawn semantics encoded via different verbs:
  - genera: spawn attached, caller manages lifecycle
  - dimitte: spawn detached, dismiss to run independently
"""

import os
import sys
import asyncio
import subprocess
from dataclasses import dataclass
from typing import Optional


@dataclass
class Subprocessus:
    """Spawned subprocess handle for attached processes."""
    pid: int
    _process: subprocess.Popen

    def expiravit(self) -> int:
        """Wait for process to exit and return exit code (sync)."""
        return self._process.wait()

    async def expirabit(self) -> int:
        """Wait for process to exit and return exit code (async)."""
        loop = asyncio.get_event_loop()
        return await loop.run_in_executor(None, self._process.wait)


@dataclass
class SubprocessusAsync:
    """Async spawned subprocess handle."""
    pid: int
    _process: asyncio.subprocess.Process

    async def expirabit(self) -> int:
        """Wait for process to exit and return exit code (async)."""
        return await self._process.wait()


# =============================================================================
# SPAWN - Attached
# =============================================================================
# Verb: genera from "generare" (to generate, beget)

def genera(
    argumenta: list[str],
    directorium: Optional[str] = None,
    ambitus: Optional[dict[str, str]] = None,
) -> Subprocessus:
    """Spawn attached process - caller can wait for exit via handle.expiravit()."""
    if not argumenta:
        raise ValueError("genera: argumenta cannot be empty")

    env = {**os.environ, **(ambitus or {})} if ambitus else None

    process = subprocess.Popen(
        argumenta,
        cwd=directorium,
        env=env,
        stdin=sys.stdin,
        stdout=sys.stdout,
        stderr=sys.stderr,
    )

    return Subprocessus(pid=process.pid, _process=process)


async def generabit(
    argumenta: list[str],
    directorium: Optional[str] = None,
    ambitus: Optional[dict[str, str]] = None,
) -> SubprocessusAsync:
    """Spawn attached process (async) - returns async handle."""
    if not argumenta:
        raise ValueError("generabit: argumenta cannot be empty")

    env = {**os.environ, **(ambitus or {})} if ambitus else None

    process = await asyncio.create_subprocess_exec(
        *argumenta,
        cwd=directorium,
        env=env,
        stdin=sys.stdin,
        stdout=sys.stdout,
        stderr=sys.stderr,
    )

    return SubprocessusAsync(pid=process.pid or 0, _process=process)


# =============================================================================
# SPAWN - Detached
# =============================================================================
# Verb: dimitte from "dimittere" (to send away, dismiss)

def dimitte(
    argumenta: list[str],
    directorium: Optional[str] = None,
    ambitus: Optional[dict[str, str]] = None,
) -> int:
    """Dismiss process to run independently - returns PID."""
    if not argumenta:
        raise ValueError("dimitte: argumenta cannot be empty")

    env = {**os.environ, **(ambitus or {})} if ambitus else None

    process = subprocess.Popen(
        argumenta,
        cwd=directorium,
        env=env,
        stdin=subprocess.DEVNULL,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        start_new_session=True,
    )

    return process.pid


# =============================================================================
# SHELL EXECUTION
# =============================================================================
# Verb: exsequi/exsequetur from "exsequi" (to execute, accomplish)

def exsequi(imperium: str) -> str:
    """Execute shell command, block until complete, return stdout (sync)."""
    result = subprocess.run(
        ["sh", "-c", imperium],
        capture_output=True,
        text=True,
    )
    return result.stdout


async def exsequetur(imperium: str) -> str:
    """Execute shell command, return stdout when complete (async)."""
    process = await asyncio.create_subprocess_shell(
        imperium,
        stdout=asyncio.subprocess.PIPE,
        stderr=asyncio.subprocess.PIPE,
    )
    stdout, _ = await process.communicate()
    return stdout.decode()


# =============================================================================
# ENVIRONMENT - Read
# =============================================================================
# Verb: lege from "legere" (to read)

def lege(nomen: str) -> Optional[str]:
    """Read environment variable (returns None if not set)."""
    return os.environ.get(nomen)


# =============================================================================
# ENVIRONMENT - Write
# =============================================================================
# Verb: scribe from "scribere" (to write)

def scribe(nomen: str, valor: str) -> None:
    """Write environment variable."""
    os.environ[nomen] = valor


# =============================================================================
# PROCESS INFO - Working Directory
# =============================================================================

def sedes() -> str:
    """Get current working directory (where the process dwells)."""
    return os.getcwd()


# =============================================================================
# PROCESS INFO - Change Directory
# =============================================================================
# Verb: muta from "mutare" (to change)

def muta(via: str) -> None:
    """Change current working directory."""
    os.chdir(via)


# =============================================================================
# PROCESS INFO - Identity
# =============================================================================

def identitas() -> int:
    """Get process ID."""
    return os.getpid()


# =============================================================================
# PROCESS INFO - Arguments
# =============================================================================

def argumenta() -> list[str]:
    """Get command line arguments (excludes runtime and script path)."""
    return sys.argv[1:]


# =============================================================================
# EXIT
# =============================================================================
# Verb: exi from "exire" (to exit, depart)

def exi(code: int) -> None:
    """Exit process with code (never returns)."""
    sys.exit(code)
