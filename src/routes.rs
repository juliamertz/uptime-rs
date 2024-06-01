use crate::database::DatabaseModel;
use crate::ping::PingerManager;
use crate::templates::{
    IndexTemplate, MonitorListComponentTemplate, MonitorViewTemplate, UptimeGraphTemplate,
};
use crate::utils::{json_response, serde_response, JsonResponse};
use askama_rocket::Template;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;

use sqlx::{Pool, Sqlite};

//
// monitor_list.html
//
#[get("/")]
pub async fn monitor_list<'a>(pool: &State<Pool<Sqlite>>) -> crate::utils::TemplateResponse<'a> {
    let monitors = crate::database::Monitor::all(&pool).await;
    let view = MonitorListComponentTemplate { monitors };

    crate::utils::template_response(Status::Ok, view.render())
}

//
// uptime_graph.html
//
#[get("/<id>/uptime-graph")]
pub async fn uptime_graph<'a>(
    pool: &State<Pool<Sqlite>>,
    id: i64,
) -> crate::utils::TemplateResponse<'a> {
    let uptime_data = crate::database::MonitorPing::last_n(pool, id, 30).await;
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
        monitor: crate::database::Monitor::by_id(id, pool).await.unwrap(),
    };

    crate::utils::template_response(Status::Ok, view.render())
}

//
// index.html
//
#[get("/")]
pub async fn index<'a>(pool: &State<Pool<Sqlite>>) -> crate::utils::TemplateResponse<'a> {
    let monitors = crate::database::Monitor::all(&pool).await;

    let hello = IndexTemplate {
        title: "world",
        monitors,
    };

    crate::utils::template_response(Status::Ok, hello.render())
}

//
// up_status_card.html
//

// #[get("/<id>/up-status")]
// pub async fn up_status_card<'a>(
//     pool: &State<Pool<Sqlite>>,
//     id: i64,
// ) -> crate::utils::TemplateResponse<'a> {
//     let monitor = crate::database::Monitor::by_id(id, &pool).await.unwrap();
//     let pings = crate::database::MonitorPing::last_n(&pool, id, 1).await;
//     // let up = match pings.first() {
//     //     Some(ping) => ping.status == crate::ping::Status::UP,
//     //     None => false,
//     // };
//
//     let view = UpStatusCardTemplate { up };
//     let html = view.render().unwrap();
//     crate::utils::template_response(Status::Ok, html)
// }

//
// monitor.html
//

#[get("/<id>")]
pub async fn monitor_view<'a>(
    pool: &State<Pool<Sqlite>>,
    id: i64,
) -> crate::utils::TemplateResponse<'a> {
    let monitor = crate::database::Monitor::by_id(id, &pool).await;
    dbg!(&pool);

    let response = match monitor {
        Some(monitor) => {
            let view = MonitorViewTemplate {
                title: "Monitor",
                monitor,
            };
            crate::utils::template_response(Status::Ok, view.render())
        }
        None => crate::utils::template_response(Status::NotFound, Ok(String::from("Not found"))),
    };
    response
}

//
// Json routes
//
#[get("/<id>")]
pub async fn get_monitor<'a>(pool: &State<Pool<Sqlite>>, id: i64) -> JsonResponse<'a> {
    let query_result = crate::database::Monitor::by_id(id, &pool).await;

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
pub async fn all_monitors<'a>(pool: &State<Pool<Sqlite>>) -> JsonResponse<'a> {
    let monitors = crate::database::Monitor::all(&pool).await;

    serde_response(Status::Ok, serde_json::to_string(&monitors))
}

#[post("/<id>/pause")]
pub async fn pause_monitor(pool: &State<Pool<Sqlite>>, id: i64) -> String {
    let paused = crate::database::Monitor::toggle_paused(id, &pool).await;
    // add header for htmx to refresh on success
    match paused {
        Ok(paused) => match paused {
            true => "Resume".into(),
            false => "Pause".into(),
        },

        Err(_) => "Error".into(),
    }
}

#[get("/<monitor_id>/ping/last/<n>")]
pub async fn last_pings<'a>(
    pool: &State<Pool<Sqlite>>,
    monitor_id: i64,
    n: i64,
) -> JsonResponse<'a> {
    let pings = crate::database::MonitorPing::last_n(&pool, monitor_id, n).await;

    serde_response(Status::Ok, serde_json::to_string(&pings))
}

#[post("/", data = "<data>")]
pub async fn create_monitor<'a>(
    data: Json<uptime_rs::CreateMonitor>,
    pool: &State<Pool<Sqlite>>,
    manager: &State<PingerManager>,
) -> JsonResponse<'a> {
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

    response
}
