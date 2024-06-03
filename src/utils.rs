#![allow(dead_code)]

use askama::Template;
use rand::Rng;
use rocket::http::Status;
use rocket::response::{content, status};
use std::borrow::Cow;

pub type JsonResponse<'a> = status::Custom<content::RawJson<String>>;

pub fn json_response<'a>(
    status: Status,
    content: Option<String>,
) -> status::Custom<content::RawJson<String>> {
    match content {
        Some(content) => status::Custom(status, content::RawJson(content.into())),
        None => {
            let content = format!(
                "{{ \"status\": \"{}\" }}",
                status.reason().unwrap_or_else(|| "Unknown")
            );
            status::Custom(status, content::RawJson(content))
        }
    }
}

pub type TemplateResponse<'a> = status::Custom<content::RawHtml<String>>;

pub fn template_response<'a>(
    status: Status,
    template: impl Template,
) -> status::Custom<content::RawHtml<String>> {
    match template.render() {
        Ok(content) => status::Custom(status, content::RawHtml(content)),
        Err(_) => {
            let content = format!("<h1>{}</h1>", status.reason().unwrap_or_else(|| "Unknown"));
            status::Custom(status, content::RawHtml(content))
        }
    }
}

pub fn serde_response<'a>(
    _status: Status,
    serialized: Result<String, serde_json::Error>,
) -> JsonResponse<'a> {
    match serialized {
        Ok(json) => json_response(Status::Ok, Some(json)),
        Err(_) => json_response(Status::InternalServerError, None),
    }
}

pub fn gen_id() -> i64 {
    rand::thread_rng().gen_range(1000..9999)
}

pub async fn parse_sql_file(file_path: &str) -> std::io::Result<String> {
    let schema = std::fs::read(file_path)?;
    let as_string = String::from_utf8_lossy(schema.as_slice());
    match as_string {
        Cow::Owned(s) => Ok(s),
        Cow::Borrowed(s) => Ok(s.to_string()),
    }
}
