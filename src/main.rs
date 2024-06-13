pub mod database;
mod ping;
mod routes;
mod templates;
mod time;
mod utils;

use database::DatabaseModel;
use rocket::fs::FileServer;
use rocket_async_compression::CachedCompression;

use std::process::exit;
use time::DateOffset;

#[macro_use]
extern crate rocket;

#[launch]
async fn rocket() -> _ {
    let db_pool = database::initialize().await;
    let mut monitor_pool = ping::PingerManager::new();

    let offset = DateOffset::new(chrono::Duration::days(1));
    dbg!(&offset);
    let normalized = offset.normalize();
    dbg!(&normalized);

    // let result = database::MonitorPing::between(&db_pool, 10, offset, 1000)
    //     .await
    //     .unwrap();
    //
    // dbg!(&result, result.len());
    // exit(0);

    for monitor in database::Monitor::all(&db_pool).await.unwrap() {
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
                routes::delete_monitor
            ],
        )
        .mount(
            "/monitors", //
            routes![routes::monitor_list],
        )
        .mount(
            "/api/monitor",
            routes![
                // routes::get_monitor, //
                routes::last_pings
            ],
        )
        // .mount(
        //     "/api/monitors", //
        //     routes![routes::all_monitors],
        // )
        .mount("/public", FileServer::from("./static"))
        .attach(CachedCompression::path_suffix_fairing(vec![
            ".js".into(),
            ".css".into(),
        ]))
        .manage(monitor_pool)
        .manage(db_pool)
}
