use http_body_util::{BodyExt, Full};
use hyper::{Request, Response, StatusCode, body::Bytes, header::SET_COOKIE};
use tokio::sync::{mpsc, oneshot};
use tracing::{error, info, warn};

use crate::{
    common::{
        ResponseResult, bad_request_response, extract_requesting_admin_session,
        internal_error_response, ok_response, unauthorized_response,
    },
    state::Message,
};

pub const ADMIN_SESSION_ID_COOKIE_KEY: &str = "coco_admin_session_id";
pub const ADMIN_TOKEN_COOKIE_KEY: &str = "coco_admin_token";

pub type AdminSessionId = usize;

#[derive(Debug, Clone)]
pub struct AdminSession {
    pub id: AdminSessionId,
    pub token: String,
}

#[derive(Debug, serde::Deserialize)]
struct AdminLoginAttemptBody {
    password: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminCreateElectionBody {
    pub name: String,
    pub ballot_items: Vec<String>,
}

pub async fn start_session(
    request: Request<hyper::body::Incoming>,
    to_central_state_authority_sender: mpsc::Sender<Message>,
) -> ResponseResult {
    let body_bytes = match request.into_body().collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(e) => {
            error!("{e:?}");
            return internal_error_response();
        }
    };

    let body: AdminLoginAttemptBody = match serde_json::from_slice(&body_bytes) {
        Ok(body) => body,
        Err(e) => {
            warn!("Bad request: {e:?}");
            return bad_request_response();
        }
    };

    info!("{body:?}");

    const ADMIN_PASSWORD: &str = "abcd";
    if body.password != ADMIN_PASSWORD {
        return unauthorized_response();
    }

    let (answer_sender, answer_receiver) = oneshot::channel();

    if let Err(e) = to_central_state_authority_sender
        .send(Message::AdminStartSession { answer_sender })
        .await
    {
        error!("{e:?}");
        return internal_error_response();
    }

    let new_admin_session = match answer_receiver.await {
        Ok(new_admin_session) => new_admin_session,
        Err(e) => {
            error!("{e:?}");
            return internal_error_response();
        }
    };

    Response::builder()
        .header(
            SET_COOKIE,
            format!(
                "{}={}; Path=/",
                ADMIN_SESSION_ID_COOKIE_KEY, new_admin_session.id
            ),
        )
        .header(
            SET_COOKIE,
            format!(
                "{}={}; Path=/",
                ADMIN_TOKEN_COOKIE_KEY, new_admin_session.token
            ),
        )
        .status(StatusCode::CREATED)
        .body(Full::new(Bytes::new()))
}

pub async fn create_election(
    request: Request<hyper::body::Incoming>,
    to_central_state_authority_sender: mpsc::Sender<Message>,
) -> ResponseResult {
    let Some(requesting_admin_session) = extract_requesting_admin_session(&request) else {
        return unauthorized_response();
    };

    let body_bytes = match request.into_body().collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(e) => {
            error!("{e:?}");
            return internal_error_response();
        }
    };

    let body: AdminCreateElectionBody = match serde_json::from_slice(&body_bytes) {
        Ok(body) => body,
        Err(e) => {
            warn!("Bad request: {e:?}");
            return bad_request_response();
        }
    };

    info!("{body:?}");

    let (answer_sender, answer_receiver) = oneshot::channel();

    if let Err(e) = to_central_state_authority_sender
        .send(Message::AdminCreateElection {
            answer_sender,
            requesting_admin_session,
            admin_create_election_body: body,
        })
        .await
    {
        error!("{e:?}");
        return internal_error_response();
    }

    let answer = match answer_receiver.await {
        Ok(answer) => answer,
        Err(e) => {
            error!("{e:?}");
            return internal_error_response();
        }
    };

    match answer {
        Ok(()) => ok_response(),
        Err(err) => Response::builder()
            .status(err.http_status_code())
            .body(Full::new(Bytes::from_owner(err.to_string()))),
    }
}
