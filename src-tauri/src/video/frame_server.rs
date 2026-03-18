use axum::extract::State;
use axum::http::header;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use bytes::Bytes;
use parking_lot::RwLock;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;
use tracing::info;

/// Shared state for the frame server
pub struct FrameServerState {
    pub config_frame: RwLock<Option<Bytes>>,
    pub last_keyframe: RwLock<Option<Bytes>>,
    pub frame_tx: broadcast::Sender<Bytes>,
    pub audio_tx: broadcast::Sender<Bytes>,
}

/// Local HTTP server that streams raw H.264 video + Opus audio
pub struct FrameServer {
    state: Arc<FrameServerState>,
    port: u16,
}

impl FrameServer {
    pub fn new() -> Self {
        let (frame_tx, _) = broadcast::channel(256);
        let (audio_tx, _) = broadcast::channel(256);
        Self {
            state: Arc::new(FrameServerState {
                config_frame: RwLock::new(None),
                last_keyframe: RwLock::new(None),
                frame_tx,
                audio_tx,
            }),
            port: 0,
        }
    }

    pub async fn start(&mut self) -> anyhow::Result<u16> {
        let state = self.state.clone();

        let app = Router::new()
            .route("/stream", get(handle_stream))
            .route("/audio", get(handle_audio))
            .route("/health", get(|| async { "ok" }))
            .layer(CorsLayer::permissive())
            .with_state(state);

        let addr = SocketAddr::from(([127, 0, 0, 1], 0));
        let listener = tokio::net::TcpListener::bind(addr).await?;
        let port = listener.local_addr()?.port();
        self.port = port;

        info!("Frame server listening on http://127.0.0.1:{}", port);

        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        Ok(port)
    }

    pub fn send_frame(&self, data: Bytes) {
        let frame_type = Self::detect_frame_type(&data);
        if frame_type == FrameType::Config {
            *self.state.config_frame.write() = Some(data.clone());
        } else if frame_type == FrameType::Keyframe {
            *self.state.last_keyframe.write() = Some(data.clone());
        }
        let _ = self.state.frame_tx.send(data);
    }

    pub fn send_audio(&self, data: Bytes) {
        let _ = self.state.audio_tx.send(data);
    }

    fn detect_frame_type(data: &[u8]) -> FrameType {
        for i in 0..data.len().saturating_sub(4) {
            if data[i] == 0 && data[i + 1] == 0 && data[i + 2] == 0 && data[i + 3] == 1 {
                if i + 4 < data.len() {
                    let nal_type = data[i + 4] & 0x1F;
                    match nal_type {
                        7 => return FrameType::Config,
                        5 => return FrameType::Keyframe,
                        _ => {}
                    }
                }
            }
        }
        FrameType::Delta
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn set_init_segment(&self, _data: Bytes) {}
    pub fn send_segment(&self, _data: Bytes) {}
}

#[derive(PartialEq)]
enum FrameType {
    Config,
    Keyframe,
    Delta,
}

/// Streams raw H.264 video
async fn handle_stream(
    State(state): State<Arc<FrameServerState>>,
) -> impl IntoResponse {
    let mut rx = state.frame_tx.subscribe();
    let config = state.config_frame.read().clone();
    let keyframe = state.last_keyframe.read().clone();

    let stream = async_stream::stream! {
        if let Some(config_data) = config {
            let len = (config_data.len() as u32).to_be_bytes();
            yield Ok::<Bytes, std::io::Error>(Bytes::copy_from_slice(&len));
            yield Ok(config_data);
        }
        if let Some(kf_data) = keyframe {
            let len = (kf_data.len() as u32).to_be_bytes();
            yield Ok::<Bytes, std::io::Error>(Bytes::copy_from_slice(&len));
            yield Ok(kf_data);
        }
        loop {
            match rx.recv().await {
                Ok(frame_data) => {
                    let len = (frame_data.len() as u32).to_be_bytes();
                    yield Ok::<Bytes, std::io::Error>(Bytes::copy_from_slice(&len));
                    yield Ok(frame_data);
                }
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    };

    (
        [
            (header::CONTENT_TYPE, "application/octet-stream"),
            (header::CACHE_CONTROL, "no-cache"),
        ],
        axum::body::Body::from_stream(stream),
    )
}

/// Streams raw Opus audio packets with 4-byte length prefix
async fn handle_audio(
    State(state): State<Arc<FrameServerState>>,
) -> impl IntoResponse {
    let mut rx = state.audio_tx.subscribe();

    let stream = async_stream::stream! {
        loop {
            match rx.recv().await {
                Ok(audio_data) => {
                    let len = (audio_data.len() as u32).to_be_bytes();
                    yield Ok::<Bytes, std::io::Error>(Bytes::copy_from_slice(&len));
                    yield Ok(audio_data);
                }
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    };

    (
        [
            (header::CONTENT_TYPE, "application/octet-stream"),
            (header::CACHE_CONTROL, "no-cache"),
        ],
        axum::body::Body::from_stream(stream),
    )
}
