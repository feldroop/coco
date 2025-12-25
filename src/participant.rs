use http_body_util::{BodyExt, Full};
use hyper::{Request, Response, StatusCode, body::Bytes, header::SET_COOKIE};
use tokio::sync::{mpsc, oneshot};
use tracing::{error, info, warn};

use crate::{
    common::{
        ResponseResult, bad_request_response, internal_error_response, unauthorized_response,
    },
    state::Message,
};

pub const PARTICIPANT_ID_COOKIE_KEY: &str = "coco_participant_id";
pub const ACCESS_TOKEN_COOKIE_KEY: &str = "coco_access_token";

pub type ParticipantId = usize;

#[derive(Debug, serde::Serialize, Clone)]
pub struct Participant {
    pub id: usize,
    pub token: String,
}

#[derive(Debug, serde::Deserialize)]
struct AddParticipantBody {
    password: String,
}

pub async fn add(
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

    let body: AddParticipantBody = match serde_json::from_slice(&body_bytes) {
        Ok(body) => body,
        Err(e) => {
            warn!("Bad request: {e:?}");
            return bad_request_response();
        }
    };

    info!("{body:?}");

    const PASSWORD: &str = "abc";
    if body.password != PASSWORD {
        return unauthorized_response();
    }

    let (answer_sender, answer_receiver) = oneshot::channel();

    if let Err(e) = to_central_state_authority_sender
        .send(Message::ParticipantsGet { answer_sender })
        .await
    {
        error!("{e:?}");
        return internal_error_response();
    }

    let new_participant = match answer_receiver.await {
        Ok(new_participant) => new_participant,
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
                PARTICIPANT_ID_COOKIE_KEY, new_participant.id
            ),
        )
        .header(
            SET_COOKIE,
            format!(
                "{}={}; Path=/",
                ACCESS_TOKEN_COOKIE_KEY, new_participant.token
            ),
        )
        .status(StatusCode::CREATED)
        .body(Full::new(Bytes::new()))
}
