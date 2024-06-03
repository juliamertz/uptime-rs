use crate::{
    ping::{self, PingerManager},
    utils,
};
use async_trait::async_trait;
use dotenv::dotenv;
use rocket::{http::Status, State};
use serde::{Deserialize, Serialize};
use sqlx::{migrate::MigrateDatabase, Pool, Sqlite, SqlitePool};

pub async fn initialize() -> Pool<Sqlite> {
    dotenv().ok();
    let db_path_env = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
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

impl Monitor {
    pub fn get_average_ping_duration(pings: &Vec<crate::database::MonitorPing>) -> i64 {
        if pings.len() == 0 {
            return 0;
        }
        pings.iter().fold(0, |acc, ping| acc + ping.duration_ms) / pings.len() as i64
    }

    pub async fn get_uptime_percentage(&self, pool: &Pool<Sqlite>) -> i64 {
        let pings = MonitorPing::last_n(pool, self.id, 30).await;
        let total_pings = pings.len() as i64;
        let bad_pings = pings.iter().filter(|ping| ping.bad).count() as i64;

        if total_pings == 0 {
            return 100;
        }

        ((total_pings - bad_pings) * 100) / total_pings
    }

    pub async fn is_paused(id: i64, pool: &Pool<Sqlite>) -> bool {
        let query_result = sqlx::query!(
            r#"
            SELECT paused FROM monitor WHERE id = ? LIMIT 1
            "#,
            id
        )
        .fetch_one(pool)
        .await;

        match query_result {
            Ok(monitor) => monitor.paused.to_bool(),
            Err(_) => false,
        }
    }

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

    pub async fn update(&self, pool: &Pool<Sqlite>) -> Result<&Self, sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE monitor SET name = ?, ip = ?, port = ?, interval = ? WHERE id = ?
            "#,
            self.name,
            self.ip,
            self.port,
            self.interval,
            self.id
        )
        .execute(pool)
        .await?;

        Ok(self)
    }

    // pub async fn is_up(self, db: &Pool<Sqlite>) -> bool {
    //     MonitorPing::last_n(db, self.id, 1)
    //         .await
    //         .first()
    //         .map(|ping| ping.bad)
    //         .unwrap_or(false)
    // }

    pub async fn toggle_paused(
        id: i64,
        pool: &Pool<Sqlite>,
        pinger_manager: &State<PingerManager>,
    ) -> Result<bool, sqlx::Error> {
        let monitor = Monitor::by_id(id, pool).await.unwrap();
        let paused = !monitor.paused;
        sqlx::query!(
            r#"
            UPDATE monitor SET paused = ? WHERE id = ?
            "#,
            paused,
            id
        )
        .execute(pool)
        .await?;

        let mut pingers = pinger_manager.pingers.lock().await;
        match pingers.get_mut(&id) {
            Some(pinger) => {
                pinger.enabled = !paused;
                Ok(paused)
            }
            None => Err(sqlx::Error::RowNotFound),
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
            self.interval,
        )
        .execute(pool)
        .await?;

        Ok(Monitor {
            protocol: ping::Protocol::HTTP,
            id: query_result.last_insert_rowid(),
            name: self.name.clone(),
            ip: self.ip.clone(),
            port: self.port,
            interval: self.interval,
            paused: self.paused,
        })
    }

    async fn by_id(id: i64, pool: &Pool<Sqlite>) -> Result<Self, sqlx::Error> {
        let monitor = sqlx::query!(
            r#"
            SELECT * FROM monitor WHERE id = ?
            "#,
            id
        )
        .fetch_one(pool)
        .await?;

        Ok(Monitor {
            protocol: ping::Protocol::HTTP,
            id: monitor.id,
            name: monitor.name,
            ip: monitor.ip,
            port: monitor.port,
            interval: monitor.interval,
            paused: monitor.paused.to_bool(),
        })
    }

    async fn all(pool: &Pool<Sqlite>) -> Result<Vec<Self>, sqlx::Error> {
        let query_result = sqlx::query!(
            r#"
            SELECT * FROM monitor
            "#
        )
        .fetch_all(pool)
        .await?;

        Ok(query_result
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
            .collect())
    }

    async fn delete(id: i64, pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            DELETE FROM monitor_ping WHERE monitor_id = ?
            "#,
            id
        )
        .execute(pool)
        .await?;

        sqlx::query!(
            r#"
            DELETE FROM monitor WHERE id = ?
            "#,
            id
        )
        .execute(pool)
        .await?;

        Ok(())
    }
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
    // pub async fn between_dates(pool: &Pool<Sqlite>, dates: (String,String))

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

    async fn by_id(id: i64, pool: &Pool<Sqlite>) -> Result<Self, sqlx::Error> {
        let query_result = sqlx::query!(
            r#"
            SELECT * FROM monitor_ping WHERE id = ?
            "#,
            id
        )
        .fetch_one(pool)
        .await?;

        Ok(MonitorPing {
            id: query_result.id,
            status: Status::from_code(query_result.status as u16).expect("Invalid status code"),
            timestamp: query_result.timestamp,
            monitor_id: query_result.monitor_id,
            duration_ms: query_result.duration_ms,
            bad: query_result.bad.to_bool(),
        })
    }

    async fn all(pool: &Pool<Sqlite>) -> Result<Vec<Self>, sqlx::Error> {
        let query_result = sqlx::query!(
            r#"
            SELECT * FROM monitor_ping
            "#
        )
        .fetch_all(pool)
        .await?;

        Ok(query_result
            .iter()
            .map(|monitor_ping| MonitorPing {
                id: monitor_ping.id,
                status: Status::from_code(monitor_ping.status as u16).expect("Invalid status code"),
                timestamp: monitor_ping.timestamp.clone(),
                monitor_id: monitor_ping.monitor_id,
                duration_ms: monitor_ping.duration_ms,
                bad: monitor_ping.bad.to_bool(),
            })
            .collect())
    }

    async fn delete(id: i64, pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            DELETE FROM monitor_ping WHERE id = ?
            "#,
            id
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MonitorStats {
    id: i64,
    average_response_ms: i64,
    uptime_percentage_24h: i64,
    uptime_percentage_30d: i64,
}

#[async_trait]
impl DatabaseModel for MonitorStats {
    async fn initialize(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
        let schema = utils::parse_sql_file("schemas/monitor_stats.sql").await?;
        sqlx::query(&schema).execute(pool).await?;

        Ok(())
    }

    async fn create(&self, pool: &Pool<Sqlite>) -> Result<Self, sqlx::Error> {
        let query_result = sqlx::query!(
            r#"
            INSERT INTO monitor_stats (average_response_ms, uptime_percentage_24h, uptime_percentage_30d) VALUES (?, ?, ?)
            "#,
            self.average_response_ms,
            self.uptime_percentage_24h,
            self.uptime_percentage_30d
        )
        .execute(pool)
        .await;

        match query_result {
            Ok(result) => Ok(MonitorStats {
                id: result.last_insert_rowid(),
                average_response_ms: self.average_response_ms,
                uptime_percentage_24h: self.uptime_percentage_24h,
                uptime_percentage_30d: self.uptime_percentage_30d,
            }),
            Err(err) => Err(err),
        }
    }

    async fn by_id(id: i64, pool: &Pool<Sqlite>) -> Result<Self, sqlx::Error> {
        let query_result = sqlx::query!(
            r#"
            SELECT * FROM monitor_stats WHERE id = ?
            "#,
            id
        )
        .fetch_one(pool)
        .await?;

        Ok(MonitorStats {
            id: query_result.id,
            average_response_ms: query_result.average_response_ms,
            uptime_percentage_24h: query_result.uptime_percentage_24h,
            uptime_percentage_30d: query_result.uptime_percentage_30d,
        })
    }

    async fn all(pool: &Pool<Sqlite>) -> Result<Vec<Self>, sqlx::Error> {
        let query_result = sqlx::query!(
            r#"
            SELECT * FROM monitor_stats
            "#
        )
        .fetch_all(pool)
        .await?;

        Ok(query_result
            .iter()
            .map(|monitor_stats| MonitorStats {
                id: monitor_stats.id,
                average_response_ms: monitor_stats.average_response_ms,
                uptime_percentage_24h: monitor_stats.uptime_percentage_24h,
                uptime_percentage_30d: monitor_stats.uptime_percentage_30d,
            })
            .collect())
    }

    async fn delete(id: i64, pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            DELETE FROM monitor_stats WHERE id = ?
            "#,
            id
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}

#[async_trait]
pub trait DatabaseModel {
    async fn initialize(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error>;
    async fn create(&self, pool: &Pool<Sqlite>) -> Result<Self, sqlx::Error>
    where
        Self: Sized;
    async fn by_id(id: i64, pool: &Pool<Sqlite>) -> Result<Self, sqlx::Error>
    where
        Self: Sized;
    async fn all(pool: &Pool<Sqlite>) -> Result<Vec<Self>, sqlx::Error>
    where
        Self: Sized;
    async fn delete(id: i64, pool: &Pool<Sqlite>) -> Result<(), sqlx::Error>
    where
        Self: Sized;
}
