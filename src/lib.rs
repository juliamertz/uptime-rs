use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateMonitor {
    pub name: String,
    pub ip: String,
    pub port: i64,
    pub interval: i64,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct CreateMonitorPing {
    pub monitor_id: i64,
    pub timestamp: String,
    pub status: String,
}
