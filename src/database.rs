use crate::{ping, utils};
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
    }

    let pool = SqlitePool::connect(database_url)
        .await
        .expect("Failed to connect to database");

    Monitor::initialize(&pool)
        .await
        .expect("Failed to initialize monitor table");
    MonitorPing::initialize(&pool)
        .await
        .expect("Failed to initialize monitor_ping table");

    pool
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Monitor {
    pub id: i64,
    pub name: String,
    pub ip: String,
    pub port: Option<i64>,
    pub protocol: ping::Protocol,
    pub interval: i64,
    pub paused: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MonitorPing {
    pub id: i64,
    pub monitor_id: i64,
    pub timestamp: String,
    pub status: Status,
    pub duration_ms: i64,
    pub bad: bool,
}

impl Monitor {
    pub fn hostname(&self) -> String {
        match self.port {
            Some(port) => format!("{}:{}", self.ip, port),
            None => format!("{}", self.ip),
        }
    }
    pub fn address(&self) -> String {
        match self.port {
            Some(port) => format!("{}://{}:{}", self.protocol, self.ip, port),
            None => format!("{}://{}", self.protocol, self.ip),
        }
    }

    pub async fn toggle_paused(id: i64, pool: &Pool<Sqlite>) -> Result<bool, sqlx::Error> {
        let monitor = Monitor::by_id(id, pool).await.unwrap();
        let paused = !monitor.paused;
        let query_result = sqlx::query!(
            r#"
            UPDATE monitor SET paused = ? WHERE id = ?
            "#,
            paused,
            id
        )
        .execute(pool)
        .await;

        match query_result {
            Ok(_) => Ok(paused),
            Err(e) => Err(e),
        }
    }
}

#[async_trait]
impl DatabaseModel for Monitor {
    async fn initialize(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
        let schema = utils::parse_sql_file("schemas/monitor.sql").await?;
        sqlx::query(&schema).execute(pool).await?;

        Ok(())
    }

    async fn create(&self, pool: &Pool<Sqlite>) -> Result<Self, sqlx::Error> {
        let query_result = sqlx::query!(
            r#"
            INSERT INTO monitor (name, ip, port, interval) VALUES (?, ?, ?, ?)
            "#,
            self.name,
            self.ip,
            self.port,
            self.interval
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
                interval: self.interval,
                paused: self.paused,
            }),
            Err(e) => Err(e),
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
                interval: monitor.interval,
                paused: monitor.paused.to_bool(),
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
                    interval: monitor.interval,
                    paused: monitor.paused.to_bool(),
                })
                .collect(),
            Err(_) => Vec::new(),
        }
    }
}

impl MonitorPing {
    // async fn last_ping(&self, pool: &Pool<Sqlite>) -> Option<Self> {
    //     let query_result = sqlx::query!(
    //         r#"
    //         SELECT * FROM monitor_ping WHERE monitor_id = ? ORDER BY timestamp DESC LIMIT 1
    //         "#,
    //         self.monitor_id
    //     )
    //     .fetch_one(pool)
    //     .await;
    //
    //     match query_result {
    //         Ok(monitor_ping) => Some(MonitorPing {
    //             id: monitor_ping.id,
    //             status: Status::from_code(monitor_ping.status as u16).expect("Invalid status code"),
    //             timestamp: monitor_ping.timestamp.clone(),
    //             monitor_id: monitor_ping.monitor_id,
    //             duration_ms: monitor_ping.duration_ms,
    //             bad: monitor_ping.bad.to_bool(),
    //         }),
    //         Err(_) => None,
    //     }
    // }

    pub async fn last_n(pool: &Pool<Sqlite>, monitor_id: i64, n: i64) -> Vec<Self> {
        if let Ok(monitor_pings) = sqlx::query!(
            r#"
            SELECT * FROM monitor_ping WHERE monitor_id=? ORDER BY timestamp DESC LIMIT ?;
            "#,
            monitor_id,
            n
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
                    bad: monitor_ping.bad.to_bool(),
                })
                .collect()
        } else {
            Vec::new()
        }
    }
}

pub trait ToBool {
    fn to_bool(&self) -> bool;
}

impl ToBool for i64 {
    fn to_bool(&self) -> bool {
        match self {
            0 => false,
            1 => true,
            _ => panic!("Bad value for bool conversion: {}", self),
        }
    }
}

#[async_trait]
impl DatabaseModel for MonitorPing {
    async fn initialize(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
        let schema = utils::parse_sql_file("schemas/monitor_ping.sql").await?;
        sqlx::query(&schema).execute(pool).await?;

        Ok(())
    }

    async fn create(&self, pool: &Pool<Sqlite>) -> Result<Self, sqlx::Error> {
        let query_result = sqlx::query!(
            r#"
            INSERT INTO monitor_ping (monitor_id, timestamp, status, duration_ms, bad) VALUES (?, ?, ?, ?, ?)
            "#,
            self.monitor_id,
            self.timestamp,
            self.status.code,
            self.duration_ms,
            self.bad
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
                bad: self.bad,
            }),
            Err(err) => Err(err),
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
                bad: monitor_ping.bad.to_bool(),
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
                    bad: monitor_ping.bad.to_bool(),
                })
                .collect()
        } else {
            Vec::new()
        }
    }
}

#[async_trait]
pub trait DatabaseModel {
    async fn initialize(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error>;
    async fn create(&self, pool: &Pool<Sqlite>) -> Result<Self, sqlx::Error>
    where
        Self: Sized;
    async fn by_id(id: i64, pool: &Pool<Sqlite>) -> Option<Self>
    where
        Self: Sized;
    async fn all(pool: &Pool<Sqlite>) -> Vec<Self>
    where
        Self: Sized;
}
