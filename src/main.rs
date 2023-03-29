// TODO close connections gracefully (on SIGTERM/SIGKILL or as needed)
// TODO NATS API to send OCPP messages
use tracing::{info_span, instrument, Instrument};

use futures_util::{SinkExt, StreamExt};
use log::{error, info};
use std::net::SocketAddr;
use std::os::unix::prelude::PermissionsExt;
use std::str::FromStr;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpListener;
use tokio::task;
use tokio_rustls::rustls::ServerConfig;
use tokio_rustls::TlsAcceptor;
use tokio_tungstenite::accept_hdr_async;
use tungstenite::Message;
use x509_parser::pem::Pem;
use x509_parser::x509::X509Version;

pub mod evse_state;
pub mod logstuff;
pub mod metrics;
pub mod normalize_input;
pub mod ocpp;
pub mod ocpp_handlers;

const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
const TLS_CERTIFICATE_PEM_FILENAME: &str = "./certificate.pem";
const TLS_PRIVATE_KEY_PEM_FILENAME: &str = "./private_key.pem";

#[derive(Clone, Copy)]
enum Protocol {
    Ws,
    Wss,
}

const ADDRESSES: &[(Protocol, &str)] = &[
    // (Protocol::Ws, "0.0.0.0:8765"),
    (Protocol::Wss, "0.0.0.0:5678"),
];

#[tokio::main]
async fn main() {
    crate::logstuff::setup_logger();
    let current_exe = std::env::current_exe().expect("Current exe's path not found");
    let my_metadata = std::fs::metadata(&current_exe).expect("Unable to read exe's metadata");
    info!(
        "Hello, world! Executing {} {} bytes ({:0o}), version {}",
        current_exe.to_string_lossy(),
        my_metadata.len(),
        my_metadata.permissions().mode(),
        CARGO_PKG_VERSION,
    );

    tracing::info!(service = "ocpp.rs", version = "v0", "Starting up");

    let mut tasks = Vec::new();

    for &(protocol, address) in ADDRESSES {
        let server_task = match protocol {
            Protocol::Ws => {
                info!("Will listen on: ws://{}", address);
                task::spawn(serve_unencrypted(address))
            }
            Protocol::Wss => {
                info!("Will listen on: wss://{}", address);
                task::spawn(serve_encrypted_tls(address))
            }
        };

        tasks.push(server_task);
    }

    tasks.push(tokio::spawn(crate::metrics::start_http_server()));

    // Wait for all tasks to complete
    for task in tasks {
        if let Err(e) = task.await {
            error!("Task failed! {}", e);
        }
    }
}

#[instrument]
async fn bind(addr: &str) -> Result<TcpListener, Box<dyn std::error::Error>> {
    // Bind the TCP listener
    let bound = Ok(TcpListener::bind(
        String::from_str(addr)
            .unwrap()
            .parse::<SocketAddr>()
            .expect("Failed to parse socket address"),
    )
    .await
    .expect(&format!("Failed to bind to address {}", &addr)));

    tracing::info!("Bound to address {}", &addr);

    bound
}

#[instrument]
async fn serve_unencrypted(addr: &str) {
    // Accept incoming connections
    let tcp_listener = bind(addr).await.unwrap();
    loop {
        if let Err(e) = accept_unencrypted_connection(addr, &tcp_listener).await {
            error!("ERROR! {:?}", e);
        }
    }
}

#[instrument]
async fn serve_encrypted_tls(addr: &str) {
    // Bind the TCP listener before attempting to load any TLS stuff
    // TODO how to get rid of this unwrap? I'm getting future returned by `serve_encrypted_tls` is not `Send`
    let tcp_listener = bind(addr).await.unwrap();

    // TODO print and validate permissions. private key should be 0o600 perms
    let tls_certificate_pem = tokio::fs::read(TLS_CERTIFICATE_PEM_FILENAME)
        .await
        .expect("Failed to read the TLS certificate file");
    let tls_private_key_pem = tokio::fs::read(TLS_PRIVATE_KEY_PEM_FILENAME)
        .await
        .expect("Failed to read the TLS password file");

    // Parse and validate certificate
    // TODO verify certificate expiration date and other metadata
    let mut pem_contains_at_least_one_cert = false;
    let mut server_tls_name: Option<String> = None;
    for (index, pem) in Pem::iter_from_buffer(&tls_certificate_pem).enumerate() {
        pem_contains_at_least_one_cert = true;
        let pem = pem.expect("Reading next PEM block failed");
        let x509 = pem.parse_x509().expect(&format!(
            "X.509: decoding DER failed of PEM cert index {}",
            index
        ));
        assert_eq!(x509.tbs_certificate.version, X509Version::V3);
        if index == 0 {
            let san = x509
                .subject_alternative_name()
                .expect(
                    "Expect Subject Alternative Name field in the end-entity certificate (index 0)",
                )
                .expect("um");
            info!("Loaded cert with Subject alt names (SNA): {:?}", san.value);
            server_tls_name = Some(format!("{:?}", san.value));
        }
    }
    assert!(pem_contains_at_least_one_cert);

    // Load certificate without validation
    let der_encoded_certificate_chain = rustls_pemfile::certs(&mut tls_certificate_pem.as_ref())
        .expect("Unable to convert certificate PEM into DER")
        .into_iter()
        .map(|bytes| tokio_rustls::rustls::Certificate(bytes))
        .collect();

    // Load private key without validation
    // TODO verify this private key corresponds to the certificate
    let der_encoded_private_key_chain =
        match rustls_pemfile::read_one(&mut tls_private_key_pem.as_ref())
            .expect("No PEM section found in the given private key")
            .expect("No valid item found in PEM")
        {
            rustls_pemfile::Item::RSAKey(contents)
            | rustls_pemfile::Item::PKCS8Key(contents)
            | rustls_pemfile::Item::ECKey(contents) => {
                Some(tokio_rustls::rustls::PrivateKey(contents))
            }
            _ => None,
        }
        .expect("Unable to find a private key in the given PEM");

    let config = ServerConfig::builder()
        .with_safe_default_cipher_suites()
        .with_safe_default_kx_groups()
        .with_protocol_versions(&[
            &tokio_rustls::rustls::version::TLS13,
            &tokio_rustls::rustls::version::TLS12,
        ])
        .expect("Unable to set TLS settings")
        .with_no_client_auth()
        .with_single_cert(der_encoded_certificate_chain, der_encoded_private_key_chain)
        .expect("Unable to build rustls ServerConfig");
    let tls_acceptor = TlsAcceptor::from(std::sync::Arc::new(config));

    // TODO PRINT TLS HOSTNAME let sni_hostname = tls_acceptor.sni_hostname();

    // Accept incoming connections
    loop {
        if let Err(e) = async { accept_tls_connection(addr, &tcp_listener, &tls_acceptor).await }
            .instrument(info_span!("tls", server_tls_name))
            .await
        {
            error!("ERROR! {:?}", e);
        }
    }
}

async fn accept_tls_connection(
    addr: &str,
    tcp_listener: &TcpListener,
    tls_acceptor: &TlsAcceptor,
) -> Result<(), Box<dyn std::error::Error>> {
    let (tcp_stream, peer_addr) = async { tcp_listener.accept().await }
        .instrument(info_span!("Accepted TCP connection"))
        .await?;
    info!(
        "Secured TLS Connection to {} received from {}",
        addr, peer_addr
    );

    // let mut tls_stream = tls_acceptor.accept(tcp_stream).await?;
    //use tokio::io::AsyncReadExt;
    //loop {
    //    let mut buffer = [0; 256];

    //    let n = tls_stream.read(&mut buffer).await.unwrap();

    //    if n == 0 {
    //        continue;
    //    }
    //    info!("The bytes: {:?}", &buffer[..n]);
    //}

    let now = std::time::Instant::now();
    let tls_stream = tls_acceptor.accept(tcp_stream).await?;

    crate::metrics::TLS_HANDSHAKE_TIME
        .with_label_values(&[peer_addr.to_string().as_str()])
        .set(now.elapsed().as_secs_f64());

    tokio::spawn(handle_connection_with_tracing(
        tls_stream,
        peer_addr,
        addr.to_string(),
    ));
    Ok(())
}

#[tracing::instrument]
async fn accept_unencrypted_connection(
    addr: &str,
    tcp_listener: &TcpListener,
) -> Result<(), Box<dyn std::error::Error>> {
    let (tcp_stream, peer_addr) = tcp_listener.accept().await?;
    info!(
        "Unencrypted Connection to {} received from {}",
        addr, peer_addr
    );
    tokio::spawn(handle_connection_with_tracing(
        tcp_stream,
        peer_addr,
        addr.to_string(),
    ));
    Ok(())
}

async fn handle_connection_with_tracing<S>(stream: S, peer_addr: SocketAddr, addr: String)
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    async { handle_connection(stream, peer_addr).await }
        .instrument(info_span!(
            "websocket",
            addr,
            peer_addr = peer_addr.to_string()
        ))
        .await
}

async fn handle_connection<S>(stream: S, peer_addr: SocketAddr)
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let serial_number_mutex = std::sync::Mutex::new("?".to_string());
    let http_callback =
        |req: &tungstenite::handshake::server::Request,
         response: tungstenite::handshake::server::Response| {
            info!("The request's path is: {}", req.uri().path());
            serial_number_mutex
                .lock()
                .unwrap()
                .replace_range(.., req.uri().path());
            for (ref header, _value) in req.headers() {
                info!("* {}: {:?}", header, _value);
            }
            Ok(response)
        };

    // Accept the WebSocket handshake
    let now = std::time::Instant::now();
    let mut ws_stream = accept_hdr_async(stream, http_callback)
        .await
        .expect("Failed to accept websocket connection");
    let s = serial_number_mutex.lock().unwrap().clone();
    let serial_number = s.as_str();
    crate::metrics::WEBSOCKET_HANDSHAKE_TIME
        .with_label_values(&[peer_addr.to_string().as_str(), serial_number])
        .set(now.elapsed().as_secs_f64());

    tracing::info!("Accepted websocket connection");

    let mut state: crate::evse_state::EvseState = crate::evse_state::EvseState::Empty;

    // Handle incoming messages
    while let Some(msg) = ws_stream.next().await {
        handle_websocket_msg(msg, &mut ws_stream, &mut state, serial_number, peer_addr)
            .instrument(info_span!("msg"))
            .await
    }
}

pub async fn handle_websocket_msg<S>(
    msg: Result<tokio_tungstenite::tungstenite::Message, tokio_tungstenite::tungstenite::Error>,
    ws_stream: &mut tokio_tungstenite::WebSocketStream<S>,
    state: &mut crate::evse_state::EvseState,
    serial_number: &str,
    peer_addr: SocketAddr,
) where
    S: AsyncRead + AsyncWrite + Unpin,
{
    info!("Websocket message from {}", &peer_addr);

    match msg {
        Ok(msg_contents) => {
            crate::metrics::WEBSOCKET_BYTES_RECEIVED
                .with_label_values(&[serial_number])
                .inc_by(msg_contents.len() as u64);
            match msg_contents {
                Message::Text(text) => {
                    info!("Received Text message: {}", text);
                    crate::metrics::OCPP_MESSAGE_BYTES_RECEIVED
                        .with_label_values(&[serial_number])
                        .inc_by(text.len() as u64);
                    tracing::info!("Received Text from websocket connection");

                    let raw_response =
                        crate::ocpp_handlers::ocpp_process_and_respond_str(text, state);

                    match raw_response {
                        Ok(response) => {
                            info!("Sending response: {}", response);
                            crate::metrics::OCPP_MESSAGE_BYTES_SENT
                                .with_label_values(&[serial_number])
                                .inc_by(response.len() as u64);
                            tracing::info!("Sending Text to websocket connection");
                            ws_stream
                                .send(Message::Text(response))
                                .await
                                .expect("Failed to send response to websocket Text message");
                        }
                        Err(err) => {
                            error!("ERROR parsing request and generating response: {}", err)
                        }
                    }
                }
                Message::Binary(data) => {
                    info!("Received Binary message with length: {}", data.len());
                }
                Message::Ping(data) => {
                    info!("Received PING message with length: {}", data.len());
                    let ping_now = std::time::Instant::now();
                    ws_stream
                        .send(Message::Pong(data))
                        .await
                        .expect("Failed to send PONG in response to PING websocket message");

                    crate::metrics::WEBSOCKET_PONG_TRANSMIT_TIME
                        .with_label_values(&[peer_addr.to_string().as_str(), serial_number])
                        .set(ping_now.elapsed().as_secs_f64());

                    info!("Sent PONG message in response to PING");
                }
                Message::Pong(data) => {
                    info!("Received PONG message with length: {}", data.len());
                }
                Message::Close(data) => {
                    info!("Received CLOSE message {:?}", data);
                    return;
                }
                Message::Frame(data) => {
                    info!("Received FRAME message with length: {}", data.len());
                }
            }
        }
        Err(tungstenite::Error::Protocol(
            tungstenite::error::ProtocolError::ResetWithoutClosingHandshake,
        )) => {
            error!("ERROR Client closed without Websocket Closing Handshake");
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
            error!("ERROR while processing websocket message: {}", err);
            return;
        }
    }
}
