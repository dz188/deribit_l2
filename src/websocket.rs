use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc::Sender;
use tokio::time::{Duration, sleep};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use url::Url;

pub async fn run(instruments: Vec<String>, raw_tx: Sender<String>) -> Result<()> {
    let url = Url::parse("wss://www.deribit.com/ws/api/v2")?;

    loop {
        println!("Connecting to Deribit...");

        let connect_result = connect_async(url.as_str()).await;

        let (ws_stream, _) = match connect_result {
            Ok(v) => v,
            Err(e) => {
                println!("Connect error: {:?}", e);
                sleep(Duration::from_secs(3)).await;
                continue;
            }
        };

        println!("WebSocket connected.");

        let (mut write, mut read) = ws_stream.split();

        let channels: Vec<String> = instruments
            .iter()
            .map(|inst| format!("book.{}.100ms", inst))
            .collect();

        let subscribe_msg = serde_json::json!({
            "jsonrpc":"2.0",
            "method":"public/subscribe",
            "params":{
                "channels":channels
            },
            "id":1
        });

        if let Err(e) = write
            .send(Message::Text(subscribe_msg.to_string().into()))
            .await
        {
            println!("Subscribe failed: {:?}", e);
            sleep(Duration::from_secs(3)).await;
            continue;
        }

        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    let text_string = text.to_string();
                    if raw_tx.send(text_string).await.is_err() {
                        println!("State engine dropped. Exiting.");
                        return Ok(());
                    }
                }

                //add ping
                Ok(Message::Ping(payload)) => {
                    println!("Received ping.");
                    if let Err(e) = write.send(Message::Pong(payload)).await {
                        println!("Failed to send pong: {:?}", e);
                        break;
                    }
                }

                // pong (ignored)
                Ok(Message::Pong(_)) => {}

                // server Close
                Ok(Message::Close(frame)) => {
                    println!("Server closed connection: {:?}", frame);
                    break;
                }

                Ok(_) => {}
                Err(e) => {
                    println!("Read error: {:?}", e);
                    break;
                }
            }
        }

        println!("Disconnected. Reconnecting in 3 seconds...");
        sleep(Duration::from_secs(3)).await;
    }
}
