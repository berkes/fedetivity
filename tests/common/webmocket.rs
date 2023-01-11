use reqwest::Client;

const WM_MESSAGES_URI: &str = "http://localhost:3000/messages";

pub async fn reset() {
    Client::default()
        .delete(WM_MESSAGES_URI)
        .send()
        .await
        .expect("Reset webmocket server. Is it running?");
}

pub async fn server_to_client_message(message: String) {
    Client::default()
        .post(WM_MESSAGES_URI)
        .body(message)
        .send()
        .await
        .expect("Sending message from server to client");
}
