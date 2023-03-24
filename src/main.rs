use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use std::str::FromStr;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpListener;
use tokio::task;
use tokio_rustls::rustls::ServerConfig;
use tokio_rustls::TlsAcceptor;
use tokio_tungstenite::accept_async;
use tungstenite::Message;

pub mod evse_state;
pub mod normalize_input;
pub mod ocpp;
pub mod ocpp_handlers;

const TLS_IDENTITY_PKCS12_DER_FILENAME: &str = "./letsencrypt_identity.pkcs12.der";

#[derive(Clone, Copy)]
enum Protocol {
    Ws,
    Wss,
}

const ADDRESSES: &[(Protocol, &str)] = &[
    (Protocol::Ws, "0.0.0.0:8765"),
    (Protocol::Wss, "0.0.0.0:5678"),
];

#[tokio::main]
async fn main() {
    let mut tasks = Vec::new();

    for &(protocol, address) in ADDRESSES {
        let server_task = match protocol {
            Protocol::Ws => {
                println!("Will listen on: ws://{}", address);
                task::spawn(serve_unencrypted(address))
            }
            Protocol::Wss => {
                println!("Will listen on: wss://{}", address);
                task::spawn(serve_encrypted_tls(address))
            }
        };

        tasks.push(server_task);
    }

    // Wait for all tasks to complete
    for task in tasks {
        if let Err(e) = task.await {
            eprintln!("Task failed! {}", e);
        }
    }
}

async fn bind(addr: &str) -> Result<TcpListener, Box<dyn std::error::Error>> {
    // Bind the TCP listener
    Ok(TcpListener::bind(
        String::from_str(addr)
            .unwrap()
            .parse::<SocketAddr>()
            .expect("Failed to parse socket address"),
    )
    .await
    .expect(&format!("Failed to bind to address {}", &addr)))
}

async fn serve_unencrypted(addr: &str) {
    // Accept incoming connections
    let tcp_listener = bind(addr).await.unwrap();
    loop {
        if let Err(e) = accept_unencrypted_connection(addr, &tcp_listener).await {
            eprintln!("ERROR! {:?}", e);
        }
    }
}

async fn serve_encrypted_tls(addr: &str) {
    // Bind the TCP listener before attempting to load any TLS stuff
    // TODO how to get rid of this unwrap? I'm getting future returned by `serve_encrypted_tls` is not `Send`
    let tcp_listener = bind(addr).await.unwrap();

    // Load the TLS certificate and private key from the Identity file
    let server_identity_pkcs12_der = tokio::fs::read(TLS_IDENTITY_PKCS12_DER_FILENAME)
        .await
        .expect("Failed to read the PKCS12 DER file");
    let password_raw = std::env::var("TLS_IDENTITY_PASSWORD")
        .expect("Failed to load env var TLS_IDENTITY_PASSWORD");
    let password = password_raw.trim_end();

    println!(
        "Loaded data for TLS cert and private key files for address {}",
        &addr,
    );

    let openssl_pkcs12 = openssl::pkcs12::Pkcs12::from_der(server_identity_pkcs12_der.as_slice())
        .expect("Failed to parse identity file")
        .parse2(password)
        .expect("Failed to parse identity file with given password");
    let cert = openssl_pkcs12
        .cert
        .expect("No TLS certificate found in identity file");
    let ca_cert_chain: Vec<_> = openssl_pkcs12
        .ca
        .map(|stack| stack.into_iter().collect())
        .expect("At least 1 CA cert is needed");

    // TODO check validity of cert, etc
    let private_key = openssl_pkcs12
        .pkey
        .expect("No TLS private key found in identity file");

    println!(
        "Loaded cert with Subject alt names (SNA): {:?}",
        &cert.subject_alt_names().expect("Cert must have SNA field.")
    );

    // Rustls has no support for pkcs12
    let mut cert_chain = ca_cert_chain.clone();
    cert_chain.insert(0, cert);
    let an_entire_pem: Vec<_> = cert_chain
        .into_iter()
        .map(|c| c.to_pem().unwrap())
        .collect();

    let der_encoded_cert_chain: Vec<tokio_rustls::rustls::Certificate> =
        rustls_pemfile::certs(&mut an_entire_pem.concat().as_ref())
            .expect("Parse DER")
            .into_iter()
            .map(|c| tokio_rustls::rustls::Certificate(c))
            .collect();

    let der_encoded_pkey = tokio_rustls::rustls::PrivateKey(
        private_key
            .private_key_to_der()
            .expect("Unable to encode private key as DER"),
    );

    let config = ServerConfig::builder()
        .with_safe_default_cipher_suites()
        .with_safe_default_kx_groups()
        .with_protocol_versions(&[&tokio_rustls::rustls::version::TLS13])
        .expect("Unable to set TLS settings")
        .with_no_client_auth()
        .with_single_cert(der_encoded_cert_chain, der_encoded_pkey)
        .expect("Unable to build rustls ServerConfig");
    let tls_acceptor = TlsAcceptor::from(std::sync::Arc::new(config));

    // PRINT TLS HOSTNAME let sni_hostname = tls_acceptor.sni_hostname();

    // Accept incoming connections
    loop {
        if let Err(e) = accept_tls_connection(addr, &tcp_listener, &tls_acceptor).await {
            eprintln!("ERROR! {:?}", e);
        }
    }
}

async fn accept_tls_connection(
    addr: &str,
    tcp_listener: &TcpListener,
    tls_acceptor: &TlsAcceptor,
) -> Result<(), Box<dyn std::error::Error>> {
    let (tcp_stream, peer_addr) = tcp_listener.accept().await?;
    println!("Connection to {} received from {}", addr, peer_addr);
    let tls_stream = tls_acceptor.accept(tcp_stream).await?;
    tokio::spawn(handle_connection(tls_stream, peer_addr));
    Ok(())
}

async fn accept_unencrypted_connection(
    addr: &str,
    tcp_listener: &TcpListener,
) -> Result<(), Box<dyn std::error::Error>> {
    let (tcp_stream, peer_addr) = tcp_listener.accept().await?;
    println!("Connection to {} received from {}", addr, peer_addr);
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
    let mut state: crate::evse_state::EvseState = crate::evse_state::EvseState::Empty;

    // Handle incoming messages
    while let Some(msg) = ws_stream.next().await {
        println!("Websocket message from {}", &peer_addr);
        match msg {
            Ok(Message::Text(text)) => {
                println!("Received Text message: {}", text);
                let raw_response =
                    crate::ocpp_handlers::ocpp_process_and_respond_str(text, &mut state);

                match raw_response {
                    Ok(response) => {
                        println!("Sending response: {}", response);
                        ws_stream
                            .send(Message::Text(response))
                            .await
                            .expect("Failed to send response to websocket Text message");
                    }
                    Err(err) => {
                        eprintln!("ERROR parsing request and generating response: {}", err)
                    }
                }
            }
            Ok(Message::Binary(data)) => {
                println!("Received Binary message with length: {}", data.len());
            }
            Ok(Message::Ping(data)) => {
                println!("Received PING message with length: {}", data.len());
                println!("Sent PONG message in response to PING {:?}", &data);
                ws_stream
                    .send(Message::Pong(data))
                    .await
                    .expect("Failed to send PONG in response to PING websocket message");
            }
            Ok(Message::Pong(data)) => {
                println!("Received PONG message with length: {}", data.len());
            }
            Ok(Message::Close(data)) => {
                println!("Received CLOSE message {:?}", data);
                return;
            }
            Ok(Message::Frame(data)) => {
                println!("Received FRAME message with length: {}", data.len());
            }
            Err(tungstenite::Error::Protocol(
                tungstenite::error::ProtocolError::ResetWithoutClosingHandshake,
            )) => {
                eprintln!("ERROR Client closed without Websocket Closing Handshake");
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
                eprintln!("ERROR while processing websocket message: {}", err);
                return;
            }
        }
    }
}
