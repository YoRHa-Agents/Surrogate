use std::future::Future;
use std::io;
use std::pin::Pin;

/// Boxed future returned by [`Transport::connect`].
pub type ConnectFuture<'a> =
    Pin<Box<dyn Future<Output = io::Result<Box<dyn TransportStream>>> + Send + 'a>>;

/// Abstraction for outbound connections, enabling pluggable transports
/// (direct TCP, TLS-wrapped, proxy-chained, etc.)
pub trait Transport: Send + Sync {
    /// Open a connection to `addr` (in `host:port` form) and return a
    /// bidirectional byte stream.
    fn connect(&self, addr: &str) -> ConnectFuture<'_>;
}

/// A bidirectional byte stream for relay.
pub trait TransportStream: tokio::io::AsyncRead + tokio::io::AsyncWrite + Send + Unpin {}

impl<T> TransportStream for T where T: tokio::io::AsyncRead + tokio::io::AsyncWrite + Send + Unpin {}

#[cfg(test)]
mod tests {
    use super::*;

    struct FailingTransport;

    impl Transport for FailingTransport {
        fn connect(&self, addr: &str) -> ConnectFuture<'_> {
            let addr = addr.to_string();
            Box::pin(async move {
                Err(io::Error::new(
                    io::ErrorKind::ConnectionRefused,
                    format!("cannot connect to {addr}"),
                ))
            })
        }
    }

    #[tokio::test]
    async fn dummy_transport_connect() {
        let transport = FailingTransport;
        let result = transport.connect("127.0.0.1:80").await;
        match result {
            Err(err) => {
                assert_eq!(err.kind(), io::ErrorKind::ConnectionRefused);
                assert!(err.to_string().contains("127.0.0.1:80"));
            }
            Ok(_) => panic!("expected connection error"),
        }
    }

    #[test]
    fn transport_stream_blanket_impl() {
        fn assert_transport_stream<T: TransportStream>() {}
        assert_transport_stream::<tokio::net::TcpStream>();
    }
}
