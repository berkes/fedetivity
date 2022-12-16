use std::env;

use awc::Client;
use futures_util::stream::StreamExt;

use log::{debug, info};

extern crate fedetivity;

use fedetivity::transmitter::*;
use fedetivity::determinator::*;

use actix::{Context, Actor, Handler};
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


#[actix_rt::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let ws_uri = env::var("MASTODON_WS_URL").unwrap_or("ws://localhost:3000/ws".to_string());

    let determinator = Determinator::start(LogWorker.start().recipient());

    debug!("Connecting to {}", ws_uri);
    let client = Client::builder()
        .max_http_version(awc::http::Version::HTTP_11)
        .finish();
    let (_, framed) = client.ws(ws_uri).connect().await?;

    let (sink, stream): (WsFramedSink, WsFramedStream) = framed.split();
    let _addr = FedClient::start(determinator.clone().recipient(), sink, stream);

    actix_rt::signal::ctrl_c().await?;
    Ok(())
}
