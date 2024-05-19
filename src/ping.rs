use crate::database;
use serde::{Deserialize, Serialize};
use std::{thread, time};
use tokio::sync::Mutex;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Protocol {
    HTTP,
    HTTPS,
}

impl Protocol {
    pub fn as_str(&self) -> &str {
        match self {
            Protocol::HTTP => "http",
            Protocol::HTTPS => "https",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Pinger {
    pub monitor: database::Monitor,
    pub timeout_sec: u64,
    pub callback: fn(),
    pub enabled: bool,
    last_ping: u64,
}

impl Pinger {
    pub fn new(monitor: database::Monitor, timeout_sec: u64, callback: fn()) -> Pinger {
        Pinger {
            monitor,
            callback,
            timeout_sec,
            enabled: true,
            last_ping: timeout_sec,
        }
    }

    pub async fn ping(&self) -> bool {
        reqwest::get(&self.monitor.address())
            .await
            .map(|res| res.status().is_success())
            .unwrap_or(false)
    }

    pub async fn tick(&mut self) {
        dbg!(&self);
        if self.last_ping >= self.timeout_sec {
            let is_alive = self.ping().await;

            if is_alive {
                println!("{} is alive", self.monitor.address());
            } else {
                println!("{} is dead", self.monitor.address());
            }
            self.last_ping = 0;
            dbg!(&self);
        }

        self.last_ping += 1;
    }
}

#[derive(Debug)]
pub struct PingerManager {
    pub pingers: Mutex<Vec<Pinger>>,
}

impl PingerManager {
    pub fn new() -> PingerManager {
        PingerManager {
            pingers: Mutex::new(Vec::new()),
        }
    }

    pub async fn add_pinger(&mut self, pinger: Pinger) {
        self.pingers.lock().await.push(pinger);
    }

    pub async fn start(&self) {
        loop {
            let mut pingers = self.pingers.lock().await;
            for pinger in pingers.iter_mut() {
                if pinger.enabled {
                    pinger.tick().await;
                }
            }

            thread::sleep(time::Duration::from_secs(1));
        }
    }
}
