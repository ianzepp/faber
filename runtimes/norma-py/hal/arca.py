"""arca.py - Database Device Implementation

Native Python implementation of the HAL database interface.
Uses sqlite3 for SQLite, and optional asyncpg/aiomysql for PostgreSQL/MySQL.

Verb conjugation encodes async and cardinality:
  - Future singular (-et): async, returns one value
  - Future plural (-ent): async generator, yields multiple values
"""

import sqlite3
from dataclasses import dataclass, field
from typing import Any, Optional, AsyncIterator
from urllib.parse import urlparse

try:
    import asyncpg
    _HAS_ASYNCPG = True
except ImportError:
    _HAS_ASYNCPG = False

try:
    import aiomysql
    _HAS_AIOMYSQL = True
except ImportError:
    _HAS_AIOMYSQL = False


@dataclass
class Connexio:
    """Database connection wrapper."""
    _conn: Any
    _driver: str
    _open: bool = field(default=True)

    # =========================================================================
    # QUERIES
    # =========================================================================

    async def quaerent(self, sql: str, params: list[Any]) -> AsyncIterator[dict[str, Any]]:
        """Stream rows as async generator."""
        if self._driver == "sqlite":
            cursor = self._conn.execute(sql, params)
            columns = [desc[0] for desc in cursor.description]
            for row in cursor:
                yield dict(zip(columns, row))
        elif self._driver == "postgres" and _HAS_ASYNCPG:
            async with self._conn.transaction():
                async for record in self._conn.cursor(sql, *params):
                    yield dict(record)
        elif self._driver == "mysql" and _HAS_AIOMYSQL:
            async with self._conn.cursor(aiomysql.DictCursor) as cur:
                await cur.execute(sql, params)
                async for row in cur:
                    yield row

    async def quaeret(self, sql: str, params: list[Any]) -> list[dict[str, Any]]:
        """Return all rows as list."""
        if self._driver == "sqlite":
            cursor = self._conn.execute(sql, params)
            columns = [desc[0] for desc in cursor.description]
            return [dict(zip(columns, row)) for row in cursor.fetchall()]
        elif self._driver == "postgres" and _HAS_ASYNCPG:
            rows = await self._conn.fetch(sql, *params)
            return [dict(row) for row in rows]
        elif self._driver == "mysql" and _HAS_AIOMYSQL:
            async with self._conn.cursor(aiomysql.DictCursor) as cur:
                await cur.execute(sql, params)
                return await cur.fetchall()
        return []

    async def capiet(self, sql: str, params: list[Any]) -> Optional[dict[str, Any]]:
        """Return first row or None."""
        if self._driver == "sqlite":
            cursor = self._conn.execute(sql, params)
            columns = [desc[0] for desc in cursor.description]
            row = cursor.fetchone()
            return dict(zip(columns, row)) if row else None
        elif self._driver == "postgres" and _HAS_ASYNCPG:
            row = await self._conn.fetchrow(sql, *params)
            return dict(row) if row else None
        elif self._driver == "mysql" and _HAS_AIOMYSQL:
            async with self._conn.cursor(aiomysql.DictCursor) as cur:
                await cur.execute(sql, params)
                return await cur.fetchone()
        return None

    # =========================================================================
    # MUTATIONS
    # =========================================================================

    async def exsequetur(self, sql: str, params: list[Any]) -> int:
        """Execute INSERT/UPDATE/DELETE, return affected row count."""
        if self._driver == "sqlite":
            cursor = self._conn.execute(sql, params)
            self._conn.commit()
            return cursor.rowcount
        elif self._driver == "postgres" and _HAS_ASYNCPG:
            result = await self._conn.execute(sql, *params)
            return int(result.split()[-1]) if result else 0
        elif self._driver == "mysql" and _HAS_AIOMYSQL:
            async with self._conn.cursor() as cur:
                await cur.execute(sql, params)
                await self._conn.commit()
                return cur.rowcount
        return 0

    async def inseret(self, sql: str, params: list[Any]) -> int:
        """Execute INSERT, return last inserted ID."""
        if self._driver == "sqlite":
            cursor = self._conn.execute(sql, params)
            self._conn.commit()
            return cursor.lastrowid or 0
        elif self._driver == "postgres" and _HAS_ASYNCPG:
            row = await self._conn.fetchrow(sql + " RETURNING id", *params)
            return row["id"] if row else 0
        elif self._driver == "mysql" and _HAS_AIOMYSQL:
            async with self._conn.cursor() as cur:
                await cur.execute(sql, params)
                await self._conn.commit()
                return cur.lastrowid or 0
        return 0

    # =========================================================================
    # TRANSACTIONS
    # =========================================================================

    async def incipiet(self) -> "Transactio":
        """Begin transaction."""
        if self._driver == "sqlite":
            return Transactio(_conn=self._conn, _driver=self._driver)
        elif self._driver == "postgres" and _HAS_ASYNCPG:
            tx = self._conn.transaction()
            await tx.start()
            return Transactio(_conn=self._conn, _driver=self._driver, _tx=tx)
        elif self._driver == "mysql" and _HAS_AIOMYSQL:
            await self._conn.begin()
            return Transactio(_conn=self._conn, _driver=self._driver)
        return Transactio(_conn=self._conn, _driver=self._driver)

    # =========================================================================
    # LIFECYCLE
    # =========================================================================

    def claude(self) -> None:
        """Close connection."""
        self._open = False
        if self._driver == "sqlite":
            self._conn.close()

    def aperta(self) -> bool:
        """Check if connection is open."""
        return self._open


@dataclass
class Transactio:
    """Database transaction wrapper."""
    _conn: Any
    _driver: str
    _tx: Any = None

    async def quaerent(self, sql: str, params: list[Any]) -> AsyncIterator[dict[str, Any]]:
        """Stream rows as async generator within transaction."""
        if self._driver == "sqlite":
            cursor = self._conn.execute(sql, params)
            columns = [desc[0] for desc in cursor.description]
            for row in cursor:
                yield dict(zip(columns, row))
        elif self._driver == "postgres" and _HAS_ASYNCPG:
            async for record in self._conn.cursor(sql, *params):
                yield dict(record)
        elif self._driver == "mysql" and _HAS_AIOMYSQL:
            async with self._conn.cursor(aiomysql.DictCursor) as cur:
                await cur.execute(sql, params)
                async for row in cur:
                    yield row

    async def quaeret(self, sql: str, params: list[Any]) -> list[dict[str, Any]]:
        """Return all rows as list within transaction."""
        if self._driver == "sqlite":
            cursor = self._conn.execute(sql, params)
            columns = [desc[0] for desc in cursor.description]
            return [dict(zip(columns, row)) for row in cursor.fetchall()]
        elif self._driver == "postgres" and _HAS_ASYNCPG:
            rows = await self._conn.fetch(sql, *params)
            return [dict(row) for row in rows]
        elif self._driver == "mysql" and _HAS_AIOMYSQL:
            async with self._conn.cursor(aiomysql.DictCursor) as cur:
                await cur.execute(sql, params)
                return await cur.fetchall()
        return []

    async def exsequetur(self, sql: str, params: list[Any]) -> int:
        """Execute mutation within transaction."""
        if self._driver == "sqlite":
            cursor = self._conn.execute(sql, params)
            return cursor.rowcount
        elif self._driver == "postgres" and _HAS_ASYNCPG:
            result = await self._conn.execute(sql, *params)
            return int(result.split()[-1]) if result else 0
        elif self._driver == "mysql" and _HAS_AIOMYSQL:
            async with self._conn.cursor() as cur:
                await cur.execute(sql, params)
                return cur.rowcount
        return 0

    async def committet(self) -> None:
        """Commit transaction."""
        if self._driver == "sqlite":
            self._conn.commit()
        elif self._driver == "postgres" and _HAS_ASYNCPG and self._tx:
            await self._tx.commit()
        elif self._driver == "mysql" and _HAS_AIOMYSQL:
            await self._conn.commit()

    async def revertet(self) -> None:
        """Rollback transaction."""
        if self._driver == "sqlite":
            self._conn.rollback()
        elif self._driver == "postgres" and _HAS_ASYNCPG and self._tx:
            await self._tx.rollback()
        elif self._driver == "mysql" and _HAS_AIOMYSQL:
            await self._conn.rollback()


# =============================================================================
# CONNECTION
# =============================================================================

async def connectet(url: str) -> Connexio:
    """Connect to database (driver inferred from URL scheme).

    URLs: postgres://, mysql://, sqlite:///path, sqlite://:memory:
    """
    parsed = urlparse(url)
    scheme = parsed.scheme

    if scheme == "sqlite":
        path = parsed.path
        if path.startswith("//"):
            path = path[2:]
        conn = sqlite3.connect(path)
        return Connexio(_conn=conn, _driver="sqlite")

    elif scheme in ("postgres", "postgresql"):
        if not _HAS_ASYNCPG:
            raise RuntimeError("asyncpg package required for PostgreSQL")
        conn = await asyncpg.connect(url)
        return Connexio(_conn=conn, _driver="postgres")

    elif scheme == "mysql":
        if not _HAS_AIOMYSQL:
            raise RuntimeError("aiomysql package required for MySQL")
        conn = await aiomysql.connect(
            host=parsed.hostname or "localhost",
            port=parsed.port or 3306,
            user=parsed.username or "",
            password=parsed.password or "",
            db=parsed.path.lstrip("/"),
        )
        return Connexio(_conn=conn, _driver="mysql")

    else:
        raise ValueError(f"Unsupported database scheme: {scheme}")
