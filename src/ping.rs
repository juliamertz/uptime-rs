use crate::{database, utils, DatabaseModel};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;
use std::time::{Duration, Instant};
use std::{sync::Arc, thread, time};
// use tokio::sync::Mutex;
use rocket::{futures::lock::Mutex, http::Status};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Protocol {
    HTTP,
    HTTPS,
}

impl Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Protocol::HTTP => write!(f, "http"),
            Protocol::HTTPS => write!(f, "https"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Pinger {
    pub monitor: database::Monitor,
    pub callback: fn(),
    pub enabled: bool,
    last_ping: i64,
}

#[derive(Debug)]
pub struct PingResponse {
    pub is_alive: bool,
    pub status: Status,
    pub duration: Duration,
}

impl Pinger {
    pub fn new(monitor: database::Monitor, timeout_sec: i64, callback: fn()) -> Pinger {
        let enabled = !&monitor.paused;
        Pinger {
            monitor,
            callback,
            enabled,
            last_ping: timeout_sec,
        }
    }

    async fn ping(&self) -> PingResponse {
        let start = Instant::now();
        let response = reqwest::get(&self.monitor.address()).await;
        let duration = start.elapsed();

        return match response {
            Ok(res) => {
                let status = Status::from_code(res.status().as_u16()).unwrap();
                PingResponse {
                    is_alive: res.status().is_success(),
                    status,
                    duration,
                }
            }
            Err(_) => PingResponse {
                is_alive: false,
                status: Status::InternalServerError,
                duration,
            },
        };
    }

    pub async fn tick(&mut self) {
        if self.last_ping >= self.monitor.interval {
            let ping = self.ping().await;
            let pool = database::initialize().await;

            if ping.is_alive {
                let ping = database::MonitorPing {
                    id: utils::gen_id(),
                    monitor_id: self.monitor.id,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    status: Status::from_code(ping.status.code).unwrap_or(Status::ImATeapot),
                    duration_ms: ping.duration.as_millis() as i64,
                    bad: false,
                };

                ping.create(&pool).await.expect("Failed to create ping");

                println!("{} is alive", self.monitor.address());
            } else {
                let ping = database::MonitorPing {
                    id: utils::gen_id(),
                    monitor_id: self.monitor.id,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    status: Status::Ok,
                    duration_ms: ping.duration.as_millis() as i64,
                    bad: true,
                };

                ping.create(&pool).await.expect("Failed to create ping");
                println!("{} is dead", self.monitor.address());
            }

            pool.close().await;
            self.last_ping = 0;
        }

        self.last_ping += 1;
    }
}

#[derive(Debug)]
pub struct PingerManager {
    pub started: bool,
    pub pingers: Arc<Mutex<HashMap<i64, Pinger>>>,
}

impl PingerManager {
    pub fn new() -> PingerManager {
        PingerManager {
            started: false,
            pingers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn add_pinger(&self, pinger: Pinger) {
        self.pingers.lock().await.insert(pinger.monitor.id, pinger);
    }

    pub async fn start(&mut self) {
        let pingers = self.pingers.clone();
        self.started = true;
        tokio::spawn(async move {
            loop {
                let mut gaurd = pingers.lock().await;
                for (_, pinger) in gaurd.iter_mut() {
                    if pinger.enabled {
                        pinger.tick().await;
                    }
                }
                drop(gaurd);

                thread::sleep(time::Duration::from_secs(1));
            }
        });
    }
}
