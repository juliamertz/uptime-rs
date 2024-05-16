mod database;
mod utils;

use rocket::http::Status;
use utils::{json_response, JsonResponse};

#[macro_use]
extern crate rocket;

#[get("/<name>/<age>")]
fn wave(name: &str, age: u8) -> String {
    format!("👋 Hello, {} year old named {}!", age, name)
}

#[get("/<id>")]
 async fn get_monitor<'a>(id: i64) -> JsonResponse<'a>{
    let monitor = database::Monitor {
        id,
        name: "Monitor 1".into(),
        ip: "127.0.0.1".into(),
        port: 8080
    };

    json_response(Status::Ok, Some("YesYes"))
}

#[launch]
async fn rocket() -> _ {
    database::initialize().await;
    rocket::build()
        .mount("/wave", routes![wave])
        .mount("/monitor", routes![get_monitor])
}
