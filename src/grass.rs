use anyhow::{Result, Context};
use tokio::time::{interval, Duration};
use tokio::task::JoinHandle;
use tokio::sync::mpsc;

use reqwest::{Client, ClientBuilder};
use reqwest_websocket::{Message, RequestBuilderExt, WebSocket};
use futures_util::stream::SplitSink;
use futures_util::{StreamExt, SinkExt};

use serde_json::Value;
use uuid::Uuid;

const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36";

pub struct Grass {
    client: Client,
    ping_task: Option<JoinHandle<()>>,
    tx: Option<mpsc::Sender<Message>>,

    log_target: String,
    pub user_id: String,
    pub device_id: String,
}

impl Grass {
    pub fn new(log_target: String, user_id: String, proxy: Option<&str>) -> Result<Self> {
        let mut builder = ClientBuilder::new();

        if let Some(proxy_str) = proxy {
            builder = builder.proxy(reqwest::Proxy::all(proxy_str).unwrap());
        }

        let client = builder
            .danger_accept_invalid_certs(true)
            .danger_accept_invalid_hostnames(true)
            .build()
            .context("Failed to build reqwest client")?;

        Ok(Self {
            client,
            ping_task: None,
            tx: None,
            log_target,
            user_id,
            device_id: Uuid::new_v4().to_string(),
        })
    }

    fn start_ping_task(&mut self, tx: mpsc::Sender<Message>) {
        let ping_task = tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(110));

            loop {
                interval.tick().await;
                let ping_msg = Message::Text(
                    serde_json::json!({
                        "id": Uuid::new_v4().to_string(),
                        "version": "1.0.0",
                        "action": "PING",
                        "data": serde_json::json!({}),
                    }).to_string()
                );

                if let Err(e) = tx.send(ping_msg).await {
                    log::error!("Failed to send ping: {:?}", e);
                    break;
                }
            }
        });

        self.ping_task = Some(ping_task);
    }

    async fn writer_task(mut writer: SplitSink<WebSocket, Message>, mut rx: mpsc::Receiver<Message>) {
        while let Some(message) = rx.recv().await {
            if let Err(e) = writer.send(message).await {
                log::error!("Failed to send message: {:?}", e);
                break;
            }
        }
    }

    pub async fn connect(&mut self) -> Result<()> {
        let websocket = self.client
            .get("wss://proxy2.wynd.network:4444/")
            .header("User-Agent", USER_AGENT)
            .upgrade()
            .send()
            .await?
            .into_websocket()
            .await?;

        let (writer, mut reader) = websocket.split();

        let (tx, rx) = mpsc::channel(32);
        self.tx = Some(tx.clone());

        tokio::spawn(Self::writer_task(writer, rx));
        self.start_ping_task(tx);

        loop {
            match reader.next().await {
                Some(Ok(message)) => {
                    if let Message::Text(text) = message {
                        if let Err(e) = self.handle_message(text).await {
                            log::error!(target: &self.log_target, "Error handling message: {:?}", e);
                        }
                    }
                },
                Some(Err(e)) => {
                    log::error!(target: &self.log_target, "WebSocket error: {:?}", e);
                    break;
                },
                None => {
                    log::info!(target: &self.log_target, "WebSocket closed");
                    break;
                }
            }
        }

        if let Some(task) = self.ping_task.take() {
            task.abort();
        }

        Ok(())
    }

    async fn handle_message(&mut self, message: String) -> Result<()> {
        let json: Value = serde_json::from_str(&message)
            .context("Failed to parse WebSocket message as JSON")?;
        let action = json["action"].as_str().context("Missing action field")?;
        let message_id = json["id"].as_str().context("Missing id field")?;

        log::info!(target: &self.log_target, "Received message: {}", message);

        match action {
            "PONG" => {
                let ping_message = Message::Text(
                    serde_json::json!({
                        "id": message_id,
                        "origin_action": "PONG",
                    }).to_string()
                );

                if let Some(tx) = &self.tx {
                    tx.send(ping_message).await?;
                }
            }
            "AUTH" => {
                let auth_message = Message::Text(
                    serde_json::json!({
                        "id": message_id,
                        "origin_action": "AUTH",
                        "result": serde_json::json!({
                            "browser_id": self.device_id,
                            "user_id": self.user_id,
                            "user_agent": USER_AGENT,
                            "timestamp": chrono::Utc::now().timestamp(),
                            "device_type": "desktop",
                            "version": "4.26.2",
                        }),
                    }).to_string()
                );

                if let Some(tx) = &self.tx {
                    tx.send(auth_message).await?;
                    log::info!(target: &self.log_target, "Authenticated as user_id: {}", self.user_id);
                }
            }
            _ => {
                log::debug!(target: &self.log_target, "Unhandled action: {}", action);
            }
        }

        Ok(())
    }
}

impl Drop for Grass {
    fn drop(&mut self) {
        if let Some(task) = self.ping_task.take() {
            task.abort();
        }
    }
}