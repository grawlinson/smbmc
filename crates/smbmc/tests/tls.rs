//! TLS behaviour against a local HTTPS server with a self-signed certificate.

#![cfg(feature = "rustls")]

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::Arc;

use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer};
use smbmc::{Client, Error};

/// Serve `connections` HTTPS connections on a random port, answering every
/// request with a successful login. Returns the base URL.
fn https_server(connections: usize) -> String {
    let key = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
    let cert = CertificateDer::from(key.cert.der().to_vec());
    let pkey = PrivatePkcs8KeyDer::from(key.signing_key.serialize_der());
    let config = Arc::new(
        rustls::ServerConfig::builder_with_provider(Arc::new(
            rustls::crypto::ring::default_provider(),
        ))
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(vec![cert], pkey.into())
        .unwrap(),
    );

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        for stream in listener.incoming().take(connections) {
            let conn = rustls::ServerConnection::new(Arc::clone(&config)).unwrap();
            let mut tls = rustls::StreamOwned::new(conn, stream.unwrap());
            // Errors are expected when the client rejects the certificate.
            let _ = serve_login(&mut tls);
        }
    });
    format!("https://localhost:{port}")
}

fn serve_login(stream: &mut (impl Read + Write)) -> std::io::Result<()> {
    let mut reader = BufReader::new(&mut *stream);
    let mut length = 0;
    loop {
        let mut line = String::new();
        reader.read_line(&mut line)?;
        if let Some(v) = line.to_ascii_lowercase().strip_prefix("content-length:") {
            length = v.trim().parse().unwrap_or(0);
        }
        if line == "\r\n" || line.is_empty() {
            break;
        }
    }
    let mut body = vec![0; length];
    reader.read_exact(&mut body)?;
    stream.write_all(
        b"HTTP/1.1 200 OK\r\nSet-Cookie: SID=tls; path=/\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
    )?;
    stream.flush()
}

#[tokio::test]
async fn self_signed_rejected_by_default() {
    let client = Client::new(&https_server(1), "u", "p").unwrap();
    let err = client.login().await.unwrap_err();
    assert!(matches!(err, Error::Http(_)), "{err:?}");
}

#[tokio::test]
async fn self_signed_accepted_when_allowed() {
    let client = Client::builder(&https_server(1), "u", "p")
        .accept_invalid_certs(true)
        .build()
        .unwrap();
    client.login().await.unwrap();
}
