use actix::{io::SinkWrite, prelude::*};
use actix_codec::Framed;
use awc::{error::WsProtocolError, ws, BoxedSocket};
use futures::stream::{SplitSink, SplitStream};

use log::{debug, error, info};

pub type WsFramedSink = SplitSink<Framed<BoxedSocket, ws::Codec>, ws::Message>;
pub type WsFramedStream = SplitStream<Framed<BoxedSocket, ws::Codec>>;

#[derive(Debug)]
pub struct Error;

pub struct Socket;
pub struct HttpRequest;

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct Ping;

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct Event {
    op: String,
    topic: String,
}
impl Event {
    pub fn new(op: String, topic: String) -> Self {
        Self { op, topic }
    }
}

pub struct FedClient {
    sink: SinkWrite<ws::Message, WsFramedSink>,
}

impl FedClient {
    pub fn start(sink: WsFramedSink, stream: WsFramedStream) -> Addr<Self> {
        FedClient::create(|ctx| {
            ctx.add_stream(stream);
            FedClient {
                sink: SinkWrite::new(sink, ctx),
            }
        })
    }
}

impl Actor for FedClient {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        dbg!(ctx);
        println!("Actor is alive");
    }

    fn stopped(&mut self, _ctx: &mut Context<Self>) {
        // TODO: disconnect from the stream elegantly
        println!("Actor is stopped");
    }
}

impl Handler<Event> for FedClient {
    type Result = ();

    fn handle(&mut self, msg: Event, _ctx: &mut Self::Context) {
        info!("Pushing Message {:?}", msg);
        if let Err(error) = self.sink.write(ws::Message::Text(
            format!("op: {}, topic: {}", msg.op, msg.topic).into(),
        )) {
            error!("Error FedClient {:?}", error);
        }
    }
}
impl Handler<Ping> for FedClient {
    type Result = ();

    fn handle(&mut self, _msg: Ping, _ctx: &mut Self::Context) {
        info!("Pinging server");
        if let Err(error) = self.sink.write(ws::Message::Ping("".into())) {
            error!("Error FedClient {:?}", error);
        }
    }
}

impl StreamHandler<Result<ws::Frame, WsProtocolError>> for FedClient {
    fn handle(&mut self, item: Result<ws::Frame, WsProtocolError>, _ctx: &mut Self::Context) {
        use ws::Frame;
        match item.unwrap() {
            Frame::Text(text_bytes) => {
                debug!("received Text");
                let text = std::str::from_utf8(text_bytes.as_ref()).unwrap();
                info!("Receiving Message: {}", text);
            }
            Frame::Binary(_) => {
                debug!("received Binary");
            }
            Frame::Continuation(_) => {
                debug!("received Continuation");
            }
            Frame::Ping(content) => {
                debug!("received Ping");
                // TODO: set hb alive
                if let Err(error) = self.sink.write(ws::Message::Pong(content)) {
                    error!("Error returning a ping with a pong: {:?}", error);
                }
            }
            Frame::Pong(_) => {
                debug!("received Pong");
                // self.hb = Instant::now();
            }
            Frame::Close(_) => {
                debug!("received Close");
            }
        }
    }
}

impl actix::io::WriteHandler<WsProtocolError> for FedClient {}
