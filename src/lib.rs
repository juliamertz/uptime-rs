use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateMonitor {
    pub name: String,
    pub ip: String,
    pub port: Option<i64>,
    pub interval: i64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateMonitorPing {
    pub monitor_id: i64,
    pub timestamp: String,
    pub status: String,
}

pub enum DatabaseConnection {
    Rocket(Option<()>),
    Sqlx(sqlx::Pool<sqlx::Sqlite>),
}

#[macro_export]
macro_rules! db_conn {
    ( $a:ident ) => {{
        let conn = $a;
        match conn {
            DatabaseConnection::Rocket(_) => {
                let pool = crate::database::initialize().await;
                DatabaseConnection::Sqlx(pool)
            }
            DatabaseConnection::Sqlx(pool) => DatabaseConnection::Sqlx(pool),
        }
    }};
}
