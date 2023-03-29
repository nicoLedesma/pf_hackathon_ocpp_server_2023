// TODO [prom] count how many websocket closes or errors, same as python
// TODO [prom]   metadata: version and binary size and tls protocol version
// TODO [prom] measure tls handshake time
// TODO [prom] measure websocket handshake time
// TODO [prom] measure websocket latency
// TODO [prom] measure websocket bandwidth, with and without compression
use lazy_static::lazy_static;
use prometheus::{self, register_gauge_vec, register_int_counter_vec, GaugeVec, IntCounterVec};
use prometheus_hyper::Server;
use std::net::SocketAddr;

const PROMETHEUS_PORT: u16 = 8080;

// TODO add labels lol
lazy_static! {
    pub static ref WEBSOCKET_BYTES_RECEIVED: IntCounterVec = register_int_counter_vec!(
        "websocket_bytes_received",
        "Number of websocket bytes received from a given connection",
        &["serial_number"],
    )
    .unwrap();
    pub static ref OCPP_MESSAGE_BYTES_SENT: IntCounterVec = register_int_counter_vec!(
        "ocpp_message_bytes_sent",
        "Number of websocket bytes sent from a given connection",
        &["serial_number"],
    )
    .unwrap();
    pub static ref OCPP_MESSAGE_BYTES_RECEIVED: IntCounterVec = register_int_counter_vec!(
        "ocpp_message_bytes_received",
        "Number of websocket bytes received from a given connection",
        &["serial_number"],
    )
    .unwrap();
    pub static ref TLS_HANDSHAKE_TIME: GaugeVec =
        register_gauge_vec!("tls_handshake_time", "In seconds", &["peer_address"],).unwrap();
    pub static ref WEBSOCKET_HANDSHAKE_TIME: GaugeVec = register_gauge_vec!(
        "websocket_handshake_time",
        "In seconds",
        &["peer_address", "serial_number"],
    )
    .unwrap();
    pub static ref WEBSOCKET_PONG_TRANSMIT_TIME: GaugeVec = register_gauge_vec!(
        "websocket_pong_transmit_time",
        "In seconds",
        &["peer_addr", "serial_number"],
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
