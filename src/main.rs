use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_native_tls::TlsStream;
use tokio_tungstenite::accept_async;
use tungstenite::Message;

#[tokio::main]
async fn main() {
    // Load the TLS certificate and private key from the Identity file
    let server_identity_pkcs12_der = tokio::fs::read("identity.p12.der")
        .await
        .expect("Failed to read the PKCS12 DER file");
    let key = tokio::fs::read_to_string("identity_password.txt")
        .await
        .expect("Failed to read the identity password file");
    let pkcs12 = openssl::pkcs12::Pkcs12::from_der(&server_identity_pkcs12_der)
        .expect("Failed to create Pkcs12 from DER");
    let identity = tokio_native_tls::native_tls::Identity::from_pkcs12(
        &pkcs12.to_der().expect("Failed to convert Pkcs12 to DER"),
        &key,
    )
    .expect("Failed to create Identity from PKCS12 and key");

    // Create the TLS acceptor
    let tls_acceptor = tokio_native_tls::TlsAcceptor::from(
        native_tls::TlsAcceptor::builder(identity)
            .build()
            .expect("Failed to build a native_tls TLS acceptor object"),
    );

    // Bind the TCP listener
    let addr = "127.0.0.1:8765"
        .parse::<SocketAddr>()
        .expect("Failed to parse address");
    let tcp_listener = TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");

    println!("Listening on: {}", addr);

    // Accept incoming connections
    loop {
        if let Err(e) = accept_connection(&tcp_listener, &tls_acceptor).await {
            println!("Error! {:?}", e);
        }
    }
}

async fn accept_connection(
    tcp_listener: &TcpListener,
    tls_acceptor: &tokio_native_tls::TlsAcceptor,
) -> Result<(), Box<dyn std::error::Error>> {
    let (tcp_stream, _) = tcp_listener.accept().await?;
    let peer_addr = tcp_stream
        .peer_addr()
        .expect("Unable to find new connection's incoming address")
        .to_string();
    println!("Connection received from {:?}", peer_addr);
    let tls_stream = tls_acceptor.accept(tcp_stream).await?;
    tokio::spawn(handle_connection(tls_stream, peer_addr));
    Ok(())
}

async fn handle_connection(tls_stream: TlsStream<TcpStream>, peer_addr: String) {
    // Accept the WebSocket handshake
    let mut ws_stream = accept_async(tls_stream)
        .await
        .expect("Failed to accept websocket connection");

    // Handle incoming messages
    while let Some(msg) = ws_stream.next().await {
        println!("Websocket message from {}", &peer_addr);
        match msg {
            Ok(Message::Text(text)) => {
                println!("Received text message: {}", text);
                ws_stream
                    .send(Message::Text("thank you, next!".into()))
                    .await
                    .expect("Failed to send response to websocket Text message");
            }
            Ok(Message::Binary(data)) => {
                println!("Received binary message with length: {}", data.len());
            }
            Ok(Message::Ping(data)) => {
                ws_stream
                    .send(Message::Pong(data))
                    .await
                    .expect("Failed to send PONG in response to PING websocket message");
            }
            Ok(Message::Pong(_)) => {}
            Ok(Message::Close(_)) => {
                break;
            }
            Ok(Message::Frame(_)) => {}
            Err(tungstenite::Error::Protocol(
                tungstenite::error::ProtocolError::ResetWithoutClosingHandshake,
            )) => {
                eprintln!("Client closed without Websocket Closing Handshake");
                return;
            }
            Err(err) => {
                // Are any of these errors recoverable?
                eprintln!("Error while processing websocket message: {}", err);
                return;
            }
        }
    }
}
