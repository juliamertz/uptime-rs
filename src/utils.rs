use rocket::http::Status;
use rocket::response::{content, status};

pub type JsonResponse<'a> = status::Custom<content::RawJson<&'a str>>;

pub fn json_response<'a>(
    status: Status,
    content: Option<&'a str>,
) -> status::Custom<content::RawJson<&'a str>> {
    match content {
        Some(content) => status::Custom(status, content::RawJson(content)),
        None => status::Custom(status, content::RawJson("{ \"status\": true }")),
    }
}
