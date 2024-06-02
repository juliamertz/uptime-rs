use rocket::http::uri::Origin;
use rocket::{http::Status, FromForm};
use serde::{Deserialize, Serialize};

use rocket::http::{ContentType, Header};

#[derive(rocket::Responder)]
#[response(status = 200, content_type = "json")]
struct MyResponder {
    inner: Status,
    more: Header<'static>,
}

pub struct RedirectResponder {
    pub content: String,
    pub redirect_uri: Option<Origin<'static>>,
}

use rocket::request::Request;
use std::io::Cursor;

use rocket::response::{self, Response};

#[rocket::async_trait]
impl<'r> rocket::response::Responder<'r, 'static> for RedirectResponder {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        let redirect_uri = self
            .redirect_uri
            .map(|uri| uri.to_string())
            .unwrap_or("".to_string());

        Response::build()
            .header(ContentType::HTML)
            .header(Header::new("HX-Redirect", redirect_uri))
            .sized_body(self.content.len(), Cursor::new(self.content))
            .ok()

        // match self.redirect_uri {
        //     Some(uri) => response.header(Header::new("HX-Redirect", uri.to_string())),
        //     None => response.header(Header::new("HX-Redirect", "")),
        // }
        // .sized_body(self.content.len(), Cursor::new(self.content))
        // .header(ContentType::HTML)
        // .ok()
    }
}

#[derive(Debug, Deserialize, FromForm, Serialize)]
pub struct CreateMonitor {
    pub name: String,
    pub ip: String,
    pub port: Option<i64>,
    pub interval: i64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateMonitorPing {
    pub monitor_id: i64,
    pub timestamp: String,
    pub status: String,
}
