use std::env;
use log::{debug, info};

use awc;
use actix::{Context, Actor, Handler};
use futures_util::stream::StreamExt;

extern crate fedetivity;

use fedetivity::transmitter::*;
use fedetivity::determinator::*;
use fedetivity::messages::Job;

struct LogWorker;

impl Actor for LogWorker {
    type Context = Context<Self>;
}

impl Handler<Job> for LogWorker {
    type Result = ();

    fn handle(&mut self, msg: Job, ctx: &mut Self::Context) -> Self::Result {
        debug!("Context when handling Job: {:?}", ctx);
        info!("Received message: {:?}", msg)
    }
}

struct WsClient {
    uri: String,
    client: awc::Client,
}

impl WsClient {
    fn new(uri: String) -> Self {
        let client = awc::Client::builder()
            .max_http_version(awc::http::Version::HTTP_11)
            .finish();
        Self { uri, client }
    }

    async fn ws_connect(&self) -> Result<(WsFramedSink, WsFramedStream), Box<dyn std::error::Error>> {
        let (_, framed) = self.client.ws(&self.uri).connect().await?;
        let (sink, stream): (WsFramedSink, WsFramedStream) = framed.split();
        Ok((sink, stream))
    }
}

#[actix_rt::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let ws_uri = env::var("MASTODON_WS_URL").unwrap_or("ws://localhost:3000/ws".to_string());

    debug!("Starting Determinator");
    let determinator = Determinator::start(LogWorker.start().recipient());

    debug!("Connecting to {}", ws_uri);
    let (sink, stream) = WsClient::new(ws_uri).ws_connect().await?;

    debug!("Starting FedClient");
    let _addr = FedClient::start(determinator.clone().recipient(), sink, stream);

    actix_rt::signal::ctrl_c().await?;
    Ok(())
}
