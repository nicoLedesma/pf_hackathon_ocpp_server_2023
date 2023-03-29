// TODO [prom]   metadata: version and binary size and tls protocol version
// TODO [prom] measure tls handshake time
// TODO [prom] measure websocket handshake time
// TODO [prom] measure websocket latency
// TODO [prom] measure websocket bandwidth, with and without compression
use lazy_static::lazy_static;
use prometheus::{self, register_int_counter, IntCounter};
use prometheus_hyper::Server;
use std::net::SocketAddr;

const PROMETHEUS_PORT: u16 = 8080;

// TODO add labels lol
lazy_static! {
    pub static ref WEBSOCKET_BYTES_RECEIVED: IntCounter = register_int_counter!(
        "websocket_bytes_received",
        "Number of websocket bytes received from a given connection",
    )
    .unwrap();
    pub static ref OCPP_MESSAGE_BYTES_SENT: IntCounter = register_int_counter!(
        "ocpp_message_bytes_sent",
        "Number of websocket bytes sent from a given connection",
    )
    .unwrap();
    pub static ref OCPP_MESSAGE_BYTES_RECEIVED: IntCounter = register_int_counter!(
        "ocpp_message_bytes_received",
        "Number of websocket bytes received from a given connection",
    )
    .unwrap();
}

pub async fn start_http_server() {
    Server::run(
        prometheus::default_registry(),
        SocketAddr::from(([0; 4], PROMETHEUS_PORT)),
        futures_util::future::pending(),
    )
    .await
    .unwrap()
}
