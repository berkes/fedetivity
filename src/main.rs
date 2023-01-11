use std::env;
use log::{debug, info};

use actix::{Context, Actor, Handler};

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

#[actix_rt::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let ws_uri = env::var("MASTODON_WS_URL").unwrap_or("ws://localhost:3000/ws".to_string());

    debug!("Starting Determinator");
    let determinator = Determinator::start(LogWorker.start().recipient());

    debug!("Starting FedClient");
    let fed_client = FedClient::start(determinator.clone().recipient(), ws_uri);
    fed_client.send(fedetivity::messages::Connect).await?;

    actix_rt::signal::ctrl_c().await?; 
    Ok(())
}
