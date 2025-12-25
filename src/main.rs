mod common;
mod election;
mod frontend;
mod participant;
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

use std::{error::Error, net::SocketAddr};

use state::Message;

use crate::common::ResponseResult;
use crate::frontend::FRONTEND_FILES;

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

    let (to_central_state_authority_sender, central_state_authority_receiver) = mpsc::channel(512);

    tokio::spawn(state::central_state_authority(
        central_state_authority_receiver,
    ));

    let socket_address: SocketAddr = ([127, 0, 0, 1], 3030).into();
    let listener = TcpListener::bind(socket_address).await?;

    info!("Listening on http://{}", socket_address);

    loop {
        let (tcp_stream, address) = listener.accept().await?;
        info!("Accepted connection from: {}", address);

        let connection_io_stream = TokioIo::new(tcp_stream);
        let to_central_state_authority_sender = to_central_state_authority_sender.clone();

        task::spawn(async move {
            let service = service_fn(|request| {
                handle_request(request, to_central_state_authority_sender.clone())
            });

            if let Err(err) = http1::Builder::new()
                .serve_connection(connection_io_stream, service)
                .await
            {
                error!("Error serving connection: {:?}", err);
            }
        });
    }
}

async fn handle_request(
    request: Request<hyper::body::Incoming>,
    to_central_state_authority_sender: mpsc::Sender<Message>,
) -> ResponseResult {
    info!("Got request {:?}", request);

    let result = match (request.method(), request.uri().path()) {
        (&Method::GET, path) if FRONTEND_FILES.contains_key(path) => {
            let file_data = FRONTEND_FILES[path];
            Response::builder()
                .header("Content-Type", file_data.kind.content_type())
                .body(Full::new(Bytes::from(file_data.content)))
        }
        (&Method::POST, "/participants/add") => {
            participant::add(request, to_central_state_authority_sender).await
        }
        (&Method::POST, "/elections/vote") => {
            election::vote(request, to_central_state_authority_sender).await
        }
        (&Method::GET, "/elections") => {
            election::get(request, to_central_state_authority_sender).await
        }
        _ => {
            warn!("Unable to handle request");
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Full::new(Bytes::from("CoCo does not know this page.")))
        }
    };

    if let Ok(response) = result.as_ref() {
        info!(
            "Sending response: {:?} {:?}",
            response.status(),
            response.headers()
        );
    }

    result
}
