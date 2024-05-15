use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};

const DATABASE_URL: &str = "sqlite:database.db";

pub async fn initialize() {
    let exists = Sqlite::database_exists(DATABASE_URL)
        .await
        .expect("Failed to check if database exists");

    if !exists {
        Sqlite::create_database(DATABASE_URL)
            .await
            .expect("Failed to create database");

        println!("Database created");
    }

    let pool = SqlitePool::connect(DATABASE_URL)
        .await
        .expect("Failed to connect to database");

    if let Err(msg) = sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS monitor (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            ip TEXT NOT NULL,
            port INTEGER NOT NULL
        );
        "#,
    ).execute(&pool).await {
        eprintln!("Failed to create monitor table: {}", msg);
    };
}
