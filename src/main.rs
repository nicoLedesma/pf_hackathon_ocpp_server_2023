use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use std::str::FromStr;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpListener;
use tokio::task;
use tokio_tungstenite::accept_async;
use tungstenite::Message;

#[derive(Clone, Copy)]
enum Protocol {
    WS,
    WSS,
}

const ADDRESSES: &[(Protocol, &str)] = &[
    (Protocol::WS, "127.0.0.1:8765"),
    (Protocol::WSS, "127.0.0.1:8766"),
];

#[tokio::main]
async fn main() {
    let mut tasks = Vec::new();

    for &(protocol, address) in ADDRESSES {
        let server_task = match protocol {
            Protocol::WS => {
                println!("Will listen on: ws://{}", address);
                task::spawn(serve_unencrypted(address))
            }
            Protocol::WSS => {
                println!("Will listen on: wss://{}", address);
                task::spawn(serve_encrypted_tls(&address))
            }
        };

        tasks.push(server_task);
    }

    // Wait for all tasks to complete
    for task in tasks {
        task.await.unwrap();
    }
}

async fn serve_unencrypted(addr: &str) {
    // Bind the TCP listener
    let tcp_listener = TcpListener::bind(
        String::from_str(addr).unwrap()
            .parse::<SocketAddr>()
            .expect("Failed to parse socket address")
    )
    .await
    .expect("Failed to bind to address");

    // Accept incoming connections
    loop {
        if let Err(e) = accept_unencrypted_connection(&tcp_listener).await {
            println!("Error! {:?}", e);
        }
    }
}

async fn serve_encrypted_tls(addr: &str) {
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
    let tcp_listener = TcpListener::bind(
        String::from_str(addr).unwrap()
            .parse::<SocketAddr>()
            .expect("Failed to parse socket address")
    )
    .await
    .expect("Failed to bind to address");

    // Accept incoming connections
    loop {
        if let Err(e) = accept_tls_connection(&tcp_listener, &tls_acceptor).await {
            println!("Error! {:?}", e);
        }
    }
}

async fn accept_tls_connection(
    tcp_listener: &TcpListener,
    tls_acceptor: &tokio_native_tls::TlsAcceptor,
) -> Result<(), Box<dyn std::error::Error>> {
    let (tcp_stream, _) = tcp_listener.accept().await?;
    let peer_addr = tcp_stream
        .peer_addr()
        .expect("Unable to find new connection's incoming address");
    println!("Connection received from {}", peer_addr);
    let tls_stream = tls_acceptor.accept(tcp_stream).await?;
    tokio::spawn(handle_connection(tls_stream, peer_addr));
    Ok(())
}

async fn accept_unencrypted_connection(
    tcp_listener: &TcpListener,
) -> Result<(), Box<dyn std::error::Error>> {
    let (tcp_stream, _) = tcp_listener.accept().await?;
    let peer_addr = tcp_stream
        .peer_addr()
        .expect("Unable to find new connection's incoming address");
    println!("Connection received from {}", peer_addr);
    tokio::spawn(handle_connection(tcp_stream, peer_addr));
    Ok(())
}

async fn handle_connection<S>(stream: S, peer_addr: SocketAddr)
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    // Accept the WebSocket handshake
    let mut ws_stream = accept_async(stream)
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
                /*
                                Are any of these errors recoverable?

                                Chat GPT 4:
                                WebSocket protocol errors can be categorized into two types: recoverable and non-recoverable errors. It's important to understand which errors can be recovered from and which require closing the WebSocket connection.

                                Recoverable errors:
                                Recoverable errors are those that can be handled by the server without closing the WebSocket connection. These errors might include:

                                Invalid message format: If the server receives a message with an invalid format, it may ignore the message and continue processing subsequent messages.
                                Application-level errors: Errors that occur within the application logic that uses the WebSocket connection can often be handled without closing the connection. For example, if a chat server receives a malformed chat message, it could respond with an error message to the client but keep the connection open for further communication.

                                Non-recoverable errors:
                                Non-recoverable errors are those that require closing the WebSocket connection. These errors often involve issues with the WebSocket protocol itself or with the underlying transport (e.g., TCP). Examples of non-recoverable errors include:

                                Protocol violations: If a client sends a message that violates the WebSocket protocol, such as using a reserved opcode or providing an incorrect payload length, the server should close the connection.
                                Connection issues: If the underlying TCP connection is lost or experiences severe issues (e.g., high latency, packet loss), the WebSocket connection may need to be closed.
                                Authentication or authorization issues: If a client fails to authenticate or does not have the necessary permissions to perform an action, the server may decide to close the connection.
                                Resource constraints: If the server is running low on resources (e.g., memory, CPU), it might decide to close some WebSocket connections to free up resources.

                In general, it's essential to handle errors gracefully and only close the WebSocket connection when necessary. Always consider the specific requirements and constraints of your application when deciding how to handle errors.
                                */
                eprintln!("Error while processing websocket message: {}", err);
                return;
            }
        }
    }
}
