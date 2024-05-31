mod database;
mod ping;
mod routes;
mod utils;

use database::DatabaseModel;
use rocket::fs::FileServer;

#[macro_use]
extern crate rocket;

#[launch]
async fn rocket() -> _ {
    let db = database::initialize().await;
    let mut monitor_pool = ping::PingerManager::new();

    for monitor in database::Monitor::all(&db).await {
        let pinger = ping::Pinger::new(monitor, 3, || {});
        monitor_pool.add_pinger(pinger).await;
    }

    db.close().await;
    monitor_pool.start().await;

    rocket::build()
        .mount(
            "/", //
            routes![routes::index],
        )
        .mount(
            "/monitor",
            routes![routes::monitor_view, routes::uptime_graph],
        )
        .mount(
            "/monitors", //
            routes![routes::monitor_list],
        )
        .mount(
            "/api/monitor",
            routes![
                routes::get_monitor,
                routes::create_monitor,
                routes::last_pings
            ],
        )
        .mount(
            "/api/monitors", //
            routes![routes::all_monitors],
        )
        .mount("/public", FileServer::from("./static"))
        .manage(monitor_pool)
}
