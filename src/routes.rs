use std::io::Cursor;

use crate::{
    database::{self, DatabaseModel},
    ping::{self, PingerManager},
    templates::*,
    utils::{self, template_response},
};
use askama_rocket::Template;
use rocket::{
    form::{validate::Len, Contextual, Form},
    http::{Header, Status},
    Response, State,
};
use sqlx::{Pool, Sqlite};
use uptime_rs::CreateMonitor;
use utils::{json_response, serde_response, JsonResponse};

//
// monitor_list.html
//
#[get("/")]
pub async fn monitor_list<'a>(pool: &State<Pool<Sqlite>>) -> utils::TemplateResponse<'a> {
    let monitors = database::Monitor::all(&pool).await;
    let view = MonitorListComponentTemplate { monitors };

    utils::template_response(Status::Ok, view.render())
}

//
// uptime_graph.html
//
#[get("/<id>/uptime-graph")]
pub async fn uptime_graph<'a>(pool: &State<Pool<Sqlite>>, id: i64) -> utils::TemplateResponse<'a> {
    let uptime_data = database::MonitorPing::last_n(pool, id, 30).await;

    let view = UptimeGraphTemplate {
        uptime_graph: Some(uptime_data),
        monitor: database::Monitor::by_id(id, pool).await.unwrap(),
    };

    utils::template_response(Status::Ok, view.render())
}

//
// index.html
//
#[get("/")]
pub async fn index<'a>(pool: &State<Pool<Sqlite>>) -> utils::TemplateResponse<'a> {
    let monitors = database::Monitor::all(&pool).await;

    let hello = IndexTemplate {
        title: "world",
        monitors,
    };

    utils::template_response(Status::Ok, hello.render())
}

//
// create_monitor.html
//
#[get("/create")]
pub async fn create_monitor_view<'a>(pool: &State<Pool<Sqlite>>) -> utils::TemplateResponse<'a> {
    // let monitors = database::Monitor::all(&pool).await;

    let hello = CreateMonitorViewTemplate { title: "world" };

    utils::template_response(Status::Ok, hello.render())
}

//
// monitor_status_badge.html
//
#[get("/<id>/status-badge")]
pub async fn monitor_status_badge<'a>(
    pool: &State<Pool<Sqlite>>,
    id: i64,
) -> utils::TemplateResponse<'a> {
    let monitor = database::Monitor::by_id(id, pool).await.unwrap();
    let pings = database::MonitorPing::last_n(pool, id, 1).await;
    let up = match pings.first() {
        Some(ping) => !ping.bad && ping.status.code <= 400,
        None => false,
    };

    let view = MonitorStatusBadgeTemplate { monitor, up };
    utils::template_response(Status::Ok, view.render())
}

//
// monitor.html
//
#[get("/<id>")]
pub async fn monitor_view<'a>(pool: &State<Pool<Sqlite>>, id: i64) -> utils::TemplateResponse<'a> {
    let monitor = database::Monitor::by_id(id, &pool).await;
    let response = match monitor {
        Some(monitor) => {
            let view = MonitorViewTemplate {
                title: "Monitor",
                monitor,
            };
            utils::template_response(Status::Ok, view.render())
        }
        None => utils::template_response(Status::NotFound, Ok(String::from("Not found"))),
    };
    response
}

//
// Json routes
//
#[get("/<id>")]
pub async fn get_monitor<'a>(pool: &State<Pool<Sqlite>>, id: i64) -> JsonResponse<'a> {
    let query_result = database::Monitor::by_id(id, &pool).await;

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
    let monitors = database::Monitor::all(&pool).await;

    serde_response(Status::Ok, serde_json::to_string(&monitors))
}

#[post("/<id>/pause")]
pub async fn pause_monitor(pool: &State<Pool<Sqlite>>, id: i64) -> String {
    let paused = database::Monitor::toggle_paused(id, &pool).await;
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
    let pings = database::MonitorPing::last_n(&pool, monitor_id, n).await;

    serde_response(Status::Ok, serde_json::to_string(&pings))
}

// #[post("/", data = "<form>")]
// fn submit<'r>(form: Form<Contextual<'r, Submit<'r>>>) -> (Status, Template) {
//     let template = match form.value {
//         Some(ref submission) => {
//             println!("submission: {:#?}", submission);
//             Template::render("success", &form.context)
//         }
//         None => Template::render("index", &form.context),
//     };
//
//     (form.context.status(), template)
// }

#[get("/<id>/edit")]
pub async fn edit_monitor_view<'a>(
    pool: &State<Pool<Sqlite>>,
    id: i64,
) -> utils::TemplateResponse<'a> {
    let monitor = database::Monitor::by_id(id, &pool).await.unwrap();
    let view = EditMonitorView { monitor };

    utils::template_response(Status::Ok, view.render())
}

#[put("/<id>/edit", data = "<form>")]
pub async fn update_monitor<'a>(
    pool: &State<Pool<Sqlite>>,
    id: i64,
    form: Form<Contextual<'a, CreateMonitor>>,
) -> String {
    match form.value {
        Some(ref data) => {
            let monitor = database::Monitor {
                interval: data.interval,
                protocol: ping::Protocol::HTTP,
                id,
                name: data.name.clone(),
                ip: data.ip.clone(),
                port: data.port,
                paused: false,
            };

            "aight".into()
            // let response = match monitor.update(&pool).await {
            //     Ok(result) => "ok".into(),
            //     Err(msg) => "err".into(),
            // };
            //
            // response
        }
        None => "no".into(),
    }
}

#[post("/", data = "<form>")]
pub async fn create_monitor<'a>(
    form: Form<Contextual<'a, CreateMonitor>>,
    pool: &State<Pool<Sqlite>>,
    manager: &State<PingerManager>,
) -> String {
    match form.value {
        Some(ref data) => {
            let monitor = database::Monitor {
                interval: data.interval,
                protocol: ping::Protocol::HTTP,
                id: utils::gen_id(),
                name: data.name.clone(),
                ip: data.ip.clone(),
                port: data.port,
                paused: false,
            };

            let response = match monitor.create(&pool).await {
                Ok(result) => "ok".into(),
                Err(msg) => "err".into(),
            };

            let interval = monitor.interval.clone();
            manager
                .add_pinger(ping::Pinger::new(monitor, interval, || {}))
                .await;

            response
        }
        None => {
            "no".into()
            // let msg = "No very bad input!";
            // Response::build()
            //     .status(Status::BadRequest)
            //     .sized_body(msg.len(), Cursor::new(msg))
            //     .finalize()
            //     .body()
            //     .to_string()
            //     .await
            //     .unwrap()
        }
    }
}

// #[post("/", data = "<data>")]
// pub async fn create_monitor<'a>(
//     data: Json<uptime_rs::CreateMonitor>,
//     pool: &State<Pool<Sqlite>>,
//     manager: &State<PingerManager>,
// ) -> JsonResponse<'a> {
//     let monitor = database::Monitor {
//         interval: data.interval,
//         protocol: ping::Protocol::HTTP,
//         id: utils::gen_id(),
//         name: data.name.clone(),
//         ip: data.ip.clone(),
//         port: data.port,
//         paused: false,
//     };
//
//     let response = match monitor.create(&pool).await {
//         Ok(result) => serde_response(Status::Created, serde_json::to_string(&result)),
//         Err(_) => json_response(Status::InternalServerError, None),
//     };
//
//     let interval = monitor.interval.clone();
//     manager
//         .add_pinger(ping::Pinger::new(monitor, interval, || {}))
//         .await;
//
//     response
// }
