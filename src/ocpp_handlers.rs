use crate::evse_state::{ConnectorInfo, EvseMetadata, EvseState};
use crate::ocpp::{parse_ocpp_message, Action, CallPayload, CallResultPayload, OcppMessage};
use anyhow::{anyhow, Result};
use chrono::Utc;
use rust_ocpp::v1_6::messages::boot_notification::{
    BootNotificationRequest, BootNotificationResponse,
};
use rust_ocpp::v1_6::messages::heart_beat::{HeartbeatRequest, HeartbeatResponse};
use rust_ocpp::v1_6::messages::meter_values::{MeterValuesRequest, MeterValuesResponse};
use rust_ocpp::v1_6::messages::status_notification::{
    StatusNotificationRequest, StatusNotificationResponse,
};
use rust_ocpp::v1_6::types::RegistrationStatus;
use uuid::Uuid;

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

impl InfallibleMessageHandler for HeartbeatRequest {
    type CallResult = HeartbeatResponse;

    fn handle_message(self, evse_state: &mut EvseState) -> Self::CallResult {
        // Handle the Heartbeat message and return the response
        println!("Handling Heartbeat message: {:?}", self);

        HeartbeatResponse {
            current_time: Utc::now(),
        }
    }
}

impl InfallibleMessageHandler for MeterValuesRequest {
    type CallResult = MeterValuesResponse;

    fn handle_message(self, evse_state: &mut EvseState) -> Self::CallResult {
        // Handle the MeterValues message and return the response
        println!("Handling MeterValues message: {:?}", self);

        MeterValuesResponse {
        }
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
            }
            (Action::Heartbeat, CallPayload::Heartbeat(call)) => Ok(OcppMessage::CallResult {
                unique_id,
                payload: CallResultPayload::Heartbeat(call.handle_message(evse_state)),
            }),
            (Action::MeterValues, CallPayload::MeterValues(call)) => Ok(OcppMessage::CallResult {
                unique_id,
                payload: CallResultPayload::MeterValues(call.handle_message(evse_state)),
            }),
            _ => Err(anyhow!(
                "handle_ocpp_call function expects the Action and CallPayload to be same type"
            )),
        },
        _ => Err(anyhow!("handle_ocpp_call function expects a Call message")),
    }
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
    use pretty_assertions::assert_eq;
    // A fixed UUID for unit testing
    const UNIQUE_ID: &str = "4b05fb33-6510-445e-b7c4-d3a6c611400e";
    // Make sure the JSON keys are in the same order as in the Rust structs
    const BOOT_NOTIFICATION_CALL: &str = r#"[2,"4b05fb33-6510-445e-b7c4-d3a6c611400e","BootNotification",{"chargePointModel":"TRI93-50-01","chargePointSerialNumber":"123456","chargePointVendor":"Tritium","firmwareVersion":"v2.3.2","iccid":"89014103270749598363","imsi":"310410074959836"}]"#;
    const STATUS_NOTIFICATION_CALL_WITH_MISSING_FIELDS: &str = r#"[2,"4b05fb33-6510-445e-b7c4-d3a6c611400e","StatusNotification",{"connectorId":1,"errorCode":"NoError","status":"Preparing","timestamp":"2023-03-17T22:42:50.008427"}]"#;
    const HEARTBEAT_CALL: &str = r#"[2,"4b05fb33-6510-445e-b7c4-d3a6c611400e","Heartbeat",{}]"#;

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
    fn test_handle_status_notification() {
        let message = parse_ocpp_message(STATUS_NOTIFICATION_CALL_WITH_MISSING_FIELDS.into())
            .expect("Could not parse test JSON");

        let mut state = EvseState::Empty;
        let response = ocpp_process_and_respond(message, &mut state)
            .expect("Could not process StatusNotification");

        match response {
            OcppMessage::CallResult {
                unique_id,
                payload: CallResultPayload::StatusNotification(_),
            } => {
                assert_eq!(unique_id, UNIQUE_ID);
            }
            _ => panic!("Expected CallResult with StatusNotification CallResultPayload"),
        }
    }

    #[test]
    fn test_handle_heartbeat() {
        let message = parse_ocpp_message(HEARTBEAT_CALL.into()).expect("Could not parse test JSON");

        let mut state = EvseState::Empty;
        let response =
            ocpp_process_and_respond(message, &mut state).expect("Could not process Heartbeat");

        match response {
            OcppMessage::CallResult {
                unique_id,
                payload: CallResultPayload::Heartbeat(_),
            } => {
                assert_eq!(unique_id, UNIQUE_ID);
            }
            _ => panic!("Expected CallResult with Heartbeat CallResultPayload"),
        }
    }
}
