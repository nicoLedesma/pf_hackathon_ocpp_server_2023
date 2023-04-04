use crate::normalize_input::normalize_json_input;
use anyhow::Result;
use rust_ocpp::v1_6::messages::authorize::{AuthorizeRequest, AuthorizeResponse};
use rust_ocpp::v1_6::messages::boot_notification::{
    BootNotificationRequest, BootNotificationResponse,
};
use rust_ocpp::v1_6::messages::heart_beat::{HeartbeatRequest, HeartbeatResponse};
use rust_ocpp::v1_6::messages::meter_values::{MeterValuesRequest, MeterValuesResponse};
use rust_ocpp::v1_6::messages::status_notification::{
    StatusNotificationRequest, StatusNotificationResponse,
};
use serde::{ser::SerializeSeq, Deserialize, Serialize, Serializer};
use serde_json::Value;

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
                seq.serialize_element(&message_type_of(self))?;
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
                seq.serialize_element(&message_type_of(self))?;
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
    Heartbeat,
    MeterValues,
    Authorize,
    // Add more actions here as needed
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CallPayload {
    BootNotification(BootNotificationRequest),
    StatusNotification(StatusNotificationRequest),
    MeterValues(MeterValuesRequest),
    Authorize(AuthorizeRequest),
    // Heartbeats should go last since it's an empty struct and can cause problems with serde's untagged
    // parser
    Heartbeat(HeartbeatRequest),
    // Add more payload types as needed
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CallResultPayload {
    BootNotification(BootNotificationResponse),
    StatusNotification(StatusNotificationResponse),
    MeterValues(MeterValuesResponse),
    Authorize(AuthorizeResponse),
    // Heartbeats should go last since it's an empty struct and can cause problems with serde's untagged
    // parser
    Heartbeat(HeartbeatResponse),
    // Add more payload types as needed
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

pub fn parse_ocpp_message(message: String) -> Result<OcppMessage> {
    let parsed_json = normalize_json_input(message.as_str())?;
    Ok(parsed_json.try_into()?)
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

    #[cfg(test)]
    mod test_serde_parsing {
        use chrono::{DateTime, Utc};
        use rust_ocpp::v1_6::types::{ChargePointErrorCode, ChargePointStatus};
        #[test]
        fn test_serde_datetimes() {
            let _datetime: DateTime<Utc> =
                serde_json::from_str(r#""2023-03-17T22:42:50.008427Z""#).unwrap();
            let _datetime: DateTime<Utc> =
                serde_json::from_str(r#""2023-03-17T22:42:50Z""#).unwrap();
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
            let _error_code: ChargePointErrorCode =
                serde_json::from_str(r#""ResetFailure""#).unwrap();
            let _status: ChargePointStatus = serde_json::from_str(r#""Available""#).unwrap();
            let _status: ChargePointStatus = serde_json::from_str(r#""Charging""#).unwrap();
        }
    }
}
