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

pub fn gen_id() -> i64 {
    rand::thread_rng().gen_range(1000..9999)
}
