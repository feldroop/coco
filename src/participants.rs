use http_body_util::{BodyExt, Full};
use hyper::{Request, Response, StatusCode, body::Bytes, header::SET_COOKIE};
use tokio::sync::{mpsc, oneshot};
use tracing::info;

use crate::state::Message;

const PASSWORD: &str = "abc";

#[derive(serde::Deserialize)]
struct AddParticipantBody {
    password: String,
}

pub async fn add(
    request: Request<hyper::body::Incoming>,
    message_sender: mpsc::Sender<Message>,
) -> Result<Response<Full<Bytes>>, hyper::http::Error> {
    let (answer_sender, answer_receiver) = oneshot::channel();

    let body_bytes = request
        .into_body()
        .collect()
        .await
        .expect("add user request body")
        .to_bytes();

    info!("Add participants request body: {body_bytes:?}");

    let body: AddParticipantBody = match serde_json::from_slice(&body_bytes) {
        Err(e) => {
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Full::new(Bytes::from(e.to_string())));
        }
        Ok(body) => body,
    };

    if body.password != PASSWORD {
        return Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(Full::new(Bytes::from("Wrong password.")));
    }

    message_sender
        .send(Message::AddParticipant { answer_sender })
        .await
        .expect("send message");

    let new_participant_id = answer_receiver.await.expect("receive answer");

    Response::builder()
        .header(SET_COOKIE, "logged_in=true; Path=/")
        .status(StatusCode::OK)
        .body(Full::new(Bytes::from(format!(
            "Hello, participant {new_participant_id}!"
        ))))
}
