mod common;

use actix::prelude::*;
use fedetivity::fed_client::*;
use fedetivity::messages::*;

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

#[actix_rt::test]
async fn test_message_calls_is_handled() {
    common::setup();
    common::webmocket::reset().await;

    let receiver = TestReceiver { received: vec![] };
    let receiver_addr = receiver.start();

    let sut = FedClient::start(
        receiver_addr.clone().recipient(),
        "ws://localhost:3000/ws".to_string(),
    );

    sut.send(Connect).await.unwrap();

    common::webmocket::server_to_client_message("New here!".to_string()).await;

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
