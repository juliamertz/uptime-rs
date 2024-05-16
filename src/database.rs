use async_trait::async_trait;
use dotenv::dotenv;
use sqlx::{migrate::MigrateDatabase, Pool, Sqlite, SqlitePool};
use std::env;

pub async fn initialize() {
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
}

#[derive(Debug)]
pub struct Monitor<'a> {
    pub id: i64,
    pub name: &'a str,
    pub ip: &'a str,
    pub port: i32,
}

pub struct MonitorPing<'a> {
    pub id: i64,
    pub monitor_id: &'a i64,
    pub timestamp: &'a str,
    pub status: &'a str,
}

#[async_trait]
impl DatabaseModel for Monitor<'_> {
    async fn initialize(pool: &Pool<Sqlite>) {
        if let Err(msg) = sqlx::query!(
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
        {
            eprintln!("Failed to create monitor table: {}", msg);
        };
    }

    async fn create(&self, pool: &Pool<Sqlite>) {
        if let Err(msg) = sqlx::query!(
            r#"
            INSERT INTO monitor (name, ip, port) VALUES (?, ?, ?)
            "#,
            self.name,
            self.ip,
            self.port
        )
        .execute(pool)
        .await
        {
            eprintln!("Failed to create monitor: {}", msg);
        };
    }

    async fn by_id(id: i64, pool: &Pool<Sqlite>) -> Option<Self> {
        if let Err(msg) = sqlx::query!(
            r#"
            SELECT * FROM monitor WHERE id = ?
            "#,
            id
        )
        .fetch_one(pool)
        .await
        {
            eprintln!("Failed to fetch monitor: {}", msg);
        };
        None
    }
}

#[async_trait]
impl DatabaseModel for MonitorPing<'_> {
    async fn initialize(pool: &Pool<Sqlite>) {
        if let Err(msg) = sqlx::query!(
            r#"
            create table if not exists monitor_ping (
                id integer primary key,
                monitor_id integer not null,
                timestamp text not null,
                status text not null,
                foreign key (monitor_id) references monitor(id)
            );
        "#
        )
        .execute(pool)
        .await
        {
            eprintln!("Failed to create monitor table: {}", msg);
        };
    }

    async fn create(&self, pool: &Pool<Sqlite>) {
        if let Err(msg) = sqlx::query!(
            r#"
            INSERT INTO monitor_ping (monitor_id, timestamp, status) VALUES (?, ?, ?)
            "#,
            self.monitor_id,
            self.timestamp,
            self.status
        )
        .execute(pool)
        .await
        {
            eprintln!("Failed to create monitor: {}", msg);
        };
    }

    async fn by_id(id: i64, pool: &Pool<Sqlite>) -> Option<Self> {
        if let Ok(monitor_ping) = sqlx::query!(
            r#"
            SELECT * FROM monitor_ping WHERE id = ?
            "#,
            id
        )
        .fetch_one(pool)
        .await {
            Some(MonitorPing{
                id: monitor_ping.id,
                status: monitor_ping.status.as_str(), 
                timestamp: monitor_ping.timestamp.as_str(),
                monitor_id: &monitor_ping.monitor_id
            })
        }
        else {
            None
        }
    }
}

#[async_trait]
pub trait DatabaseModel {
    async fn initialize(pool: &Pool<Sqlite>);
    async fn create(&self, pool: &Pool<Sqlite>);
    async fn by_id(id: i64, pool: &Pool<Sqlite>) -> Option<Self> where Self: Sized;
    // fn all(&self);
}
