use crate::database;
use serde::{Deserialize, Serialize};
use std::{thread, time};

#[derive(Debug, Deserialize, Serialize)]
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

#[derive(Debug)]
pub struct Pinger {
    pub monitor: database::Monitor,
    pub timeout_sec: u64,
    pub callback: fn(),
    last_ping: u64,
}

impl Pinger {
    pub fn new(monitor: database::Monitor, timeout_sec: u64, callback: fn()) -> Pinger {
        Pinger {
            monitor,
            callback,
            timeout_sec,
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
        if self.last_ping >= self.timeout_sec {
            let is_alive = self.ping().await;

            if is_alive {
                println!("{} is alive", self.monitor.address());
            } else {
                println!("{} is dead", self.monitor.address());
            }
            self.last_ping = 0;
        }

        self.last_ping += 1;
    }
}

#[derive(Debug)]
pub struct PingerManager {
    pub pingers: Vec<Pinger>,
}

impl PingerManager {
    pub fn new() -> PingerManager {
        PingerManager {
            pingers: Vec::new(),
        }
    }

    pub fn add_pinger(&mut self, pinger: Pinger) {
        self.pingers.push(pinger);
    }

    pub async fn start(&mut self) {
        loop {
            for pinger in &mut self.pingers {
                pinger.tick().await;
            }

            thread::sleep(time::Duration::from_secs(1));
        }
    }
}
