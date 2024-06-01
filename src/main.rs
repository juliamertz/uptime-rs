mod database;
mod ping;
mod routes;
mod templates;
mod utils;

use database::DatabaseModel;
use rocket::fs::FileServer;

#[macro_use]
extern crate rocket;

#[launch]
async fn rocket() -> _ {
    let db_pool = database::initialize().await;
    let mut monitor_pool = ping::PingerManager::new();

    for monitor in database::Monitor::all(&db_pool).await {
        let pinger = ping::Pinger::new(monitor, 3, || {});
        monitor_pool.add_pinger(pinger).await;
    }

    monitor_pool.start().await;

    rocket::build()
        .mount(
            "/", //
            routes![routes::index],
        )
        .mount(
            "/monitor",
            routes![
                routes::monitor_view,
                routes::uptime_graph,
                routes::pause_monitor,
                routes::create_monitor,
                routes::edit_monitor_view,
                routes::update_monitor,
                routes::monitor_status_badge,
                routes::create_monitor_view,
            ],
        )
        .mount(
            "/monitors", //
            routes![routes::monitor_list],
        )
        .mount(
            "/api/monitor",
            routes![routes::get_monitor, routes::last_pings],
        )
        .mount(
            "/api/monitors", //
            routes![routes::all_monitors],
        )
        .mount("/public", FileServer::from("./static"))
        .manage(monitor_pool)
        .manage(db_pool)
}
