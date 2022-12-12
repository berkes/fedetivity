use actix::{io::SinkWrite, prelude::*};
use actix_codec::Framed;
use awc::{error::WsProtocolError, ws, ws::Frame, BoxedSocket};
use futures::stream::{SplitSink, SplitStream};

use log::{debug, error};

use crate::messages::*;

pub type WsFramedSink = SplitSink<Framed<BoxedSocket, ws::Codec>, ws::Message>;
pub type WsFramedStream = SplitStream<Framed<BoxedSocket, ws::Codec>>;

pub struct Socket;
pub struct HttpRequest;

pub struct FedClient {
    sink: SinkWrite<ws::Message, WsFramedSink>,
    determinator: Recipient<Activity>,
}

impl FedClient {
    pub fn start(
        determinator: Recipient<Activity>,
        sink: WsFramedSink,
        stream: WsFramedStream,
    ) -> Addr<Self> {
        FedClient::create(|ctx| {
            debug!("Start FedClient");
            ctx.add_stream(stream);
            FedClient {
                sink: SinkWrite::new(sink, ctx),
                determinator,
            }
        })
    }
}

impl Actor for FedClient {
    type Context = Context<Self>;

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        if self.sink.write(ws::Message::Close(None)).is_err() {
            error!("Could not close the connection");
        }

        debug!("Stopped");
        actix::Running::Stop
    }
}

impl Handler<Ping> for FedClient {
    type Result = ();

    fn handle(&mut self, _msg: Ping, _ctx: &mut Self::Context) {
        debug!("Pinging server");
        if let Err(error) = self.sink.write(ws::Message::Ping("".into())) {
            error!("Error FedClient {:?}", error);
        }
    }
}

impl StreamHandler<Result<ws::Frame, WsProtocolError>> for FedClient {
    fn handle(&mut self, item: Result<ws::Frame, WsProtocolError>, _ctx: &mut Self::Context) {
        match item.unwrap() {
            Frame::Text(text_bytes) => {
                let text = std::str::from_utf8(text_bytes.as_ref()).unwrap();
                debug!("Received Text: {}", text);
                self.determinator
                    .try_send(Activity)
                    .expect("Send to determinator");
                debug!("Sent to determinator");
            }
            Frame::Binary(_) => {
                debug!("Received Binary");
            }
            Frame::Continuation(_) => {
                debug!("Received Continuation");
            }
            Frame::Ping(content) => {
                debug!("Received Ping");
                // TODO: set hb alive
                if let Err(error) = self.sink.write(ws::Message::Pong(content)) {
                    error!("Error returning a ping with a pong: {:?}", error);
                }
            }
            Frame::Pong(_) => {
                debug!("Received Pong");
                // self.hb = Instant::now();
            }
            Frame::Close(_) => {
                debug!("Received Close");
            }
        }
    }
}

impl actix::io::WriteHandler<WsProtocolError> for FedClient {}
