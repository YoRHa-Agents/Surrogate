use crate::bridge_trait::{FlowMetadata, PlatformBridge};
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use surrogate_contract::error::BridgeError;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Notify;

/// Linux bridge using an explicit HTTP/SOCKS5 proxy listener.
///
/// Binds a TCP listener on [`Self::listen_addr`] and accepts inbound
/// connections that will be handed off to the kernel for proxying.
pub struct LinuxExplicitProxyBridge {
    listen_addr: SocketAddr,
    shutdown: Arc<Notify>,
    bound_addr: Mutex<Option<SocketAddr>>,
}

impl LinuxExplicitProxyBridge {
    pub fn new(listen_addr: SocketAddr) -> Self {
        Self {
            listen_addr,
            shutdown: Arc::new(Notify::new()),
            bound_addr: Mutex::new(None),
        }
    }

    /// Returns the address the listener actually bound to (available after
    /// [`PlatformBridge::start`] succeeds). Useful when binding to port 0.
    pub fn bound_addr(&self) -> Option<SocketAddr> {
        *self.bound_addr.lock().unwrap()
    }
}

impl PlatformBridge for LinuxExplicitProxyBridge {
    fn start(&self) -> Pin<Box<dyn Future<Output = Result<(), BridgeError>> + Send + '_>> {
        Box::pin(async {
            let listener = TcpListener::bind(self.listen_addr)
                .await
                .map_err(|e| BridgeError::PlatformApi(format!("failed to bind listener: {e}")))?;

            let local_addr = listener
                .local_addr()
                .map_err(|e| BridgeError::PlatformApi(format!("failed to get local addr: {e}")))?;

            {
                let mut addr = self.bound_addr.lock().unwrap();
                *addr = Some(local_addr);
            }

            let shutdown = Arc::clone(&self.shutdown);
            tokio::spawn(async move {
                loop {
                    tokio::select! {
                        result = listener.accept() => {
                            match result {
                                Ok((_stream, _peer)) => {
                                    // Future: hand connection off to the proxy kernel.
                                }
                                Err(e) => {
                                    eprintln!("bridge accept error: {e}");
                                    break;
                                }
                            }
                        }
                        _ = shutdown.notified() => break,
                    }
                }
            });

            Ok(())
        })
    }

    fn stop(&self) -> Pin<Box<dyn Future<Output = Result<(), BridgeError>> + Send + '_>> {
        Box::pin(async {
            self.shutdown.notify_one();
            Ok(())
        })
    }

    fn platform_name(&self) -> &str {
        "linux"
    }

    fn supports_process_identification(&self) -> bool {
        false
    }
}

/// Extract client metadata from an accepted TCP stream.
///
/// With an explicit proxy the source process cannot be identified, so
/// `process_name`, `process_id`, and `bundle_id` are always `None`.
pub fn extract_client_metadata(stream: &TcpStream) -> FlowMetadata {
    let source_addr = stream
        .peer_addr()
        .unwrap_or_else(|_| SocketAddr::from(([0, 0, 0, 0], 0)));
    let dest_addr = stream
        .local_addr()
        .unwrap_or_else(|_| SocketAddr::from(([0, 0, 0, 0], 0)));

    FlowMetadata {
        source_addr,
        dest_addr,
        process_name: None,
        process_id: None,
        bundle_id: None,
    }
}
