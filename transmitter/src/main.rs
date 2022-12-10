use awc::Client;
use futures_util::stream::StreamExt;

extern crate transmitter;
use transmitter::*;

#[actix_rt::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();
  
    let (_, framed) = Client::default()
        .ws("http://localhost:3000/ws")
        .connect()
        .await?;
    let (sink, stream): (WsFramedSink, WsFramedStream) = framed.split();
    let addr = FedClient::start(sink, stream);

    let _res = addr
        .send(Event::new(format!("subscribe"), "/client_count".to_string()))
        .await
        .unwrap();

    let _res = addr
        .send(Ping)
        .await
        .unwrap();
    let _ = actix_rt::signal::ctrl_c().await?;
    Ok(())
}
