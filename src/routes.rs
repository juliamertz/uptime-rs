use crate::database::DatabaseModel;
use crate::ping::PingerManager;
use crate::utils::{json_response, serde_response, JsonResponse};
use askama_rocket::Template;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use rocket_db_pools::sqlx::{self, Row};
use rocket_db_pools::{Connection, Database};

//
// monitor_list.html
//
#[derive(Template)]
#[template(path = "components/monitor_list.html")]
pub struct MonitorListComponentTemplate {
    pub monitors: Vec<crate::database::Monitor>,
}

// async fn read(mut db: Connection<Logs>, id: i64) -> Option<String> {
//     sqlx::query("SELECT content FROM logs WHERE id = ?").bind(id)
//         .fetch_one(&mut **db).await
//         .and_then(|r| Ok(r.try_get(0)?))
//         .ok()
// }

#[get("/")]
pub async fn monitor_list<'a>(db: Connection<crate::Logs>) -> crate::utils::TemplateResponse<'a> {
    let monitors = crate::database::Monitor::all(db).await;
    let view = MonitorListComponentTemplate { monitors };
    let html = view.render().unwrap();
    crate::utils::template_response(Status::Ok, html)
}

//
// uptime_graph.html
//
#[derive(Template)]
#[template(path = "components/uptime_graph.html")]
pub struct UptimeGraphTemplate {
    uptime_graph: Option<Vec<crate::database::MonitorPing>>,
    average_response_time: Option<i64>,
    last_response_time: Option<i64>,
    monitor: crate::database::Monitor,
}

fn vec_last<'a, T>(vec: &'a Vec<T>) -> Option<&'a T> {
    vec.last()
}

#[get("/<id>/uptime-graph")]
pub async fn uptime_graph<'a>(id: i64) -> crate::utils::TemplateResponse<'a> {
    let pool = crate::database::initialize().await;
    let uptime_data = crate::database::MonitorPing::last_n(&pool, id, 30).await;
    // Divide by zero bug here, fix later!!
    let average_response_time = uptime_data
        .iter()
        .fold(0, |acc, ping| acc + ping.duration_ms)
        / uptime_data.len() as i64;
    let last_response_time = uptime_data.last().unwrap().duration_ms;

    let view = UptimeGraphTemplate {
        uptime_graph: Some(uptime_data),
        average_response_time: Some(average_response_time),
        last_response_time: Some(last_response_time),
        monitor: crate::database::Monitor::by_id(id, &pool).await.unwrap(),
    };
    let html = view.render().unwrap();
    pool.close().await;
    crate::utils::template_response(Status::Ok, html)
}

//
// index.html
//
#[derive(Template)]
#[template(path = "views/index.html")]
pub struct IndexTemplate<'a> {
    title: &'a str,
    monitors: Vec<crate::database::Monitor>,
}

#[get("/")]
pub async fn index<'a>(mut db: Connection<crate::Logs>) -> crate::utils::TemplateResponse<'a> {
    let pool = crate::database::initialize().await;
    let monitors = crate::database::Monitor::all(db).await;
    pool.close().await;

    let hello = IndexTemplate {
        title: "world",
        monitors,
    };
    let html = hello.render().unwrap();

    crate::utils::template_response(Status::Ok, html)
}

//
// monitor.html
//
#[derive(Template)]
#[template(path = "views/monitor.html")]
pub struct MonitorViewTemplate<'a> {
    title: &'a str,
    monitor: crate::database::Monitor,
}

#[get("/<id>")]
pub async fn monitor_view<'a>(id: i64) -> crate::utils::TemplateResponse<'a> {
    let pool = crate::database::initialize().await;
    let monitor = crate::database::Monitor::by_id(id, &pool).await;

    let response = match monitor {
        Some(monitor) => {
            let view = MonitorViewTemplate {
                title: "Monitor",
                monitor,
            };
            let html = view.render().unwrap();
            crate::utils::template_response(Status::Ok, html)
        }
        None => crate::utils::template_response(Status::NotFound, String::from("Not found")),
    };
    pool.close().await;
    response
}

//
// Json routes
//
#[get("/<id>")]
pub async fn get_monitor<'a>(id: i64) -> JsonResponse<'a> {
    let pool = crate::database::initialize().await;
    let query_result = crate::database::Monitor::by_id(id, &pool).await;
    pool.close().await;

    match query_result {
        Some(monitor) => {
            let serialized = serde_json::to_string(&monitor);
            match serialized {
                Ok(json) => json_response(Status::Ok, Some(json)),
                Err(_) => json_response(Status::InternalServerError, None),
            }
        }
        None => json_response(Status::NotFound, None),
    }
}

#[get("/")]
pub async fn all_monitors<'a>(mut db: Connection<crate::Logs>) -> JsonResponse<'a> {
    let pool = crate::database::initialize().await;
    let monitors = crate::database::Monitor::all(db).await;
    pool.close().await;

    serde_response(Status::Ok, serde_json::to_string(&monitors))
}

#[get("/<monitor_id>/ping/last/<n>")]
pub async fn last_pings<'a>(monitor_id: i64, n: i64) -> JsonResponse<'a> {
    let pool = crate::database::initialize().await;
    let pings = crate::database::MonitorPing::last_n(&pool, monitor_id, n).await;
    pool.close().await;

    serde_response(Status::Ok, serde_json::to_string(&pings))
}

#[post("/", data = "<data>")]
pub async fn create_monitor<'a>(
    data: Json<uptime_rs::CreateMonitor>,
    manager: &State<PingerManager>,
) -> JsonResponse<'a> {
    let pool = crate::database::initialize().await;
    let monitor = crate::database::Monitor {
        interval: data.interval,
        protocol: crate::ping::Protocol::HTTP,
        id: crate::utils::gen_id(),
        name: data.name.clone(),
        ip: data.ip.clone(),
        port: data.port,
        paused: false,
    };

    let response = match monitor.create(&pool).await {
        Ok(result) => serde_response(Status::Created, serde_json::to_string(&result)),
        Err(_) => json_response(Status::InternalServerError, None),
    };

    let interval = monitor.interval.clone();
    manager
        .add_pinger(crate::ping::Pinger::new(monitor, interval, || {}))
        .await;

    pool.close().await;
    response
}
