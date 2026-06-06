use tokio_tungstenite::{accept_async, tungstenite::Message};
use tokio::net::TcpListener;
use std::net::TcpListener as StdTcpListener;
use std::sync::Arc;
use parking_lot::RwLock;
use futures_util::{SinkExt, StreamExt};
use futures_channel::mpsc::UnboundedSender;
use serde::{Deserialize, Serialize};

use crate::{TeleprompterConfig, TeleprompterError, TeleprompterUpdate};

/// Casting server for multi-device streaming
pub struct CastingServer {
    /// Blocking TCP listener (converted to async inside dedicated thread)
    ws_listener: Option<StdTcpListener>,

    /// Connected clients
    clients: Arc<RwLock<Vec<ClientConnection>>>,

    /// Server running state
    running: Arc<RwLock<bool>>,

    /// Configuration
    config: CastingConfig,
}

#[derive(Debug, Clone)]
pub struct CastingConfig {
    pub port: u16,
    pub max_connections: usize,
    pub streaming_quality: StreamingQuality,
    pub multicast_discovery: bool,
}

#[derive(Debug, Clone)]
pub enum StreamingQuality {
    Low,      // 480p, 15fps
    Medium,   // 720p, 30fps
    High,     // 1080p, 60fps
    Ultra,    // 4K, 60fps
}

impl Default for CastingConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            max_connections: 10,
            streaming_quality: StreamingQuality::High,
            multicast_discovery: true,
        }
    }
}

#[derive(Debug)]
struct ClientConnection {
    /// Client ID
    id: String,

    /// WebSocket connection
    ws_tx: UnboundedSender<Message>,

    /// Client type
    client_type: ClientType,

    /// Connection time
    connected_at: std::time::SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientType {
    WebBrowser,
    MobileApp,
    SmartTV,
    DesktopApp,
    Projector,
}

impl CastingServer {
    pub fn new(_config: &TeleprompterConfig) -> Result<Self, TeleprompterError> {
        let port = 8080;
        let std_listener = StdTcpListener::bind(format!("0.0.0.0:{}", port))
            .map_err(|e| TeleprompterError::NetworkError(e.to_string()))?;
        
        // Set non-blocking so tokio can adopt it cleanly later
        std_listener.set_nonblocking(true)
            .map_err(|e| TeleprompterError::NetworkError(e.to_string()))?;

        Ok(Self {
            ws_listener: Some(std_listener),
            clients: Arc::new(RwLock::new(Vec::new())),
            running: Arc::new(RwLock::new(false)),
            config: CastingConfig {
                port,
                max_connections: 10,
                streaming_quality: StreamingQuality::High,
                multicast_discovery: true,
            },
        })
    }

    /// Start the casting server in a dedicated thread with its own Tokio runtime.
    pub fn start(&mut self) -> Result<(), TeleprompterError> {
        let listener = self.ws_listener.take().ok_or_else(|| 
            TeleprompterError::InitializationFailed("Server already started".to_string())
        )?;

        let clients = Arc::clone(&self.clients);
        let running = Arc::clone(&self.running);

        *self.running.write() = true;

        // Spawn a dedicated thread that owns a Tokio runtime.
        std::thread::spawn(move || {
            let rt = match tokio::runtime::Runtime::new() {
                Ok(rt) => rt,
                Err(e) => {
                    eprintln!("Failed to create Tokio runtime for casting server: {}", e);
                    return;
                }
            };

            rt.block_on(async move {
                let listener = match TcpListener::from_std(listener) {
                    Ok(l) => l,
                    Err(e) => {
                        eprintln!("Failed to convert std listener to tokio: {}", e);
                        return;
                    }
                };

                println!("Casting server listening on 0.0.0.0:8080");

                while *running.read() {
                    match listener.accept().await {
                        Ok((stream, addr)) => {
                            println!("New client connection from: {}", addr);

                            let clients_clone = Arc::clone(&clients);
                            let running_clone = Arc::clone(&running);
                            tokio::spawn(handle_client_connection(stream, clients_clone, running_clone));
                        }
                        Err(e) => {
                            eprintln!("Error accepting connection: {}", e);
                        }
                    }
                }

                println!("Casting server shutting down");
            });
        });

        Ok(())
    }

    /// Broadcast update to all connected clients
    pub fn broadcast_update(&self, update: TeleprompterUpdate) -> Result<(), TeleprompterError> {
        let clients = self.clients.read();

        for client in clients.iter() {
            let message = serde_json::to_string(&update)
                .map_err(|e| TeleprompterError::NetworkError(e.to_string()))?;

            if let Err(e) = client.ws_tx.unbounded_send(Message::Text(message.into())) {
                eprintln!("Failed to send to client: {}", e);
            }
        }

        Ok(())
    }

    /// Stop the casting server
    pub fn stop(&self) -> Result<(), TeleprompterError> {
        *self.running.write() = false;

        // Disconnect all clients
        let mut clients = self.clients.write();
        clients.clear();

        Ok(())
    }

    /// Get connected client count
    pub fn get_client_count(&self) -> usize {
        self.clients.read().len()
    }

    /// Get client information
    pub fn get_clients(&self) -> Vec<ClientInfo> {
        self.clients.read().iter().map(|client| ClientInfo {
            id: client.id.clone(),
            client_type: client.client_type.clone(),
            connected_at: client.connected_at,
        }).collect()
    }
}

async fn handle_client_connection(
    stream: tokio::net::TcpStream,
    clients: Arc<RwLock<Vec<ClientConnection>>>,
    _running: Arc<RwLock<bool>>,
) {
    if let Ok(ws_stream) = accept_async(stream).await {
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

        // Create client connection
        let client_id = uuid::Uuid::new_v4().to_string();
        let (tx, mut rx) = futures_channel::mpsc::unbounded();

        let client_connection = ClientConnection {
            id: client_id.clone(),
            ws_tx: tx,
            client_type: ClientType::WebBrowser, // Default
            connected_at: std::time::SystemTime::now(),
        };

        // Add to clients — lock held only for the push, dropped before await points.
        {
            let mut clients_write = clients.write();
            clients_write.push(client_connection);
            println!("Client {} connected, total: {}", client_id, clients_write.len());
        }

        // Handle messages from client
        let clients_clone = Arc::clone(&clients);
        let client_id_clone = client_id.clone();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // Handle messages from server to client
                    Some(msg) = rx.next() => {
                        if ws_sender.send(msg).await.is_err() {
                            break;
                        }
                    }
                    // Handle messages from client to server
                    Some(msg) = ws_receiver.next() => {
                        if let Ok(msg) = msg {
                            if let Ok(text) = msg.into_text() {
                                handle_client_message(text.to_string(), &clients_clone, &client_id_clone).await;
                            }
                        }
                    }
                    else => break,
                }
            }

            // Remove client on disconnect — lock held only for retain.
            {
                let mut clients_write = clients_clone.write();
                clients_write.retain(|c| c.id != client_id_clone);
                println!("Client {} disconnected, remaining: {}", client_id_clone, clients_write.len());
            }
        });
    }
}

async fn handle_client_message(
    message: String,
    clients: &Arc<RwLock<Vec<ClientConnection>>>,
    client_id: &str,
) {
    // Parse client message (could be commands, requests, etc.)
    if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&message) {
        match client_msg {
            ClientMessage::Ping => {
                // Respond to ping
                let pong = Message::Text(
                    serde_json::json!({"type": "pong"}).to_string().into()
                );
                let clients_read = clients.read();
                if let Some(client) = clients_read.iter().find(|c| c.id == client_id) {
                    let _ = client.ws_tx.unbounded_send(pong);
                }
            }
            ClientMessage::RequestState => {
                // In real implementation: send current teleprompter state
            }
            ClientMessage::SetClientType(client_type) => {
                // Update client type
                let mut clients_write = clients.write();
                if let Some(client) = clients_write.iter_mut().find(|c| c.id == client_id) {
                    client.client_type = client_type;
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClientInfo {
    pub id: String,
    pub client_type: ClientType,
    pub connected_at: std::time::SystemTime,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    Ping,
    RequestState,
    SetClientType(ClientType),
}

impl Default for ClientMessage {
    fn default() -> Self {
        Self::Ping
    }
}
