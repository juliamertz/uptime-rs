use crate::ping;
use async_trait::async_trait;
use dotenv::dotenv;
use rocket::http::Status;
use serde::{Deserialize, Serialize};
use sqlx::{migrate::MigrateDatabase, Pool, Sqlite, SqlitePool};
use std::env;

pub async fn initialize() -> Pool<Sqlite> {
    dotenv().ok();
    let db_path_env = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let database_url = db_path_env.as_str();

    let exists = Sqlite::database_exists(database_url)
        .await
        .expect("Failed to check if database exists");

    if !exists {
        Sqlite::create_database(database_url)
            .await
            .expect("Failed to create database");

        println!("Database created");
    }

    let pool = SqlitePool::connect(database_url)
        .await
        .expect("Failed to connect to database");

    Monitor::initialize(&pool).await;
    MonitorPing::initialize(&pool).await;

    pool
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Monitor {
    pub id: i64,
    pub name: String,
    pub ip: String,
    pub port: i64,
    pub protocol: ping::Protocol,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MonitorPing {
    pub id: i64,
    pub monitor_id: i64,
    pub timestamp: String,
    pub status: Status,
    pub duration_ms: Option<i64>,
}

impl Monitor {
    pub fn address(&self) -> String {
        format!("{}://{}:{}", self.protocol.as_str(), self.ip, self.port)
    }
}

#[async_trait]
impl DatabaseModel for Monitor {
    async fn initialize(pool: &Pool<Sqlite>) {
        sqlx::query!(
            r#"
            CREATE TABLE IF NOT EXISTS monitor (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                ip TEXT NOT NULL,
                port INTEGER NOT NULL
            );
        "#
        )
        .execute(pool)
        .await
        .expect("Failed to create monitor table.");
    }

    async fn create(&self, pool: &Pool<Sqlite>) -> Result<Self, ()> {
        dbg!(&self.ip);
        let query_result = sqlx::query!(
            r#"
            INSERT INTO monitor (name, ip, port) VALUES (?, ?, ?)
            "#,
            self.name,
            self.ip,
            self.port
        )
        .execute(pool)
        .await;

        match query_result {
            Ok(result) => Ok(Monitor {
                protocol: ping::Protocol::HTTP,
                id: result.last_insert_rowid(),
                name: self.name.clone(),
                ip: self.ip.clone(),
                port: self.port,
            }),
            Err(_) => Err(()),
        }
    }

    async fn by_id(id: i64, pool: &Pool<Sqlite>) -> Option<Self> {
        let query_result = sqlx::query!(
            r#"
            SELECT * FROM monitor WHERE id = ?
            "#,
            id
        )
        .fetch_one(pool)
        .await;

        match query_result {
            Ok(monitor) => Some(Monitor {
                protocol: ping::Protocol::HTTP,
                id: monitor.id,
                name: monitor.name,
                ip: monitor.ip,
                port: monitor.port,
            }),
            Err(_) => None,
        }
    }

    async fn all(pool: &Pool<Sqlite>) -> Vec<Self> {
        let query_result = sqlx::query!(
            r#"
            SELECT * FROM monitor
            "#
        )
        .fetch_all(pool)
        .await;

        match query_result {
            Ok(monitors) => monitors
                .iter()
                .map(|monitor| Monitor {
                    protocol: ping::Protocol::HTTP,
                    id: monitor.id,
                    name: monitor.name.clone(),
                    ip: monitor.ip.clone(),
                    port: monitor.port,
                })
                .collect(),
            Err(_) => Vec::new(),
        }
    }
}

#[async_trait]
impl DatabaseModel for MonitorPing {
    async fn initialize(pool: &Pool<Sqlite>) {
        if let Err(msg) = sqlx::query!(
            r#"
            CREATE TABLE IF NOT EXISTS monitor_ping (
                id INTEGER PRIMARY KEY,
                monitor_id INTEGER NOT NULL,
                status INTEGER NOT NULL,
                timestamp TEXT NOT NULL,
                duration_ms INTEGER,
                FOREIGN KEY (monitor_id) REFERENCES monitor(id)
            );
        "#
        )
        .execute(pool)
        .await
        {
            eprintln!("Failed to create monitor table: {}", msg);
        };
    }

    async fn create(&self, pool: &Pool<Sqlite>) -> Result<Self, ()> {
        let query_result = sqlx::query!(
            r#"
            INSERT INTO monitor_ping (monitor_id, timestamp, status, duration_ms) VALUES (?, ?, ?, ?)
            "#,
            self.monitor_id,
            self.timestamp,
            self.status.code,
            self.duration_ms
        )
        .execute(pool)
        .await;

        match query_result {
            Ok(result) => Ok(MonitorPing {
                id: result.last_insert_rowid(),
                status: self.status.clone(),
                timestamp: self.timestamp.clone(),
                monitor_id: self.monitor_id,
                duration_ms: self.duration_ms,
            }),
            Err(err) => {
                eprintln!("Failed to create monitor ping: {}", err);
                Err(())
            }
        }
    }

    async fn by_id(id: i64, pool: &Pool<Sqlite>) -> Option<Self> {
        if let Ok(monitor_ping) = sqlx::query!(
            r#"
            SELECT * FROM monitor_ping WHERE id = ?
            "#,
            id
        )
        .fetch_one(pool)
        .await
        {
            Some(MonitorPing {
                id: monitor_ping.id,
                status: Status::from_code(monitor_ping.status as u16).expect("Invalid status code"),
                timestamp: monitor_ping.timestamp,
                monitor_id: monitor_ping.monitor_id,
                duration_ms: monitor_ping.duration_ms,
            })
        } else {
            None
        }
    }

    async fn all(pool: &Pool<Sqlite>) -> Vec<Self> {
        if let Ok(monitor_pings) = sqlx::query!(
            r#"
            SELECT * FROM monitor_ping
            "#
        )
        .fetch_all(pool)
        .await
        {
            monitor_pings
                .iter()
                .map(|monitor_ping| MonitorPing {
                    id: monitor_ping.id,
                    status: Status::from_code(monitor_ping.status as u16)
                        .expect("Invalid status code"),
                    timestamp: monitor_ping.timestamp.clone(),
                    monitor_id: monitor_ping.monitor_id,
                    duration_ms: monitor_ping.duration_ms,
                })
                .collect()
        } else {
            Vec::new()
        }
    }
}

#[async_trait]
pub trait DatabaseModel {
    async fn initialize(pool: &Pool<Sqlite>);
    async fn create(&self, pool: &Pool<Sqlite>) -> Result<Self, ()>
    where
        Self: Sized;
    async fn by_id(id: i64, pool: &Pool<Sqlite>) -> Option<Self>
    where
        Self: Sized;
    async fn all(pool: &Pool<Sqlite>) -> Vec<Self>
    where
        Self: Sized;
}
