"""solum.py - File System Implementation

Native Python implementation of the HAL solum (filesystem) interface.
Uses os and pathlib for sync, aiofiles for async.

Verb conjugation encodes sync/async:
  - Imperative (-a, -e, -i): synchronous
  - Future indicative (-et, -ebit): asynchronous (returns awaitable)
"""

import os
import asyncio
import shutil
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import Optional

try:
    import aiofiles
    import aiofiles.os
    _HAS_AIOFILES = True
except ImportError:
    _HAS_AIOFILES = False


@dataclass
class SolumStatus:
    """Full file status returned by describe/describet."""
    modus: int           # permission bits (e.g., 0o755)
    nexus: int           # hard link count
    possessor: int       # owner uid
    grex: int            # group gid
    magnitudo: int       # size in bytes
    modificatum: int     # mtime (ms since epoch)
    est_directorii: bool
    est_vinculum: bool   # is symlink


# =============================================================================
# READING - Text
# =============================================================================
# Verb: lege/leget from "legere" (to read, gather)

def lege(via: str) -> str:
    """Read entire file as text (sync)."""
    return Path(via).read_text()


async def leget(via: str) -> str:
    """Read entire file as text (async)."""
    if _HAS_AIOFILES:
        async with aiofiles.open(via, "r") as f:
            return await f.read()
    loop = asyncio.get_event_loop()
    return await loop.run_in_executor(None, lege, via)


# =============================================================================
# READING - Bytes
# =============================================================================
# Verb: hauri/hauriet from "haurire" (to draw up, draw water)

def hauri(via: str) -> bytes:
    """Draw entire file as bytes (sync)."""
    return Path(via).read_bytes()


async def hauriet(via: str) -> bytes:
    """Draw entire file as bytes (async)."""
    if _HAS_AIOFILES:
        async with aiofiles.open(via, "rb") as f:
            return await f.read()
    loop = asyncio.get_event_loop()
    return await loop.run_in_executor(None, hauri, via)


# =============================================================================
# READING - Lines
# =============================================================================
# Verb: carpe/carpiet from "carpere" (to pluck, pick, harvest)

def carpe(via: str) -> list[str]:
    """Pluck lines from file (sync)."""
    return Path(via).read_text().splitlines()


async def carpiet(via: str) -> list[str]:
    """Pluck lines from file (async)."""
    content = await leget(via)
    return content.splitlines()


# =============================================================================
# WRITING - Text
# =============================================================================
# Verb: scribe/scribet from "scribere" (to write)

def scribe(via: str, data: str) -> None:
    """Write text to file, overwrites existing (sync)."""
    Path(via).write_text(data)


async def scribet(via: str, data: str) -> None:
    """Write text to file, overwrites existing (async)."""
    if _HAS_AIOFILES:
        async with aiofiles.open(via, "w") as f:
            await f.write(data)
        return
    loop = asyncio.get_event_loop()
    await loop.run_in_executor(None, scribe, via, data)


# =============================================================================
# WRITING - Bytes
# =============================================================================
# Verb: funde/fundet from "fundere" (to pour, pour out)

def funde(via: str, data: bytes) -> None:
    """Pour bytes to file, overwrites existing (sync)."""
    Path(via).write_bytes(data)


async def fundet(via: str, data: bytes) -> None:
    """Pour bytes to file, overwrites existing (async)."""
    if _HAS_AIOFILES:
        async with aiofiles.open(via, "wb") as f:
            await f.write(data)
        return
    loop = asyncio.get_event_loop()
    await loop.run_in_executor(None, funde, via, data)


# =============================================================================
# WRITING - Append
# =============================================================================
# Verb: appone/apponet from "apponere" (to place near, add to)

def appone(via: str, data: str) -> None:
    """Append text to file (sync)."""
    with open(via, "a") as f:
        f.write(data)


async def apponet(via: str, data: str) -> None:
    """Append text to file (async)."""
    if _HAS_AIOFILES:
        async with aiofiles.open(via, "a") as f:
            await f.write(data)
        return
    loop = asyncio.get_event_loop()
    await loop.run_in_executor(None, appone, via, data)


# =============================================================================
# FILE INFO - Existence
# =============================================================================
# Verb: exstat/exstabit from "exstare" (to stand out, exist)

def exstat(via: str) -> bool:
    """Check if path exists (sync)."""
    return Path(via).exists()


async def exstabit(via: str) -> bool:
    """Check if path exists (async)."""
    if _HAS_AIOFILES:
        return await aiofiles.os.path.exists(via)
    loop = asyncio.get_event_loop()
    return await loop.run_in_executor(None, exstat, via)


# =============================================================================
# FILE INFO - Details
# =============================================================================
# Verb: describe/describet from "describere" (to describe, delineate)

def describe(via: str) -> SolumStatus:
    """Get file details (sync)."""
    stat = os.lstat(via)
    return SolumStatus(
        modus=stat.st_mode & 0o7777,
        nexus=stat.st_nlink,
        possessor=stat.st_uid,
        grex=stat.st_gid,
        magnitudo=stat.st_size,
        modificatum=int(stat.st_mtime * 1000),
        est_directorii=os.path.isdir(via),
        est_vinculum=os.path.islink(via),
    )


async def describet(via: str) -> SolumStatus:
    """Get file details (async)."""
    loop = asyncio.get_event_loop()
    return await loop.run_in_executor(None, describe, via)


# =============================================================================
# FILE INFO - Symlinks
# =============================================================================
# Verb: sequere/sequetur from "sequi" (to follow)

def sequere(via: str) -> str:
    """Follow symlink to get target path (sync)."""
    return os.readlink(via)


async def sequetur(via: str) -> str:
    """Follow symlink to get target path (async)."""
    loop = asyncio.get_event_loop()
    return await loop.run_in_executor(None, sequere, via)


# =============================================================================
# FILE OPERATIONS - Delete
# =============================================================================
# Verb: dele/delet from "delere" (to destroy, delete)

def dele(via: str) -> None:
    """Delete file (sync)."""
    os.remove(via)


async def delet(via: str) -> None:
    """Delete file (async)."""
    if _HAS_AIOFILES:
        await aiofiles.os.remove(via)
        return
    loop = asyncio.get_event_loop()
    await loop.run_in_executor(None, dele, via)


# =============================================================================
# FILE OPERATIONS - Copy
# =============================================================================
# Verb: exscribe/exscribet from "exscribere" (to copy out, transcribe)

def exscribe(fons: str, destinatio: str) -> None:
    """Copy file (sync)."""
    shutil.copy2(fons, destinatio)


async def exscribet(fons: str, destinatio: str) -> None:
    """Copy file (async)."""
    loop = asyncio.get_event_loop()
    await loop.run_in_executor(None, exscribe, fons, destinatio)


# =============================================================================
# FILE OPERATIONS - Rename/Move
# =============================================================================
# Verb: renomina/renominabit from "renominare" (to rename)

def renomina(fons: str, destinatio: str) -> None:
    """Rename or move file (sync)."""
    os.rename(fons, destinatio)


async def renominabit(fons: str, destinatio: str) -> None:
    """Rename or move file (async)."""
    if _HAS_AIOFILES:
        await aiofiles.os.rename(fons, destinatio)
        return
    loop = asyncio.get_event_loop()
    await loop.run_in_executor(None, renomina, fons, destinatio)


# =============================================================================
# FILE OPERATIONS - Touch
# =============================================================================
# Verb: tange/tanget from "tangere" (to touch)

def tange(via: str) -> None:
    """Touch file - create or update mtime (sync)."""
    Path(via).touch()


async def tanget(via: str) -> None:
    """Touch file - create or update mtime (async)."""
    loop = asyncio.get_event_loop()
    await loop.run_in_executor(None, tange, via)


# =============================================================================
# DIRECTORY OPERATIONS - Create
# =============================================================================
# Verb: crea/creabit from "creare" (to create, bring forth)

def crea(via: str) -> None:
    """Create directory, recursive (sync)."""
    Path(via).mkdir(parents=True, exist_ok=True)


async def creabit(via: str) -> None:
    """Create directory, recursive (async)."""
    if _HAS_AIOFILES:
        await aiofiles.os.makedirs(via, exist_ok=True)
        return
    loop = asyncio.get_event_loop()
    await loop.run_in_executor(None, crea, via)


# =============================================================================
# DIRECTORY OPERATIONS - List
# =============================================================================
# Verb: enumera/enumerabit from "enumerare" (to count out, enumerate)

def enumera(via: str) -> list[str]:
    """List directory contents (sync)."""
    return os.listdir(via)


async def enumerabit(via: str) -> list[str]:
    """List directory contents (async)."""
    if _HAS_AIOFILES:
        return await aiofiles.os.listdir(via)
    loop = asyncio.get_event_loop()
    return await loop.run_in_executor(None, enumera, via)


# =============================================================================
# DIRECTORY OPERATIONS - Prune/Remove
# =============================================================================
# Verb: amputa/amputabit from "amputare" (to cut off, prune)

def amputa(via: str) -> None:
    """Prune directory tree, recursive (sync)."""
    shutil.rmtree(via, ignore_errors=True)


async def amputabit(via: str) -> None:
    """Prune directory tree, recursive (async)."""
    loop = asyncio.get_event_loop()
    await loop.run_in_executor(None, amputa, via)


# =============================================================================
# PATH UTILITIES
# =============================================================================
# Pure functions on path strings, not filesystem I/O. Sync only.

def iunge(partes: list[str]) -> str:
    """Join path segments."""
    return os.path.join(*partes) if partes else ""


def directorium(via: str) -> str:
    """Get directory part of path."""
    return os.path.dirname(via)


def basis(via: str) -> str:
    """Get filename part of path."""
    return os.path.basename(via)


def extensio(via: str) -> str:
    """Get file extension (includes dot)."""
    _, ext = os.path.splitext(via)
    return ext


def absolve(via: str) -> str:
    """Resolve to absolute path."""
    return os.path.abspath(via)


def domus() -> str:
    """Get user's home directory."""
    return os.path.expanduser("~")


def temporarium() -> str:
    """Get system temp directory."""
    return tempfile.gettempdir()
