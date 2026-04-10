use axum::body::Body;
use axum::extract::Request;
use axum::http::{header, HeaderValue, Method, Response, StatusCode};
use include_dir::{include_dir, Dir};

static PUBLIC_DIR: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/public");
static ASSETS_DIR: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/assets");
static FIXTURES_DIR: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/fixtures");

pub async fn serve(request: Request<Body>) -> Response<Body> {
    if request.method() != Method::GET && request.method() != Method::HEAD {
        return response(
            StatusCode::METHOD_NOT_ALLOWED,
            "text/plain; charset=utf-8",
            Body::empty(),
        );
    }

    let path = request.uri().path().trim_start_matches('/');
    let file = resolve_file(path);
    let Some((file_path, file)) = file else {
        return response(
            StatusCode::NOT_FOUND,
            "text/plain; charset=utf-8",
            Body::empty(),
        );
    };

    let body = if request.method() == Method::HEAD {
        Body::empty()
    } else {
        Body::from(file.contents().to_vec())
    };

    response(StatusCode::OK, content_type(file_path), body)
}

fn resolve_file(path: &str) -> Option<(&str, &'static include_dir::File<'static>)> {
    if let Some(path) = path.strip_prefix("assets/") {
        return ASSETS_DIR.get_file(path).map(|file| (path, file));
    }

    if let Some(path) = path.strip_prefix("fixtures/") {
        return FIXTURES_DIR.get_file(path).map(|file| (path, file));
    }

    PUBLIC_DIR.get_file(path).map(|file| (path, file))
}

fn response(status: StatusCode, content_type: &'static str, body: Body) -> Response<Body> {
    let mut response = Response::new(body);
    *response.status_mut() = status;
    response
        .headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
    response
}

fn content_type(path: &str) -> &'static str {
    match path.rsplit('.').next().unwrap_or_default() {
        "css" => "text/css; charset=utf-8",
        "html" => "text/html; charset=utf-8",
        "ico" => "image/x-icon",
        "jpg" | "jpeg" => "image/jpeg",
        "js" => "text/javascript; charset=utf-8",
        "json" => "application/json",
        "map" => "application/json",
        "mp3" => "audio/mpeg",
        "mp4" => "video/mp4",
        "pdf" => "application/pdf",
        "png" => "image/png",
        "svg" => "image/svg+xml",
        "txt" => "text/plain; charset=utf-8",
        "wav" => "audio/wav",
        "wasm" => "application/wasm",
        "woff2" => "font/woff2",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        _ => "application/octet-stream",
    }
}
