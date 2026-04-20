use axum::{
    body::Body,
    http::{HeaderValue, StatusCode, Uri, header},
    response::{IntoResponse, Response},
};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../dashboard/out"]
struct DashboardAssets;

fn candidate_paths(rel: &str) -> Vec<String> {
    let rel = rel.split('?').next().unwrap_or("").trim_matches('/');

    if rel.is_empty() {
        return vec!["index.html".into()];
    }

    if rel.contains('.') && !rel.ends_with('/') {
        return vec![rel.into()];
    }

    vec![format!("{rel}.html"), format!("{rel}/index.html")]
}

fn response_bytes(
    status: StatusCode,
    path_key: &str,
    body: Vec<u8>,
) -> Response {
    let guessed = mime_guess::from_path(path_key).first_or_octet_stream();
    let ct = HeaderValue::from_str(guessed.as_ref()).unwrap_or_else(|_| {
        HeaderValue::from_static("application/octet-stream")
    });

    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, ct)
        .body(Body::from(body))
        .unwrap()
}

fn resolve(rel: &str) -> Response {
    if rel.split('/').any(|s| s == "..") {
        return StatusCode::NOT_FOUND.into_response();
    }

    for path_key in candidate_paths(rel) {
        if let Some(file) = DashboardAssets::get(&path_key) {
            return response_bytes(
                StatusCode::OK,
                &path_key,
                file.data.into_owned(),
            );
        }
    }

    if let Some(file) = DashboardAssets::get("404.html") {
        return response_bytes(
            StatusCode::NOT_FOUND,
            "404.html",
            file.data.into_owned(),
        );
    }

    StatusCode::NOT_FOUND.into_response()
}

#[inline(always)]
pub async fn index() -> impl IntoResponse {
    resolve("")
}

#[inline(always)]
pub async fn serve(uri: Uri) -> impl IntoResponse {
    let rel =
        uri.path().trim_start_matches('/').split('?').next().unwrap_or("");
    resolve(rel)
}
