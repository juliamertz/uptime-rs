mod database;
mod utils;

use rocket::http::Status;
use rocket::tokio;
use utils::{json_response, JsonResponse};

#[macro_use]
extern crate rocket;

#[derive(Debug)]
struct Monitor<'a> {
    id: &'a str,
    name: &'a str,
}

#[get("/<name>/<age>")]
fn wave(name: &str, age: u8) -> String {
    format!("👋 Hello, {} year old named {}!", age, name)
}

#[get("/<id>")]
async fn get_monitor(id: &str) -> JsonResponse {
    let monitor = Monitor {
        id: &id,
        name: "Monitor 1",
    };

    json_response(Status::Ok, Some(&monitor.name))
}

#[launch]
async fn rocket() -> _ {
    database::initialize().await;
    rocket::build()
        .mount("/wave", routes![wave])
        .mount("/monitor", routes![get_monitor])
}
