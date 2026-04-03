//! arca.rs - Database Device Implementation
//!
//! Native Rust implementation of the HAL database interface.
//! Uses sqlx with multi-database support (SQLite, PostgreSQL, MySQL).
//!
//! Verb conjugation encodes async and cardinality:
//!   - Future singular (-et): async, returns one value
//!   - Future plural (-ent): async generator, yields multiple values

use async_stream::stream;
use futures::Stream;
use serde_json::Value;
use sqlx::any::{AnyPoolOptions, AnyRow};
use sqlx::{AnyPool, Column, Row, TypeInfo};
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Database connection wrapper
pub struct Connexio {
    pool: AnyPool,
    open: Arc<AtomicBool>,
}

/// Database transaction wrapper
pub struct Transactio {
    tx: Option<sqlx::Transaction<'static, sqlx::Any>>,
}

// =============================================================================
// CONNECTION
// =============================================================================

/// Connect to database (driver inferred from URL scheme)
/// URLs: postgres://, mysql://, sqlite:///path, sqlite://:memory:
pub async fn connectet(url: &str) -> Connexio {
    // Install default drivers
    sqlx::any::install_default_drivers();

    let pool = AnyPoolOptions::new()
        .max_connections(5)
        .connect(url)
        .await
        .expect("failed to connect to database");

    Connexio {
        pool,
        open: Arc::new(AtomicBool::new(true)),
    }
}

impl Connexio {
    // =========================================================================
    // QUERIES
    // =========================================================================

    /// Stream rows as async generator
    pub fn quaerent(
        &self,
        sql: &str,
        params: &[Value],
    ) -> Pin<Box<dyn Stream<Item = Value> + Send + '_>> {
        let sql = sql.to_string();
        let params = params.to_vec();
        let pool = self.pool.clone();

        Box::pin(stream! {
            let query = build_query(&sql, &params);
            let mut rows = query.fetch(&pool);

            use futures::StreamExt;
            while let Some(row) = rows.next().await {
                if let Ok(row) = row {
                    yield row_to_value(&row);
                }
            }
        })
    }

    /// Return all rows as list
    pub async fn quaeret(&self, sql: &str, params: &[Value]) -> Vec<Value> {
        let query = build_query(sql, params);
        let rows = query
            .fetch_all(&self.pool)
            .await
            .expect("failed to execute query");

        rows.iter().map(row_to_value).collect()
    }

    /// Return first row or None
    pub async fn capiet(&self, sql: &str, params: &[Value]) -> Option<Value> {
        let query = build_query(sql, params);
        let row = query.fetch_optional(&self.pool).await.expect("failed to execute query");

        row.as_ref().map(row_to_value)
    }

    // =========================================================================
    // MUTATIONS
    // =========================================================================

    /// Execute INSERT/UPDATE/DELETE, return affected row count
    pub async fn exsequetur(&self, sql: &str, params: &[Value]) -> u64 {
        let query = build_query(sql, params);
        let result = query
            .execute(&self.pool)
            .await
            .expect("failed to execute mutation");

        result.rows_affected()
    }

    /// Execute INSERT, return last inserted ID
    pub async fn inseret(&self, sql: &str, params: &[Value]) -> i64 {
        let query = build_query(sql, params);
        let result = query
            .execute(&self.pool)
            .await
            .expect("failed to execute insert");

        result.last_insert_id().unwrap_or(0)
    }

    // =========================================================================
    // TRANSACTIONS
    // =========================================================================

    /// Begin transaction
    pub async fn incipiet(&self) -> Transactio {
        let tx = self
            .pool
            .begin()
            .await
            .expect("failed to begin transaction");

        Transactio { tx: Some(tx) }
    }

    // =========================================================================
    // LIFECYCLE
    // =========================================================================

    /// Close connection
    pub fn claude(&self) {
        self.open.store(false, Ordering::SeqCst);
        // Pool will be dropped when Connexio is dropped
    }

    /// Check if connection is open
    pub fn aperta(&self) -> bool {
        self.open.load(Ordering::SeqCst)
    }
}

impl Transactio {
    /// Stream rows as async generator within transaction
    pub fn quaerent(
        &mut self,
        sql: &str,
        params: &[Value],
    ) -> Pin<Box<dyn Stream<Item = Value> + Send + '_>> {
        let sql = sql.to_string();
        let params = params.to_vec();

        // Get mutable reference to transaction
        let tx = self.tx.as_mut().expect("transaction already consumed");

        Box::pin(stream! {
            let query = build_query(&sql, &params);
            let mut rows = query.fetch(&mut **tx);

            use futures::StreamExt;
            while let Some(row) = rows.next().await {
                if let Ok(row) = row {
                    yield row_to_value(&row);
                }
            }
        })
    }

    /// Return all rows as list within transaction
    pub async fn quaeret(&mut self, sql: &str, params: &[Value]) -> Vec<Value> {
        let tx = self.tx.as_mut().expect("transaction already consumed");
        let query = build_query(sql, params);
        let rows = query
            .fetch_all(&mut **tx)
            .await
            .expect("failed to execute query");

        rows.iter().map(row_to_value).collect()
    }

    /// Execute mutation within transaction
    pub async fn exsequetur(&mut self, sql: &str, params: &[Value]) -> u64 {
        let tx = self.tx.as_mut().expect("transaction already consumed");
        let query = build_query(sql, params);
        let result = query
            .execute(&mut **tx)
            .await
            .expect("failed to execute mutation");

        result.rows_affected()
    }

    /// Commit transaction
    pub async fn committet(mut self) {
        let tx = self.tx.take().expect("transaction already consumed");
        tx.commit().await.expect("failed to commit transaction");
    }

    /// Rollback transaction
    pub async fn revertet(mut self) {
        let tx = self.tx.take().expect("transaction already consumed");
        tx.rollback().await.expect("failed to rollback transaction");
    }
}

// =============================================================================
// HELPERS
// =============================================================================

/// Build a query with bound parameters
fn build_query<'a>(
    sql: &'a str,
    params: &'a [Value],
) -> sqlx::query::Query<'a, sqlx::Any, sqlx::any::AnyArguments<'a>> {
    let mut query = sqlx::query(sql);

    for param in params {
        query = match param {
            Value::Null => query.bind(None::<String>),
            Value::Bool(b) => query.bind(*b),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    query.bind(i)
                } else if let Some(f) = n.as_f64() {
                    query.bind(f)
                } else {
                    query.bind(None::<String>)
                }
            }
            Value::String(s) => query.bind(s.as_str()),
            Value::Array(_) | Value::Object(_) => {
                // Serialize complex types as JSON strings
                query.bind(serde_json::to_string(param).unwrap_or_default())
            }
        };
    }

    query
}

/// Convert a database row to a JSON value (object with column names as keys)
fn row_to_value(row: &AnyRow) -> Value {
    let mut obj = serde_json::Map::new();

    for col in row.columns() {
        let name = col.name().to_string();
        let type_name = col.type_info().name();

        let value: Value = match type_name {
            "INTEGER" | "INT" | "INT4" | "INT8" | "BIGINT" | "SMALLINT" => {
                row.try_get::<i64, _>(col.ordinal())
                    .map(Value::from)
                    .unwrap_or(Value::Null)
            }
            "REAL" | "FLOAT" | "FLOAT4" | "FLOAT8" | "DOUBLE" | "NUMERIC" | "DECIMAL" => {
                row.try_get::<f64, _>(col.ordinal())
                    .map(Value::from)
                    .unwrap_or(Value::Null)
            }
            "BOOLEAN" | "BOOL" => row
                .try_get::<bool, _>(col.ordinal())
                .map(Value::from)
                .unwrap_or(Value::Null),
            "TEXT" | "VARCHAR" | "CHAR" | "STRING" => row
                .try_get::<String, _>(col.ordinal())
                .map(Value::from)
                .unwrap_or(Value::Null),
            "BLOB" | "BYTEA" => row
                .try_get::<Vec<u8>, _>(col.ordinal())
                .map(|bytes| Value::from(base64_encode(&bytes)))
                .unwrap_or(Value::Null),
            _ => {
                // Default: try as string
                row.try_get::<String, _>(col.ordinal())
                    .map(Value::from)
                    .unwrap_or(Value::Null)
            }
        };

        obj.insert(name, value);
    }

    Value::Object(obj)
}

/// Simple base64 encoding for BLOB fields
fn base64_encode(data: &[u8]) -> String {
    use std::io::Write;
    let mut buf = Vec::new();
    let mut encoder = Base64Encoder::new(&mut buf);
    encoder.write_all(data).unwrap();
    drop(encoder);
    String::from_utf8(buf).unwrap_or_default()
}

/// Minimal base64 encoder (avoiding extra dependencies)
struct Base64Encoder<W: std::io::Write> {
    writer: W,
}

impl<W: std::io::Write> Base64Encoder<W> {
    fn new(writer: W) -> Self {
        Self { writer }
    }
}

impl<W: std::io::Write> std::io::Write for Base64Encoder<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        const ALPHABET: &[u8; 64] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

        for chunk in buf.chunks(3) {
            let mut n = (chunk[0] as u32) << 16;
            if chunk.len() > 1 {
                n |= (chunk[1] as u32) << 8;
            }
            if chunk.len() > 2 {
                n |= chunk[2] as u32;
            }

            self.writer.write_all(&[ALPHABET[(n >> 18 & 63) as usize]])?;
            self.writer.write_all(&[ALPHABET[(n >> 12 & 63) as usize]])?;

            if chunk.len() > 1 {
                self.writer.write_all(&[ALPHABET[(n >> 6 & 63) as usize]])?;
            } else {
                self.writer.write_all(b"=")?;
            }

            if chunk.len() > 2 {
                self.writer.write_all(&[ALPHABET[(n & 63) as usize]])?;
            } else {
                self.writer.write_all(b"=")?;
            }
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}
