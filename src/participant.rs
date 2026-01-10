use std::collections::HashMap;

use http_body_util::{BodyExt, Full};
use hyper::{Request, Response, StatusCode, body::Bytes, header::SET_COOKIE};
use tokio::sync::{mpsc, oneshot};
use tracing::{error, info, warn};

use crate::{
    common::{
        ResponseResult, bad_request_response, extract_requesting_participant,
        internal_error_response, unauthorized_response,
    },
    election::{BallotItemId, ElectionId},
    state::Message,
};

pub const PARTICIPANT_ID_COOKIE_KEY: &str = "coco_participant_id";
pub const TOKEN_COOKIE_KEY: &str = "coco_token";

pub type ParticipantId = usize;

#[derive(Copy, Clone)]
pub struct ValidParticipantId(pub usize);

#[derive(Debug, Clone)]
pub struct Participant {
    pub credentials: ParticipantCredentials,
    pub voted_ballot_item_ids_by_election_id: HashMap<ElectionId, BallotItemId>,
}

#[derive(Debug, Clone)]
pub struct ParticipantCredentials {
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
        .send(Message::ParticipantsAdd { answer_sender })
        .await
    {
        error!("{e:?}");
        return internal_error_response();
    }

    let new_participant_credentials = match answer_receiver.await {
        Ok(new_participant_credentials) => new_participant_credentials,
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
                PARTICIPANT_ID_COOKIE_KEY, new_participant_credentials.id
            ),
        )
        .header(
            SET_COOKIE,
            format!(
                "{}={}; Path=/",
                TOKEN_COOKIE_KEY, new_participant_credentials.token
            ),
        )
        .status(StatusCode::CREATED)
        .body(Full::new(Bytes::new()))
}

pub async fn get_votes(
    request: Request<hyper::body::Incoming>,
    to_central_state_authority_sender: mpsc::Sender<Message>,
) -> ResponseResult {
    let Some(requesting_participant_credentials) = extract_requesting_participant(&request) else {
        return unauthorized_response();
    };

    let (answer_sender, answer_receiver) = oneshot::channel();

    if let Err(e) = to_central_state_authority_sender
        .send(Message::ParticipantsGetVotes {
            requesting_participant_credentials,
            answer_sender,
        })
        .await
    {
        error!("{e:?}");
        return internal_error_response();
    }

    let answer = match answer_receiver.await {
        Ok(answer) => answer,
        Err(err) => {
            error!("{err:?}");
            return internal_error_response();
        }
    };

    match answer {
        Ok(body) => Response::builder().body(Full::new(body)),
        Err(err) => {
            error!("{err:?}");
            err.to_response()
        }
    }
}
