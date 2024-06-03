use crate::database::{Monitor, MonitorPing};
use askama_rocket::Template;

// Views

#[derive(Template)]
#[template(path = "views/create_monitor.html")]
pub struct CreateMonitorViewTemplate<'a> {
    pub title: &'a str,
}

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
    pub monitor: Monitor,
}

#[derive(Template)]
#[template(path = "components/monitor_status_badge.html")]
pub struct MonitorStatusBadgeTemplate {
    pub uptime_percentage: i64,
    pub monitor: Monitor,
    pub up: bool,
}

#[derive(Template)]
#[template(path = "components/create_monitor_result.html")]
pub struct CreateMonitorResultTemplate {
    pub result: Result<Monitor, sqlx::Error>,
}

#[derive(Template)]
#[template(path = "components/edit_monitor.html")]
pub struct EditMonitorView {
    pub monitor: Monitor,
}
