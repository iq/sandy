use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Sender;
use tokio_tungstenite::connect_async;

#[derive(Serialize, Deserialize)]
pub struct PendingTransaction {
    pub transactions: Vec<Packet>,
}
#[derive(Serialize, Deserialize)]
pub struct Packet {
    pub data: Vec<u8>,
    pub meta: Option<Meta>,
}
#[derive(Serialize, Deserialize)]
pub struct Meta {
    pub size: u64,
}

// for this example i just setup a simple javascript server to send transactions through ws
// obviously this would have to be refactored if used in production
pub async fn forward_incoming_transactions(pending_transactions_tx: Sender<PendingTransaction>) {
    let (ws_stream, _) = connect_async("ws://localhost:8080").await.unwrap();
    let (_, mut read) = ws_stream.split();

    while let Some(Ok(msg)) = read.next().await {
        let msg = msg.into_text().unwrap();
        let pending_transaction: PendingTransaction = serde_json::from_str(&msg).unwrap();

        pending_transactions_tx.send(pending_transaction).await.unwrap();
    }
}
