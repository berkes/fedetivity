use actix::prelude::*;

use tungstenite::Message;

use log::{debug, error};

use crate::messages::*;

pub struct Socket;
pub struct HttpRequest;

pub struct FedClient {
    determinator: Recipient<Activity>,
    ws_uri: String,
}

impl FedClient {
    pub fn start(determinator: Recipient<Activity>, ws_uri: String) -> Addr<Self> {
        FedClient::create(|_ctx| FedClient {
            determinator,
            ws_uri,
        })
    }
}

impl Actor for FedClient {
    type Context = Context<Self>;

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        debug!("Stopped");
        actix::Running::Stop
    }
}

impl Handler<Connect> for FedClient {
    type Result = ();

    fn handle(&mut self, _msg: Connect, ctx: &mut Self::Context) {
        debug!("Connecting with {}", self.ws_uri);

        let (mut websocket, _response) =
            tungstenite::connect(&self.ws_uri).expect("connecting to websocket");

        ctx.run_interval(
            std::time::Duration::from_millis(200),
            move |fed_client, ctx| match websocket.read_message() {
                Ok(Message::Text(content)) => {
                    debug!("Text Message received: {}", content);
                    fed_client.determinator.do_send(Activity);
                }
                Ok(Message::Ping(_)) => {
                    debug!("Ping received");
                }
                Ok(_) => {
                    debug!("Other message received");
                }
                Err(err) => {
                    error!("Could not read message: {}", err);
                    ctx.address().do_send(Close);
                }
            },
        );
    }
}

impl Handler<Close> for FedClient {
    type Result = ();

    fn handle(&mut self, _msg: Close, ctx: &mut Self::Context) {
        debug!("Closing connection with {}", self.ws_uri);
        ctx.stop();
    }
}
