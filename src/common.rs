use http_body_util::Full;
use hyper::{Request, Response, StatusCode, body::Bytes, header::COOKIE};
use tracing::warn;

use crate::{
    admin::{ADMIN_SESSION_ID_COOKIE_KEY, ADMIN_TOKEN_COOKIE_KEY, AdminSession},
    participant::{PARTICIPANT_ID_COOKIE_KEY, Participant, TOKEN_COOKIE_KEY},
    state::RequestingCredentials,
};

pub type ResponseResult = Result<Response<Full<Bytes>>, hyper::http::Error>;

pub fn ok_response() -> ResponseResult {
    Response::builder()
        .status(StatusCode::OK)
        .body(Full::new(Bytes::new()))
}

pub fn bad_request_response() -> ResponseResult {
    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(Full::new(Bytes::new()))
}

pub fn unauthorized_response() -> ResponseResult {
    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .body(Full::new(Bytes::new()))
}

pub fn internal_error_response() -> ResponseResult {
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Full::new(Bytes::new()))
}

pub fn get_cookie_value<'a>(cookie: &'a [u8], key: &[u8]) -> Option<&'a str> {
    cookie
        .split(|&c| c == b';')
        .find(|&cookie| cookie.trim_ascii().starts_with(key))
        .and_then(|key_value| key_value.split(|&c| c == b'=').nth(1))
        .and_then(|value| str::from_utf8(value).ok())
}

pub fn extract_requesting_participant(
    request: &Request<hyper::body::Incoming>,
) -> Option<Participant> {
    let cookie = &request.headers().get(COOKIE)?;
    let id_str = get_cookie_value(cookie.as_bytes(), PARTICIPANT_ID_COOKIE_KEY.as_bytes())?;
    let id = id_str.parse().ok()?;
    let token = get_cookie_value(cookie.as_bytes(), TOKEN_COOKIE_KEY.as_bytes())?;

    Some(Participant {
        id,
        token: token.to_string(),
    })
}

pub fn extract_requesting_admin_session(
    request: &Request<hyper::body::Incoming>,
) -> Option<AdminSession> {
    let cookie = &request.headers().get(COOKIE)?;
    let id_str = get_cookie_value(cookie.as_bytes(), ADMIN_SESSION_ID_COOKIE_KEY.as_bytes())?;
    let id = id_str.parse().ok()?;
    let token = get_cookie_value(cookie.as_bytes(), ADMIN_TOKEN_COOKIE_KEY.as_bytes())?;

    Some(AdminSession {
        id,
        token: token.to_string(),
    })
}

pub fn extract_requesting_credentials(
    request: &Request<hyper::body::Incoming>,
) -> Option<RequestingCredentials> {
    extract_requesting_admin_session(request)
        .map(RequestingCredentials::Admin)
        .or_else(|| extract_requesting_participant(request).map(RequestingCredentials::Normal))
}
