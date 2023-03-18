use crate::normalize_input::{normalize_json_input, };
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use rust_ocpp::v1_6::messages::boot_notification::{
    BootNotificationRequest, BootNotificationResponse,
};
use rust_ocpp::v1_6::messages::status_notification::{
    StatusNotificationRequest, StatusNotificationResponse,
};
use rust_ocpp::v1_6::types::{ChargePointErrorCode, ChargePointStatus, RegistrationStatus};
use serde::{ser::SerializeSeq, Deserialize, Serialize, Serializer};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

// Message Type
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(into = "u8")]
pub enum OcppMessageType {
    Call = 2,
    CallResult = 3,
    CallError = 4,
}

impl From<OcppMessageType> for u8 {
    fn from(msg_type: OcppMessageType) -> Self {
        msg_type as u8
    }
}

// Use custom Serialize to create an array with the first element being an int
#[derive(Debug, PartialEq, Deserialize)]
#[serde(tag = "0")]
pub enum OcppMessage {
    #[serde(rename = "2")]
    Call {
        #[serde(rename = "1")]
        unique_id: String,
        #[serde(rename = "2")]
        // TODO parse as String if it is unknown
        action: Action,
        #[serde(rename = "3")]
        payload: CallPayload,
    },
    #[serde(rename = "3")]
    CallResult {
        #[serde(rename = "1")]
        unique_id: String,
        #[serde(rename = "2")]
        payload: CallResultPayload,
    },
    #[serde(rename = "4")]
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

pub fn message_type_of(message: &OcppMessage) -> OcppMessageType {
    match message {
        OcppMessage::Call { .. } => OcppMessageType::Call,
        OcppMessage::CallResult { .. } => OcppMessageType::CallResult,
        OcppMessage::CallError { .. } => OcppMessageType::CallError,
    }
}

impl Serialize for OcppMessage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            OcppMessage::Call {
                unique_id,
                action,
                payload,
            } => {
                let mut seq = serializer.serialize_seq(Some(4))?;
                seq.serialize_element(&message_type_of(self))?;
                seq.serialize_element(unique_id)?;
                seq.serialize_element(action)?;
                seq.serialize_element(payload)?;
                seq.end()
            }
            OcppMessage::CallResult { unique_id, payload } => {
                let mut seq = serializer.serialize_seq(Some(3))?;
                seq.serialize_element(&message_type_of(&self))?;
                seq.serialize_element(unique_id)?;
                seq.serialize_element(payload)?;
                seq.end()
            }
            OcppMessage::CallError {
                unique_id,
                error_code,
                error_description,
                error_details,
            } => {
                let mut seq = serializer.serialize_seq(Some(5))?;
                seq.serialize_element(&message_type_of(&self))?;
                seq.serialize_element(unique_id)?;
                seq.serialize_element(error_code)?;
                seq.serialize_element(error_description)?;
                seq.serialize_element(error_details)?;
                seq.end()
            }
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Action {
    BootNotification,
    StatusNotification,
    // Add more actions here as needed
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CallPayload {
    BootNotification(BootNotificationRequest),
    StatusNotification(StatusNotificationRequest),
    // Add more payload types as needed
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CallResultPayload {
    BootNotification(BootNotificationResponse),
    StatusNotification(StatusNotificationResponse),
}

impl TryInto<OcppMessage> for String {
    type Error = serde_json::error::Error;

    fn try_into(self) -> Result<OcppMessage, Self::Error> {
        serde_json::from_str(self.as_str())
    }
}

impl TryInto<OcppMessage> for &str {
    type Error = serde_json::error::Error;

    fn try_into(self) -> Result<OcppMessage, Self::Error> {
        serde_json::from_str(self)
    }
}

impl TryInto<OcppMessage> for Value {
    type Error = serde_json::error::Error;

    fn try_into(self) -> Result<OcppMessage, Self::Error> {
        serde_json::from_value(self)
    }
}

impl TryInto<String> for OcppMessage {
    type Error = serde_json::error::Error;

    fn try_into(self) -> Result<String, Self::Error> {
        serde_json::to_string(&self)
    }
}

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
    WebsocketConnected(EvseMetadata),
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

trait InfallibleMessageHandler {
    type CallResult;
    fn handle_message(self, evse_state: &mut EvseState) -> Self::CallResult;
}

impl InfallibleMessageHandler for BootNotificationRequest {
    type CallResult = BootNotificationResponse;

    fn handle_message(self, evse_state: &mut EvseState) -> Self::CallResult {
        // Handle the BootNotification message and return the response
        println!("Handling BootNotification message: {:?}", self);

        let response = BootNotificationResponse {
            status: RegistrationStatus::Accepted,
            current_time: Utc::now(),
            interval: 300,
        };

        *evse_state = EvseState::WebsocketConnected(EvseMetadata::new(
            Uuid::new_v4(),
            self.charge_point_vendor,
            self.charge_point_model,
            self.charge_point_serial_number,
            self.firmware_version,
            self.iccid,
            self.imsi,
        ));

        response
    }
}

impl InfallibleMessageHandler for StatusNotificationRequest {
    type CallResult = StatusNotificationResponse;

    fn handle_message(self, evse_state: &mut EvseState) -> Self::CallResult {
        // Handle the StatusNotification message and return the response
        println!("Handling StatusNotification message: {:?}", self);

        match evse_state {
            EvseState::WebsocketConnected(metadata) => metadata.update_info(ConnectorInfo {
                connector_id: self.connector_id,
                status: self.status,
                error_code: self.error_code,
                vendor_error_code: self.vendor_error_code,
                vendor_id: self.vendor_id,
                timestamp: self.timestamp,
            }),
            EvseState::Empty => println!("StatusNotification but no metadata???"),
        }

        StatusNotificationResponse {}
    }
}

fn ocpp_process_and_respond(
    message: OcppMessage,
    evse_state: &mut EvseState,
) -> Result<OcppMessage> {
    match message {
        OcppMessage::Call {
            unique_id,
            action,
            payload,
        } => match (action, payload) {
            (Action::BootNotification, CallPayload::BootNotification(call)) => {
                Ok(OcppMessage::CallResult {
                    unique_id,
                    payload: CallResultPayload::BootNotification(call.handle_message(evse_state)),
                })
            }
            (Action::StatusNotification, CallPayload::StatusNotification(call)) => {
                Ok(OcppMessage::CallResult {
                    unique_id,
                    payload: CallResultPayload::StatusNotification(call.handle_message(evse_state)),
                })
            } // Add more actions and their corresponding handling here
            _ => Err(anyhow!(
                "handle_ocpp_call function expects the Action and CallPayload to be same type"
            )),
        },
        _ => Err(anyhow!("handle_ocpp_call function expects a Call message")),
    }
}

pub fn parse_ocpp_message(message: String) -> Result<OcppMessage> {
    let parsed_json = normalize_json_input(message.as_str())?;
    Ok(parsed_json.try_into()?)
}

pub fn ocpp_process_and_respond_str(message: String, evse_state: &mut EvseState) -> Result<String> {
    // Deserialize the message string into an OcppMessage
    let ocpp_message = parse_ocpp_message(message)?;
    let response = ocpp_process_and_respond(ocpp_message, evse_state)?;

    println!("Will send response {:?}", response);

    // How to convert serde error into anyhow error without unwrapping and re-wrapping?
    Ok(response.try_into()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::normalize_input::normalize_json_input_datetimes;
    use pretty_assertions::assert_eq;

    // A fixed UUID for unit testing
    const UNIQUE_ID: &str = "4b05fb33-6510-445e-b7c4-d3a6c611400e";
    // A fixed charge_point_serial_number for unit testing
    const SERIAL_NUMBER: &str = "123456";
    // Make sure the JSON keys are in the same order as in the Rust structs
    const BOOT_NOTIFICATION_CALL: &str = r#"[2,"4b05fb33-6510-445e-b7c4-d3a6c611400e","BootNotification",{"chargePointModel":"TRI93-50-01","chargePointSerialNumber":"123456","chargePointVendor":"Tritium","firmwareVersion":"v2.3.2","iccid":"89014103270749598363","imsi":"310410074959836"}]"#;
    const STATUS_NOTIFICATION_CALL_WITH_MISSING_FIELDS: &str = r#"[2,"4b05fb33-6510-445e-b7c4-d3a6c611400e","StatusNotification",{"connectorId":1,"errorCode":"NoError","status":"Preparing","timestamp":"2023-03-17T22:42:50.008427"}]"#;

    #[test]
    fn test_parse_boot_notification() {
        let message =
            parse_ocpp_message(BOOT_NOTIFICATION_CALL.into()).expect("Could not parse test JSON");

        match message {
            OcppMessage::Call {
                unique_id,
                action,
                payload:
                    CallPayload::BootNotification(BootNotificationRequest {
                        charge_point_serial_number,
                        ..
                    }),
            } => {
                assert_eq!(action, Action::BootNotification);
                assert_eq!(unique_id, UNIQUE_ID);
                assert_eq!(charge_point_serial_number, Some(SERIAL_NUMBER.to_string()));
            }
            _ => panic!("Expected BootNotification Call with BootNotification CallPayload"),
        }
    }

    #[test]
    fn test_handle_boot_notification() {
        let message =
            parse_ocpp_message(BOOT_NOTIFICATION_CALL.into()).expect("Could not parse test JSON");

        let mut state = EvseState::Empty;
        let response = ocpp_process_and_respond(message, &mut state)
            .expect("Could not process BootNotification");

        match response {
            OcppMessage::CallResult {
                unique_id,
                payload: CallResultPayload::BootNotification(_),
            } => {
                assert_eq!(unique_id, UNIQUE_ID);
            }
            _ => panic!("Expected CallResult with BootNotification CallResultPayload"),
        }
    }

    #[test]
    fn test_serde_boot_notification_call() {
        let message =
            parse_ocpp_message(BOOT_NOTIFICATION_CALL.into()).expect("Could not parse test JSON");

        let json: String = message.try_into().expect("nope");
        assert_eq!(json, BOOT_NOTIFICATION_CALL);
    }

    #[test]
    fn test_serde_status_notification_call() {
        let message = parse_ocpp_message(STATUS_NOTIFICATION_CALL_WITH_MISSING_FIELDS.into())
            .expect("Could not parse test JSON");

        let json: String = message.try_into().expect("nope");
        assert_eq!(
            json,
            normalize_json_input_datetimes(STATUS_NOTIFICATION_CALL_WITH_MISSING_FIELDS)
                .expect("Could not normalize JSON")
                .to_string()
        );
    }

    #[test]
    fn test_serde_datetimes() {
        let _datetime: DateTime<Utc> =
            serde_json::from_str(r#""2023-03-17T22:42:50.008427Z""#).unwrap();
        let _datetime: DateTime<Utc> = serde_json::from_str(r#""2023-03-17T22:42:50Z""#).unwrap();
    }

    #[test]
    fn test_bad_serde_datetimes() {
        let datetime: Result<DateTime<Utc>, serde_json::Error> =
            serde_json::from_str(r#""2023-03-17T22:42""#);
        assert!(datetime.is_err());
        assert!(datetime.as_ref().unwrap_err().is_data());
        assert!(datetime
            .as_ref()
            .unwrap_err()
            .to_string()
            .contains("premature end of input"));
    }

    #[test]
    fn test_serde_common_constants() {
        let _error_code: ChargePointErrorCode = serde_json::from_str(r#""NoError""#).unwrap();
        let _error_code: ChargePointErrorCode = serde_json::from_str(r#""ResetFailure""#).unwrap();
        let _status: ChargePointStatus = serde_json::from_str(r#""Available""#).unwrap();
        let _status: ChargePointStatus = serde_json::from_str(r#""Charging""#).unwrap();
    }
}
