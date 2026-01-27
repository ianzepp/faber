"""consolum.py - Console Device Implementation

Native Python implementation of the HAL console interface.
Uses sys for I/O.

Verb conjugation encodes sync/async:
  - Imperative (-a, -e, -i): synchronous
  - Future indicative (-et, -ebit): asynchronous (returns awaitable)

Aligns with language keywords: scribe (info), mone (warn), vide (debug)
"""

import sys
import asyncio
from typing import Optional


# =============================================================================
# STDIN - Bytes
# =============================================================================
# Verb: hauri/hauriet from "haurire" (to draw up)

def hauri(magnitudo: int) -> bytes:
    """Draw bytes from stdin (sync)."""
    return sys.stdin.buffer.read(magnitudo)


async def hauriet(magnitudo: int) -> bytes:
    """Draw bytes from stdin (async)."""
    loop = asyncio.get_event_loop()
    return await loop.run_in_executor(None, sys.stdin.buffer.read, magnitudo)


# =============================================================================
# STDIN - Text
# =============================================================================
# Verb: lege/leget from "legere" (to read)

def lege() -> str:
    """Read line from stdin (sync, blocks until newline)."""
    line = sys.stdin.readline()
    return line.rstrip("\r\n")


async def leget() -> str:
    """Read line from stdin (async)."""
    loop = asyncio.get_event_loop()
    line = await loop.run_in_executor(None, sys.stdin.readline)
    return line.rstrip("\r\n")


# =============================================================================
# STDOUT - Bytes
# =============================================================================
# Verb: funde/fundet from "fundere" (to pour)

def funde(data: bytes) -> None:
    """Pour bytes to stdout (sync)."""
    sys.stdout.buffer.write(data)
    sys.stdout.buffer.flush()


async def fundet(data: bytes) -> None:
    """Pour bytes to stdout (async)."""
    loop = asyncio.get_event_loop()
    await loop.run_in_executor(None, _funde_sync, data)


def _funde_sync(data: bytes) -> None:
    sys.stdout.buffer.write(data)
    sys.stdout.buffer.flush()


# =============================================================================
# STDOUT - Text with Newline
# =============================================================================
# Verb: scribe/scribet from "scribere" (to write)

def scribe(msg: str) -> None:
    """Write line to stdout with newline (sync)."""
    print(msg, flush=True)


async def scribet(msg: str) -> None:
    """Write line to stdout with newline (async)."""
    loop = asyncio.get_event_loop()
    await loop.run_in_executor(None, print, msg)


# =============================================================================
# STDOUT - Text without Newline
# =============================================================================
# Verb: dic/dicet from "dicere" (to say)

def dic(msg: str) -> None:
    """Say text to stdout without newline (sync)."""
    sys.stdout.write(msg)
    sys.stdout.flush()


async def dicet(msg: str) -> None:
    """Say text to stdout without newline (async)."""
    loop = asyncio.get_event_loop()
    await loop.run_in_executor(None, _dic_sync, msg)


def _dic_sync(msg: str) -> None:
    sys.stdout.write(msg)
    sys.stdout.flush()


# =============================================================================
# STDERR - Warning/Error Output
# =============================================================================
# Verb: mone/monet from "monere" (to warn)

def mone(msg: str) -> None:
    """Warn line to stderr with newline (sync)."""
    print(msg, file=sys.stderr, flush=True)


async def monet(msg: str) -> None:
    """Warn line to stderr with newline (async)."""
    loop = asyncio.get_event_loop()
    await loop.run_in_executor(None, _mone_sync, msg)


def _mone_sync(msg: str) -> None:
    print(msg, file=sys.stderr, flush=True)


# =============================================================================
# DEBUG Output
# =============================================================================
# Verb: vide/videbit from "videre" (to see)

def vide(msg: str) -> None:
    """Debug line with newline (sync)."""
    print(msg, file=sys.stderr, flush=True)


async def videbit(msg: str) -> None:
    """Debug line with newline (async)."""
    loop = asyncio.get_event_loop()
    await loop.run_in_executor(None, _vide_sync, msg)


def _vide_sync(msg: str) -> None:
    print(msg, file=sys.stderr, flush=True)


# =============================================================================
# TTY Detection
# =============================================================================

def est_terminale() -> bool:
    """Is stdin connected to a terminal?"""
    return sys.stdin.isatty()


def est_terminale_output() -> bool:
    """Is stdout connected to a terminal?"""
    return sys.stdout.isatty()
