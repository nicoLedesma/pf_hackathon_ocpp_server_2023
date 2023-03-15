use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_native_tls::TlsStream;
use tokio_tungstenite::accept_async;
use tungstenite::Message;

#[tokio::main]
async fn main() {
    // Load the TLS certificate and private key
    let cert = tokio::fs::read("cert.pem").await.unwrap();
    let key = tokio::fs::read_to_string("key.pem").await.unwrap();
    let pkcs12 = openssl::pkcs12::Pkcs12::from_der(&cert).unwrap();
    let identity =
        tokio_native_tls::native_tls::Identity::from_pkcs12(&pkcs12.to_der().unwrap(), &key)
            .unwrap();

    // Create the TLS acceptor
    let tls_acceptor = tokio_native_tls::TlsAcceptor::from(
        native_tls::TlsAcceptor::builder(identity).build().unwrap(),
    );

    // Bind the TCP listener
    let addr = "0.0.0.0:8080".parse::<SocketAddr>().unwrap();
    let tcp_listener = TcpListener::bind(&addr).await.unwrap();

    println!("Listening on: {}", addr);

    // Accept incoming connections
    loop {
        let (tcp_stream, _) = tcp_listener.accept().await.unwrap();
        let tls_stream = tls_acceptor.accept(tcp_stream).await.unwrap();
        tokio::spawn(handle_connection(tls_stream));
    }
}

async fn handle_connection(tls_stream: TlsStream<TcpStream>) {
    // Accept the WebSocket handshake
    let mut ws_stream = accept_async(tls_stream).await.unwrap();

    // Handle incoming messages
    while let Some(msg) = ws_stream.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                println!("Received text message: {}", text);
            }
            Ok(Message::Binary(data)) => {
                println!("Received binary message with length: {}", data.len());
            }
            Ok(Message::Ping(data)) => {
                ws_stream.send(Message::Pong(data)).await.unwrap();
            }
            Ok(Message::Pong(_)) => {}
            Ok(Message::Close(_)) => {
                break;
            }
            Ok(Message::Frame(_)) => {}
            Err(err) => {
                eprintln!("Error receiving message: {:?}", err);
                break;
            }
        }
    }
}
