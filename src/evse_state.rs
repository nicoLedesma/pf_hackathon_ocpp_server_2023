use chrono::{DateTime, Utc};
use rust_ocpp::v1_6::types::{ChargePointErrorCode, ChargePointStatus};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, PartialEq, Clone)]
pub struct EvseMetadata {
    pub id: Uuid,
    pub charge_point_model: String,
    pub charge_point_serial_number: Option<String>,
    pub charge_point_vendor: String,
    pub firmware_version: Option<String>,
    pub iccid: Option<String>,
    pub imsi: Option<String>,
    pub boot_time: DateTime<Utc>,
    pub last_heartbeat_time: DateTime<Utc>,
    pub connector_info: HashMap<u64, ConnectorInfo>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ConnectorInfo {
    pub connector_id: u64,
    pub status: ChargePointStatus,
    pub error_code: ChargePointErrorCode,
    pub timestamp: Option<DateTime<Utc>>,
    pub vendor_id: Option<String>,
    pub vendor_error_code: Option<String>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum EvseState {
    WebsocketConnected(Box<EvseMetadata>),
    Empty,
}

impl EvseMetadata {
    pub fn new(
        id: Uuid,
        charge_point_vendor: String,
        charge_point_model: String,
        charge_point_serial_number: Option<String>,
        firmware_version: Option<String>,
        iccid: Option<String>,
        imsi: Option<String>,
    ) -> Self {
        EvseMetadata {
            id,
            charge_point_vendor,
            charge_point_model,
            charge_point_serial_number,
            firmware_version,
            iccid,
            imsi,
            boot_time: Utc::now(),
            last_heartbeat_time: Utc::now(),
            connector_info: HashMap::new(),
        }
    }

    pub fn update_info(&mut self, connector_info: ConnectorInfo) {
        self.connector_info
            .insert(connector_info.connector_id, connector_info);
    }
}
