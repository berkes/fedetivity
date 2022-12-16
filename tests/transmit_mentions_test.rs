mod common;

use actix::prelude::*;
use fedetivity::messages::*;
use fedetivity::transmitter::*;

use awc::Client;
use futures_util::stream::StreamExt;

#[derive(Message, Debug)]
#[rtype(result = "Vec<Activity>")]
pub struct GetReceived;

struct TestReceiver {
    received: Vec<Activity>,
}

impl Actor for TestReceiver {
    type Context = Context<Self>;
}

impl Handler<Activity> for TestReceiver {
    type Result = ();

    fn handle(&mut self, msg: Activity, _ctx: &mut Self::Context) -> Self::Result {
        self.received.push(msg);
    }
}

impl Handler<GetReceived> for TestReceiver {
    type Result = Vec<Activity>;

    fn handle(&mut self, _msg: GetReceived, _ctx: &mut Self::Context) -> Self::Result {
        self.received.clone()
    }
}

use awc::ws;
use futures_util::sink::SinkExt as _;

#[actix_rt::test]
async fn test_message_calls_is_handled() {
    common::setup();

    // Reset webmocket
    Client::default()
        .delete("http://localhost:3000/messages")
        .send()
        .await
        .expect("Reset webmocket server. Is it running?");

    let receiver = TestReceiver { received: vec![] };
    let receiver_addr = receiver.start();

    let (_, mut framed) = Client::default()
        .ws("http://localhost:3000/ws")
        .connect()
        .await
        .expect("Could not connect to websocket server. Is it running?");

    framed
        .send(ws::Message::Text("Hello".into()))
        .await
        .unwrap();
    let (sink, stream): (WsFramedSink, WsFramedStream) = framed.split();

    let _sut = FedClient::start(receiver_addr.clone().recipient(), sink, stream);
 
    // Add a message
    Client::default()
        .post("http://localhost:3000/messages")
        .send_body("New here!")
        .await
        .unwrap();

    // Wait untill the message is propagated to our test receiver
    let max_rounds = 5;
    let mut recorded = vec![];
    for _n in 0..max_rounds {
        recorded = receiver_addr.send(GetReceived).await.unwrap();

        if recorded.len() > 0 {
            break;
        }

        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    assert_eq!(vec![Activity], recorded);

    System::current().stop();
}
