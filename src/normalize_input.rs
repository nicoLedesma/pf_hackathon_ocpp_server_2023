use anyhow::Result;
use serde_json::Value;

pub fn fix_payload_timestamps(payload: &mut Value) -> Result<()> {
    if let Value::Object(ref mut obj) = payload {
        match obj.get("timestamp") {
            Some(ref value) => {
                if let Some(s) = value.as_str() {
                    if !s.ends_with('Z') && !s.ends_with('z') {
                        obj.insert("timestamp".into(), Value::String(format!("{}Z", &s)));
                    }
                }
            }
            None => {}
        }
    }
    Ok(())
}
pub fn normalize_json_input_datetimes(input: &str) -> Result<Value> {
    let mut json_value: Value = serde_json::from_str(input)?;

    if let Value::Array(ref mut arr) = json_value {
        if arr.len() > 3 {
            fix_payload_timestamps(&mut arr[3])?;
        }
    }
    Ok(json_value)
}

// [2, ...] => ["2", ...]
pub fn normalize_json_input(input: &str) -> Result<Value> {
    let mut json_value = normalize_json_input_datetimes(input)?;

    if let Value::Array(ref mut arr) = json_value {
        if let Some(Value::Number(num)) = arr.get(0) {
            let message_type = num.to_string();
            arr[0] = Value::String(message_type);
        }
    }

    Ok(json_value)
}
#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_process_converts_first_to_str() {
        const JSON: &str =
            r#"[2,"","",{"connectorId":1,"timestamp":"2023-03-17T22:42:50.008427"}]"#;
        let result: Value = normalize_json_input(JSON).unwrap();
        assert!(result.is_array());
        assert_eq!(
            result.as_array().unwrap().get(0),
            Some(&Value::String("2".into()))
        );
    }

    #[test]
    fn test_process_payload_with_bad_timestamp() {
        const JSON: &str = r#"[2,"","",{"connectorId":1,"timestamp":"2023"}]"#;
        let result: Value = normalize_json_input(JSON).unwrap();
        assert!(result.is_array());
        let payload = result.as_array().unwrap().get(3).unwrap();
        assert!(payload.is_object());
        assert_eq!(
            payload.as_object().unwrap().get("timestamp"),
            Some(&Value::String("2023Z".into()))
        );
    }
}
