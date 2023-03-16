use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "0")]
enum OcppMessage {
    Call {
        #[serde(rename = "1")]
        unique_id: String,
        #[serde(rename = "2")]
        action: Action,
        #[serde(rename = "3")]
        payload: CallPayload,
    },
    CallResult {
        #[serde(rename = "1")]
        unique_id: String,
        #[serde(rename = "2")]
        payload: CallResultPayload,
    },
    CallError {
        #[serde(rename = "1")]
        unique_id: String,
        #[serde(rename = "2")]
        error_code: String,
        #[serde(rename = "3")]
        error_description: String,
        #[serde(rename = "4")]
        error_details: Option<Value>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
enum Action {
    BootNotification,
    StatusNotification,
    // Add more actions here as needed
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum CallPayload {
    BootNotification(BootNotificationPayload),
    StatusNotification(StatusNotificationPayload),
    // Add more payload types as needed
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum CallResultPayload {
    BootNotification(BootNotificationResponse),
    StatusNotification(StatusNotificationResponse),
}

#[derive(Debug, Serialize, Deserialize)]
struct BootNotificationResponse {
    status: String,
    current_time: DateTime<Utc>,
    interval: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct BootNotificationPayload {
    #[serde(rename = "chargePointModel")]
    charge_point_model: String,
    #[serde(rename = "chargePointSerialNumber")]
    charge_point_serial_number: String,
    #[serde(rename = "chargePointVendor")]
    charge_point_vendor: String,
    #[serde(rename = "firmwareVersion")]
    firmware_version: String,
    iccid: Option<String>,
    imsi: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct StatusNotificationPayload {
    #[serde(rename = "connectorId")]
    connector_id: u64,
    #[serde(rename = "errorCode")]
    error_code: String,
    #[serde(rename = "status")]
    status: String,
    #[serde(rename = "timestamp")]
    timestamp: Option<DateTime<Utc>>,
    #[serde(rename = "info")]
    info: Option<String>,
    #[serde(rename = "vendorId")]
    vendor_id: Option<String>,
    #[serde(rename = "vendorErrorCode")]
    vendor_error_code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct StatusNotificationResponse;
/*
fn test() {
    let json_data = r#"[2,"4b05fb33-6510-445e-b7c4-d3a6c611400e","BootNotification",{"chargePointModel":"TRI93-50-01","chargePointVendor":"Tritium","chargePointSerialNumber":"12336","firmwareVersion":"v2.3.2","iccid":"89014103270749598363","imsi":"310410074959836"}]"#;

    // Deserialize the JSON data
    let message: OcppMessage = serde_json::from_str(json_data).expect("Failed to deserialize OCPP message");

    println!("Parsed message: {:?}", message);
}
*/
#[derive(Debug, Clone)]
pub struct EvseState {
    pub id: Uuid,
    pub charge_point_model: String,
    pub charge_point_serial_number: String,
    pub charge_point_vendor: String,
    pub firmware_version: String,
    pub iccid: Option<String>,
    pub imsi: Option<String>,
    pub boot_time: DateTime<Utc>,
    pub last_heartbeat_time: DateTime<Utc>,
    pub status: HashMap<u32, String>,
}

#[derive(Debug, Clone)]
pub struct ConnectorStatus {
    pub connector_id: u32,
    pub status: String,
    pub error_code: String,
    pub timestamp: Option<DateTime<Utc>>,
    pub vendor_id: Option<String>,
    pub vendor_error_code: Option<String>,
}

#[derive(Debug, Clone)]
pub enum EvseStateOption {
    Valid(EvseState),
    Empty,
}

impl EvseState {
    pub fn new(
        id: Uuid,
        charge_point_vendor: &str,
        charge_point_model: &str,
        charge_point_serial_number: &str,
        firmware_version: &str,
        iccid: Option<&str>,
        imsi: Option<&str>,
    ) -> Self {
        EvseState {
            id,
            charge_point_vendor: charge_point_vendor.to_string(),
            charge_point_model: charge_point_model.to_string(),
            charge_point_serial_number: charge_point_serial_number.to_string(),
            firmware_version: firmware_version.to_string(),
            iccid: iccid.map(String::from),
            imsi: imsi.map(String::from),
            boot_time: Utc::now(),
            last_heartbeat_time: Utc::now(),
            status: HashMap::new(),
        }
    }

    pub fn update_status(&mut self, connector_status: ConnectorStatus) {
        self.status
            .insert(connector_status.connector_id, connector_status);
    }
}

trait MessageHandler {
    type Response;
    fn handle_message(&self, evse_state: &mut EvseStateOption) -> Self::Response;
}

impl MessageHandler for StatusNotificationPayload {
    type Response = StatusNotificationResponse;

    fn handle_message(&self, evse_state: &mut EvseStateOption) -> Self::Response {
        // Handle the StatusNotification message and return an empty response
        // You can add your own logic here to process the StatusNotification message
        println!("Handling StatusNotification message: {:?}", self);

        StatusNotificationResponse
    }
}

impl MessageHandler for BootNotificationPayload {
    type Response = BootNotificationResponse;

    fn handle_message(&self, evse_state: &mut EvseStateOption) -> Self::Response {
        // Handle the BootNotification message and return the response
        println!("Handling BootNotification message: {:?}", self);

        let response = BootNotificationResponse {
            status: "Accepted".to_string(),
            current_time: Utc::now(),
            interval: 300,
        };

        *evse_state = EvseStateOption::Valid(EvseState::new(
            Uuid::new_v4(),
            &self.charge_point_vendor,
            &self.charge_point_model,
            &self.charge_point_serial_number,
            &self.firmware_version,
            // TODO bad
            Some(self.iccid.unwrap().as_str()),
            Some(self.imsi.unwrap().as_str()),
        ));

        response
    }
}

fn handle_ocpp_call(
    message: &OcppMessage,
    evse_state: &mut EvseStateOption,
) -> Result<CallResultPayload> {
    match message {
        OcppMessage::Call {
            unique_id,
            action,
            payload,
        } => match action {
            Action::BootNotification => {
                let boot_notification_payload: BootNotificationPayload =
                    serde_json::from_value(payload)
                        .map_err(|_| anyhow!("Failed to deserialize BootNotificationPayload"))?;
                Ok(CallResultPayload::BootNotification(
                    boot_notification_payload.handle_message(evse_state),
                ))
            }
            Action::StatusNotification => {
                let status_notification_payload: StatusNotificationPayload =
                    serde_json::from_value(payload)
                        .map_err(|_| anyhow!("Failed to deserialize StatusNotificationPayload"))?;
                Ok(CallResultPayload::StatusNotification(
                    status_notification_payload.handle_message(evse_state),
                ))
            }
            // Add more actions and their corresponding handling here
            _ => Err(anyhow!("Unsupported action: {:?}", action)),
        },
        _ => Err(anyhow!("handle_ocpp_call function expects a Call message")),
    }
}

pub async fn ocpp_process_and_respond(
    message: String,
    evse_state: &mut EvseStateOption,
) -> Result<String, Box<dyn std::error::Error>> {
    // Deserialize the message string into an OcppMessage
    let ocpp_message: OcppMessage = serde_json::from_str(&message)?;

    // Process the OCPP message and get the response payload
    let call_result_payload = handle_ocpp_call(&ocpp_message, evse_state)?;

    // Get the unique_id from the OcppMessage::Call
    let unique_id = match ocpp_message {
        OcppMessage::Call { unique_id, .. } => unique_id,
        _ => return Err(anyhow!("Expected a Call message").into()),
    };

    // Create the CallResult message
    let call_result = OcppMessage::CallResult {
        unique_id,
        payload: call_result_payload,
    };

    // Serialize the CallResult message into a JSON string
    let response = serde_json::to_string(&call_result)?;

    Ok(response)
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_handle_boot_notification() {
        let mut evse_state = EvseStateOption::Empty;
        let message = OcppMessage::Call {
            unique_id: "test-uuid".to_string(),
            action: Action::BootNotification,
            payload: CallResultPayload::BootNotification {
                charge_point_vendor: "Tritium".to_string(),
                charge_point_model: "TRI93-50-01".to_string(),
                charge_point_serial_number: "12336".to_string(),
                firmware_version: Some("v2.3.2".to_string()),
            },
        };
        let result = handle_ocpp_call(&message, &mut evse_state);
        assert!(result.is_ok(), "Expected Ok result from handle_ocpp_call");

        if let EvseStateOption::Valid(state) = &evse_state {
            assert_eq!(state.charge_point_vendor, "Tritium");
            assert_eq!(state.charge_point_model, "TRI93-50-01");
            assert_eq!(state.charge_point_serial_number, "12336");
            assert_eq!(state.firmware_version, "v2.3.2".to_string());
        } else {
            panic!("Expected evse_state to be Connected");
        }
    }

    #[test]
    fn test_handle_status_notification() {
        let mut initial_status = HashMap::new();
        initial_status.insert(1, "Available".into());
        let mut evse_state = EvseStateOption::Valid(EvseState {
            id: Uuid::new_v4(),
            iccid: None,
            imsi: None,
            boot_time: Utc::now(),
            last_heartbeat_time:Utc::now(),

            charge_point_vendor: "Tritium".to_string(),
            charge_point_model: "TRI93-50-01".to_string(),
            charge_point_serial_number: "12336".to_string(),
            firmware_version: "v2.3.2".to_string(),
            status: initial_status,
        });

        let message = OcppMessage::Call {
            unique_id: "test-uuid".to_string(),
            action: Action::StatusNotification,
            payload: CallPayload::StatusNotification {
                connector_id: 1,
                status: ConnectorStatus::Occupied,
                error_code: ErrorCode::NoError,
                timestamp: None,
                vendor_id: None,
                vendor_error_code: None,
            },
        };
        let result = handle_ocpp_call(&message, &mut evse_state);
        assert!(result.is_ok(), "Expected Ok result from handle_ocpp_call");

        if let EvseStateOption::Valid(state) = &evse_state {
            let expected_status = HashMap::from([(1, "Occupied".into())]);
            assert_eq!(state.status, expected_status);
        } else {
            panic!("Expected evse_state to be Connected");
        }
    }
}
