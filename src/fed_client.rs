use actix::prelude::*;
use log::{debug, error};
use tungstenite::Message;

use crate::messages::*;

pub struct Socket;
pub struct HttpRequest;

pub struct FedClient {
    activity_mapper: Recipient<Activity>,
    ws_uri: String,
}

/// FedClient connects to a websocket and passes text messages along.
///
/// FedClient connects on `Connect`, and disconnects on `Close`.
/// FedClient reads incoming messages from websocket and passes any Text message
/// to its activity_mapper.
impl FedClient {
    pub fn start(activity_mapper: Recipient<Activity>, ws_uri: String) -> Addr<Self> {
        FedClient::create(|_ctx| FedClient {
            activity_mapper,
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
                    fed_client.activity_mapper.do_send(Activity);
                }
                Ok(Message::Ping(_)) => {
                    debug!("Ping received");
                }
                Ok(Message::Close(_)) => {
                    debug!("Close Received. Stopping");
                    ctx.stop();
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

#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};

    use super::*;

    #[derive(Message, Debug, PartialEq, Clone)]
    #[rtype(result = "Vec<Activity>")]
    pub struct Assert;

    #[derive(Clone, Default)]
    struct TestProbe {
        received: Vec<Activity>,
    }
    impl Actor for TestProbe {
        type Context = Context<Self>;
    }
    impl Handler<Activity> for TestProbe {
        type Result = ();

        fn handle(&mut self, msg: Activity, _ctx: &mut Self::Context) -> Self::Result {
            self.received.push(msg);
        }
    }

    impl Handler<Assert> for TestProbe {
        type Result = Vec<Activity>;

        fn handle(&mut self, _msg: Assert, _ctx: &mut Self::Context) -> Self::Result {
            self.received.clone()
        }
    }

    #[actix_rt::test]
    #[should_panic]
    async fn test_that_connect_panics_connection_err() {
        let probe = TestProbe { received: vec![] }.start();
        let sut = FedClient::start(probe.recipient(), "ws://localhost:8001".to_string());

        sut.send(Connect).await.unwrap();
    }

    #[actix_rt::test]
    async fn test_that_it_connects_to_server() {
        env_logger::init();

        let probe = TestProbe { received: vec![] }.start();
        let sut_connect = FedClient::start(probe.recipient(), "ws://localhost:3000/ws".to_string());
        let sut_assert = sut_connect.clone();

        let thread = thread::spawn(|| async move {
            sut_connect.send(Connect).await.unwrap();
        });
        assert!(sut_assert.connected());

        thread::sleep(Duration::from_millis(200));
        let res = reqwest::Client::default()
            .delete("http://localhost:3000/connections")
            .send()
            .await
            .expect("Closing connection");
        assert_eq!(200, res.status());

        thread.join().unwrap().await;
    }
}
