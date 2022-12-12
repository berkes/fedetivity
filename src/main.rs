use std::env;

use awc::Client;
use futures_util::stream::StreamExt;

extern crate fedetivity;

use fedetivity::transmitter::*;

use actix::{Actor, Context, Handler};
use fedetivity::messages::Activity;

pub struct Determinator;

impl Actor for Determinator {
    type Context = Context<Self>;
}

impl Handler<Activity> for Determinator {
    type Result = ();

    fn handle(&mut self, _msg: Activity, _ctx: &mut Self::Context) {
        // noop
    }
}

#[actix_rt::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let ws_uri = env::var("MASTODON_WS_URL").unwrap_or("ws://localhost:3000/ws".to_string());

    let determinator = Determinator::start(Determinator {});

    let (_, framed) = Client::default()
        .ws(ws_uri)
        .connect()
        .await?;
    let (sink, stream): (WsFramedSink, WsFramedStream) = framed.split();
    let _addr = FedClient::start(determinator.clone().recipient(), sink, stream);

    actix_rt::signal::ctrl_c().await?;
    Ok(())
}
