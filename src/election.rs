use std::collections::{HashMap, HashSet};

use http_body_util::{BodyExt, Full};
use hyper::{Request, Response, body::Bytes};
use tokio::sync::{mpsc, oneshot};
use tracing::{error, info, warn};

use crate::{
    common::{
        ResponseResult, bad_request_response, extract_requesting_credentials,
        extract_requesting_participant, internal_error_response, ok_response,
        unauthorized_response,
    },
    participant::ParticipantId,
    state::Message,
};

pub type BallotItemId = usize;
pub type ElectionId = usize;

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Election {
    pub id: ElectionId,
    pub name: String,
    pub ballot_items_by_id: HashMap<BallotItemId, BallotItem>,
    #[serde(skip)]
    pub participant_ids_who_voted: HashSet<ParticipantId>,
}

#[derive(Debug, serde::Serialize)]
pub struct BallotItem {
    pub id: ElectionId,
    pub name: String,
    #[serde(skip)]
    pub num_votes: usize,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ElectionsVoteBody {
    pub election_id: ElectionId,
    pub selected_ballot_item_id: BallotItemId,
}

pub async fn get(
    request: Request<hyper::body::Incoming>,
    to_central_state_authority_sender: mpsc::Sender<Message>,
) -> ResponseResult {
    let Some(requesting_credentials) = extract_requesting_credentials(&request) else {
        return unauthorized_response();
    };

    let (answer_sender, answer_receiver) = oneshot::channel();
    if let Err(e) = to_central_state_authority_sender
        .send(Message::ElectionsGet {
            answer_sender,
            requesting_credentials,
        })
        .await
    {
        error!("{e:?}");
        return internal_error_response();
    }

    let answer = match answer_receiver.await {
        Ok(body) => body,
        Err(e) => {
            error!("{e:?}");
            return internal_error_response();
        }
    };

    match answer {
        Ok(body) => Response::builder().body(Full::new(body)),
        Err(err) => Response::builder()
            .status(err.http_status_code())
            .body(Full::new(Bytes::from_owner(err.to_string()))),
    }
}

pub async fn vote(
    request: Request<hyper::body::Incoming>,
    to_central_state_authority_sender: mpsc::Sender<Message>,
) -> ResponseResult {
    let Some(requesting_participant) = extract_requesting_participant(&request) else {
        return unauthorized_response();
    };

    let body_bytes = match request.into_body().collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(e) => {
            error!("{e:?}");
            return internal_error_response();
        }
    };

    let body: ElectionsVoteBody = match serde_json::from_slice(&body_bytes) {
        Ok(body) => body,
        Err(e) => {
            warn!("Bad request: {e:?}");
            return bad_request_response();
        }
    };

    info!("{body:?}");

    let (answer_sender, answer_receiver) = oneshot::channel();

    if let Err(e) = to_central_state_authority_sender
        .send(Message::ElectionsVote {
            answer_sender,
            requesting_participant,
            elections_vote_body: body,
        })
        .await
    {
        error!("{e:?}");
        return internal_error_response();
    }

    let answer = match answer_receiver.await {
        Ok(body) => body,
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
