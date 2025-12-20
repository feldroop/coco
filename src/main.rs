mod frontend;
mod participants;
mod state;

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio::task;
use tracing::{error, info, warn};
use tracing_subscriber::fmt::time;

use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::{error::Error, net::SocketAddr};

use state::{Message, NEXT_PARTICIPANT_ID, Participant, State};

use crate::frontend::FileKind;

const INCLUDE_SOURCEMAPS: bool = true;

fn main() -> Result<(), Box<dyn Error>> {
    let subscriber_builder = tracing_subscriber::fmt();

    let time_offset_initialization_result = match time::OffsetTime::local_rfc_3339() {
        Ok(timer) => subscriber_builder.with_timer(timer).try_init(),
        Err(e) => {
            eprintln!(
                "WARNING: Unable to get local time zone information for logs. Error message: {e}"
            );
            subscriber_builder.try_init()
        }
    };

    if let Err(e) = time_offset_initialization_result {
        eprintln!("Unable to set global default subscriber: {e}");
    }

    run_server()
}

#[tokio::main]
async fn run_server() -> Result<(), Box<dyn Error>> {
    tracing::info!("Server runtime started");

    let (message_sender, message_receiver) = mpsc::channel(512);

    tokio::spawn(central_state_authority(message_receiver));

    let socket_address: SocketAddr = ([127, 0, 0, 1], 3030).into();
    let listener = TcpListener::bind(socket_address).await?;

    info!("Listening on http://{}", socket_address);

    loop {
        let (tcp, address) = listener.accept().await?;
        info!("Accepted connection from: {}", address);

        let io = TokioIo::new(tcp);
        let message_sender = message_sender.clone();

        task::spawn(async move {
            let service = service_fn(|request| {
                let message_sender = message_sender.clone();
                handle_request(request, message_sender)
            });

            if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                error!("Error serving connection: {:?}", err);
            }
        });
    }
}

async fn central_state_authority(mut message_receiver: mpsc::Receiver<Message>) {
    let mut state = State::default();

    while let Some(message) = message_receiver.recv().await {
        match message {
            Message::AddParticipant { answer_sender } => {
                let id = NEXT_PARTICIPANT_ID.fetch_add(1, Ordering::SeqCst);

                let new_participant = state
                    .participants
                    .entry(id)
                    .insert_entry(Participant { id });

                answer_sender
                    .send(new_participant.get().id)
                    .expect("send answer");
            }
        }
    }
}

async fn handle_request(
    request: Request<hyper::body::Incoming>,
    message_sender: mpsc::Sender<Message>,
) -> Result<Response<Full<Bytes>>, hyper::http::Error> {
    info!("Got request {:?}", request);

    let frontend_files: HashMap<_, _> = frontend::FILES
        .iter()
        .filter_map(|file_data| {
            if !INCLUDE_SOURCEMAPS && file_data.kind == FileKind::JsMap {
                None
            } else {
                Some((file_data.path, file_data))
            }
        })
        .collect();

    let result = match (request.method(), request.uri().path()) {
        (&Method::GET, path) if frontend_files.contains_key(path) => {
            let file_data = frontend_files[path];
            Response::builder()
                .header("Content-Type", file_data.kind.content_type())
                .body(Full::new(Bytes::from(file_data.content)))
        }
        (&Method::POST, "/participants/add") => participants::add(request, message_sender).await,
        _ => {
            warn!("Unable to handle request");
            let mut not_found =
                Response::new(Full::new(Bytes::from("CoCo does not know this page.")));
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    };

    if let Ok(response) = result.as_ref() {
        info!("Sending response: {response:?}");
    }

    result
}
