//! Database connection management.
//!
//! `tax_ctrl_core` does not hardcode a single database backend. SeaORM
//! resolves the driver (SQLite vs PostgreSQL) from the scheme of the
//! `DATABASE_URL` connection string at runtime, so the same binary can be
//! pointed at either backend simply by changing configuration:
//!
//! * `sqlite://tax_ctrl.db?mode=rwc` — local-first / development default.
//! * `postgres://user:pass@host:5432/db` — shared / production default.
//!
//! No code changes are required to switch between them.

use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use std::time::Duration;

/// Connection configuration for [`connect`].
///
/// Construct this from an explicit URL with [`DbConfig::from_url`], or from
/// the `DATABASE_URL` environment variable with [`DbConfig::from_env`].
#[derive(Debug, Clone)]
pub struct DbConfig {
    /// SeaORM/SQLx-compatible connection string. The scheme (`sqlite://`,
    /// `postgres://` / `postgresql://`) determines which backend is used.
    pub url: String,
    /// Maximum number of pooled connections.
    pub max_connections: u32,
    /// Minimum number of pooled connections kept open.
    pub min_connections: u32,
    /// Timeout when establishing a new connection.
    pub connect_timeout: Duration,
    /// Timeout when acquiring a connection from the pool.
    pub acquire_timeout: Duration,
    /// Whether SQLx should log executed statements.
    pub sqlx_logging: bool,
}

impl DbConfig {
    /// Build a configuration from an explicit connection string, using
    /// sensible defaults for pool sizing and timeouts.
    pub fn from_url(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            max_connections: 10,
            min_connections: 1,
            connect_timeout: Duration::from_secs(8),
            acquire_timeout: Duration::from_secs(8),
            sqlx_logging: false,
        }
    }

    /// Build a configuration from the `DATABASE_URL` environment variable.
    ///
    /// A local `.env` file is loaded first if present (via `dotenvy`); this
    /// is a no-op in production environments where `DATABASE_URL` is already
    /// set by the deployment environment.
    ///
    /// # Errors
    /// Returns [`DbErr::Custom`] if `DATABASE_URL` is not set.
    pub fn from_env() -> Result<Self, DbErr> {
        let _ = dotenvy::dotenv();
        let url = std::env::var("DATABASE_URL")
            .map_err(|_| DbErr::Custom("DATABASE_URL is not set".to_owned()))?;
        Ok(Self::from_url(url))
    }
}

/// Establish a database connection using the given configuration.
///
/// Because the backend is resolved from `config.url`'s scheme, this
/// function is identical for SQLite and PostgreSQL — the caller only needs
/// to supply the right connection string.
pub async fn connect(config: &DbConfig) -> Result<DatabaseConnection, DbErr> {
    let mut opt = ConnectOptions::new(&config.url);
    opt.max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .connect_timeout(config.connect_timeout)
        .acquire_timeout(config.acquire_timeout)
        .sqlx_logging(config.sqlx_logging);

    Database::connect(opt).await
}

/// Convenience wrapper that reads `DATABASE_URL` from the environment and
/// connects immediately.
pub async fn connect_from_env() -> Result<DatabaseConnection, DbErr> {
    let config = DbConfig::from_env()?;
    connect(&config).await
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The local-first / development default: an in-memory SQLite database
    /// requires no external service and should always succeed.
    #[tokio::test]
    async fn connects_to_in_memory_sqlite() {
        let config = DbConfig::from_url("sqlite::memory:");
        let conn = connect(&config).await;
        assert!(conn.is_ok(), "sqlite connection failed: {conn:?}");
    }

    /// The shared / production default: only runs when `TEST_DATABASE_URL`
    /// points at a real PostgreSQL instance (e.g. the service container CI
    /// spins up). Skipped locally so this test suite has no hard external
    /// dependencies.
    #[tokio::test]
    async fn connects_to_postgres_when_configured() {
        let Ok(url) = std::env::var("TEST_DATABASE_URL") else {
            eprintln!("skipping connects_to_postgres_when_configured: TEST_DATABASE_URL not set");
            return;
        };

        let config = DbConfig::from_url(url);
        let conn = connect(&config).await;
        assert!(conn.is_ok(), "postgres connection failed: {conn:?}");
    }
}
