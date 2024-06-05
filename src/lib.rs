use rocket::http::uri::Origin;
use rocket::http::{ContentType, Header};
use rocket::request::Request;
use rocket::response::{self, Response};
use rocket::{http::Status, FromForm};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::io::Cursor;

mod utils;

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

pub struct RedirectResponder {
    pub content: String,
    pub redirect_uri: Option<Origin<'static>>,
}

pub type RedirectResult = Result<RedirectResponder, AppError>;
pub type TemplateResult<'a> = Result<utils::TemplateResponse<'a>, AppError>;
pub type JsonResult<'a> = Result<utils::JsonResponse<'a>, AppError>;

pub struct AppError {
    pub status: Status,
    pub message: String,
}

impl From<askama_rocket::Error> for AppError {
    fn from(cause: askama_rocket::Error) -> Self {
        AppError {
            status: Status::InternalServerError,
            message: format!("Error rendering template: {}", cause),
        }
    }
}
impl From<std::io::Error> for AppError {
    fn from(cause: std::io::Error) -> Self {
        AppError {
            status: Status::InternalServerError,
            message: format!("IO Error: {}", cause),
        }
    }
}

impl From<sqlx::Error> for AppError {
    fn from(cause: sqlx::Error) -> Self {
        AppError {
            status: Status::InternalServerError,
            message: format!("Database Error: {}", cause),
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "An Error Occurred, Please Try Again!") // user-facing output
    }
}

impl fmt::Debug for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{ file: {}, line: {} }}", file!(), line!()) // programmer-facing output
    }
}

impl<'r> rocket::response::Responder<'r, 'static> for AppError {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        Response::build()
            .header(ContentType::HTML)
            .sized_body(self.message.len(), Cursor::new(self.message))
            .ok()
    }
}

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
    }
}
