use rocket::{futures::lock::MutexGuard, http::Status};
use sqlx::{Pool, Sqlite};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{database, database::DatabaseModel};
use chrono;
use std::time::Instant;

pub async fn ping(address: &str) -> Result<database::MonitorPing, ()> {
    let start = Instant::now();

    match reqwest::get(address).await {
        Ok(response) => {
            let duration = start.elapsed();
            Ok(database::MonitorPing {
                status: Status::new(response.status().as_u16()),
                duration_ms: Some(duration.as_millis() as i64),
                timestamp: chrono::Utc::now().to_rfc3339(),
                id: 1,
                monitor_id: 1,
            })
        }
        Err(_) => Err(()),
    }
}

pub struct Ticker {
    pub interval: i64,
    pub monitor: database::Monitor,
}

impl Ticker {
    fn new(interval: i64, monitor: database::Monitor) -> Self {
        Self { interval, monitor }
    }

    async fn tick(&self, pool: &Pool<Sqlite>) {
        let ping = ping(&self.monitor.ip).await.unwrap();
        ping.create(&pool).await.unwrap();
    }
}

struct TickerManager {
    tickers: Arc<Mutex<Vec<Ticker>>>,
}

impl TickerManager {
    async fn new() -> Self {
        TickerManager {
            tickers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    async fn add(&self, ticker: Ticker) {
        let mut tickers = self.tickers.lock().await;
        tickers.push(ticker);
    }

    async fn start_ticker(&self, ticker: Arc<Mutex<MutexGuard<'static, Ticker>>>) {
        let pool = database::initialize().await;
        let test = ticker.lock().await;

        self.tickers.lock().await.push(test);
        tokio::spawn(async move {
            loop {
                dbg!("Ticking");
                ticker.lock().await.tick(&pool).await;
                tokio::time::sleep(tokio::time::Duration::from_secs(
                    ticker.lock().await.interval as u64,
                ))
                .await;
            }
        });
    }

    async fn test_add(&self) {
        let test_ticker = Ticker::new(
            5,
            database::Monitor {
                id: 1,
                name: "Test".to_string(),
                ip: "https://www.google.com".to_string(),
                port: 80,
            },
        );

        test_ticker
            .start_ticker(Arc::new(Mutex::new(test_ticker)))
            .await;

        // self.add(test_ticker).await;
    }
}
