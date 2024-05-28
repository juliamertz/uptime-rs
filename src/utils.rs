use rand::Rng;
use rocket::http::Status;
use rocket::response::{content, status};

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
