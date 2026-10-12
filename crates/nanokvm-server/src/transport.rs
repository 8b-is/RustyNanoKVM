//! Validate transport and certificates before initializing hardware.
use axum::Router;
use axum_server::tls_rustls::RustlsConfig;
use std::{
    future::Future,
    io,
    net::{Ipv4Addr, SocketAddr, TcpListener},
    time::Duration,
};

pub enum PreparedTransport {
    Http(SocketAddr),
    Https(SocketAddr, RustlsConfig),
}

pub fn listener_address(protocol: &str, http_port: u16, https_port: u16) -> io::Result<SocketAddr> {
    let port = match protocol {
        "http" => http_port,
        "https" => https_port,
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Unsupported server protocol",
            ));
        }
    };
    if port == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Server port must be nonzero",
        ));
    }
    Ok(SocketAddr::from((Ipv4Addr::UNSPECIFIED, port)))
}

impl PreparedTransport {
    pub async fn prepare(
        protocol: &str,
        http_port: u16,
        https_port: u16,
        certificate: &str,
        key: &str,
    ) -> io::Result<Self> {
        let addr = listener_address(protocol, http_port, https_port)?;
        if protocol == "https" {
            let config = RustlsConfig::from_pem_file(certificate, key).await?;
            Ok(Self::Https(addr, config))
        } else {
            Ok(Self::Http(addr))
        }
    }

    pub fn address(&self) -> SocketAddr {
        match self {
            Self::Http(addr) | Self::Https(addr, _) => *addr,
        }
    }

    pub async fn serve(
        self,
        app: Router,
        shutdown: impl Future<Output = ()> + Send + 'static,
    ) -> io::Result<()> {
        let listener = TcpListener::bind(self.address())?;
        self.serve_with_listener(listener, app, shutdown).await
    }

    // Explicit listener injection lets tests use ephemeral loopback without hardware.
    pub async fn serve_with_listener(
        self,
        listener: TcpListener,
        app: Router,
        shutdown: impl Future<Output = ()> + Send + 'static,
    ) -> io::Result<()> {
        listener.set_nonblocking(true)?;
        match self {
            Self::Http(_) => {
                axum::serve(tokio::net::TcpListener::from_std(listener)?, app)
                    .with_graceful_shutdown(shutdown)
                    .await
            }
            Self::Https(_, config) => {
                let handle = axum_server::Handle::new();
                let shutdown_handle = handle.clone();
                let waiter = tokio::spawn(async move {
                    shutdown.await;
                    shutdown_handle.graceful_shutdown(Some(Duration::from_secs(10)));
                });
                let result = match axum_server::from_tcp_rustls(listener, config) {
                    Ok(server) => server.handle(handle).serve(app.into_make_service()).await,
                    Err(error) => Err(error),
                };
                waiter.abort();
                let _ = waiter.await;
                result
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selects_only_requested_protocol_port() {
        assert_eq!(listener_address("http", 8080, 8443).unwrap().port(), 8080);
        assert_eq!(listener_address("https", 8080, 8443).unwrap().port(), 8443);
    }
    #[test]
    fn invalid_protocol_and_selected_port_fail() {
        assert!(listener_address("https", 80, 0).is_err());
        assert!(listener_address("http", 0, 443).is_err());
        for protocol in ["", "HTTPS", "ftp", " http"] {
            assert!(listener_address(protocol, 80, 443).is_err());
        }
    }
}
