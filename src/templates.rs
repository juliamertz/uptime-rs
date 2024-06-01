use crate::database::{Monitor, MonitorPing};
use askama_rocket::Template;

// Views

#[derive(Template)]
#[template(path = "views/monitor.html")]
pub struct MonitorViewTemplate<'a> {
    pub title: &'a str,
    pub monitor: Monitor,
}

#[derive(Template)]
#[template(path = "views/index.html")]
pub struct IndexTemplate<'a> {
    pub title: &'a str,
    pub monitors: Vec<Monitor>,
}

// Components

#[derive(Template)]
#[template(path = "components/monitor_list.html")]
pub struct MonitorListComponentTemplate {
    pub monitors: Vec<Monitor>,
}

#[derive(Template)]
#[template(path = "components/uptime_graph.html")]
pub struct UptimeGraphTemplate {
    pub uptime_graph: Option<Vec<MonitorPing>>,
    pub average_response_time: Option<i64>,
    pub last_response_time: Option<i64>,
    pub monitor: Monitor,
}

#[derive(Template)]
#[template(path = "components/up_status_card.html")]
pub struct UpStatusCardTemplate {
    pub up: bool,
}
