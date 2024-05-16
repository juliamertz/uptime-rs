mod database;
mod utils;

use rocket::http::Status;
use utils::{json_response, JsonResponse};
use rocket::serde::{json::Json, Deserialize, Serialize};

#[macro_use]
extern crate rocket;

#[get("/<name>/<age>")]
fn wave(name: &str, age: u8) -> String {
    format!("👋 Hello, {} year old named {}!", age, name)
}

#[post("/", data = "<monitor>")]
async fn create_monitor<'a>(monitor: Json<database::Monitor>) -> JsonResponse<'a> {
    let pool = database::initialize().await;
    let monitor = database::Monitor {
        id: 1,
        name: "Monitor 1".into(),
        ip: "".into(),
        port: 8080,
    };

    json_response(Status::Ok, Some("YesYes"))
}
#[get("/<id>")]
async fn get_monitor<'a>(id: i64) -> JsonResponse<'a> {
    let monitor = database::Monitor {
        id,
        name: "Monitor 1".into(),
        ip: "127.0.0.1".into(),
        port: 8080,
    };

    let json = serde_json::to_string(&monitor).unwrap().as_str();

    json_response(Status::Ok, Some(json))
}

#[launch]
async fn rocket() -> _ {
    let pool = database::initialize().await;
    rocket::build()
        .mount("/wave", routes![wave])
        .mount("/monitor", routes![get_monitor])
}
