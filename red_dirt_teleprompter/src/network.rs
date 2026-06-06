use tracing::info;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::accept_async;

use crate::{TeleprompterConfig, TeleprompterError, TeleprompterUpdate};

/// Network interface for the teleprompter — broadcasts state via WebSocket.
pub struct NetworkInterface {
    config: TeleprompterConfig,
}

impl NetworkInterface {
    pub fn new(config: &TeleprompterConfig) -> Result<Self, TeleprompterError> {
        Ok(Self { config: config.clone() })
    }
}

/// WebSocket server for teleprompter state broadcasting.
pub struct TeleprompterServer {
    update_tx: tokio::sync::broadcast::Sender<TeleprompterUpdate>,
}

impl TeleprompterServer {
    pub fn new() -> Self {
        let (tx, _) = tokio::sync::broadcast::channel::<TeleprompterUpdate>(256);
        Self { update_tx: tx }
    }

    pub fn sender(&self) -> tokio::sync::broadcast::Sender<TeleprompterUpdate> {
        self.update_tx.clone()
    }

    pub async fn run(
        &self,
        addr: &str,
        mut shutdown: tokio::sync::watch::Receiver<bool>,
    ) -> Result<(), TeleprompterError> {
        use tokio::net::TcpListener;
        let listener = TcpListener::bind(addr).await
            .map_err(|e| TeleprompterError::NetworkError(e.to_string()))?;
        info!("Teleprompter WebSocket server listening on {}", addr);

        let tx = self.update_tx.clone();
        loop {
            tokio::select! {
                biased;
                // Graceful shutdown
                _ = shutdown.changed() => {
                    if *shutdown.borrow() {
                        info!("Network server received shutdown signal");
                        break;
                    }
                }
                // Accept new connections
                result = listener.accept() => {
                    let (stream, peer) = result
                        .map_err(|e| TeleprompterError::NetworkError(e.to_string()))?;
                    let tx = tx.clone();
                    tokio::spawn(async move {
                        if let Ok(ws) = accept_async(stream).await {
                            info!("Teleprompter client connected: {:?}", peer);
                            let mut rx = tx.subscribe();
                            let (mut write, mut _read) = ws.split();
                            loop {
                                match rx.recv().await {
                                    Ok(update) => {
                                        let json = serde_json::to_string(&update).unwrap_or_default();
                                        if write.send(tokio_tungstenite::tungstenite::Message::Text(json.into())).await.is_err() {
                                            break;
                                        }
                                    }
                                    Err(_) => break,
                                }
                            }
                        }
                    });
                }
            }
        }
        Ok(())
    }
}
