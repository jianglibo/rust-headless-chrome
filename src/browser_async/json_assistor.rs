use std::collections::HashSet;
use std::borrow::Borrow;
use crate::protocol::{self};
use log::*;

pub fn json_has_properties(json_value: &serde_json::Value, field_names: Vec<&str>) -> bool {
    if let serde_json::Value::Object(mp) = json_value {
        let keys: HashSet<_> = mp.borrow().keys().map(|s|&s[..]).collect();
        let names: HashSet<_> = field_names.into_iter().collect();
        keys.is_superset(&names)
    } else {
        error!("the value isn't an type of object. {:?}", json_value);
        false
    }
}

pub fn json_field_has_properties(json_value: &serde_json::Value, field_name: &str, field_names: Vec<&str>) -> bool {
    if let serde_json::Value::Object(mp) = json_value {
        if let Some(serde_json::Value::Object(root_map)) = mp.get(field_name) {
            json_has_properties(json_value, field_names)
        } else {
            error!("json object's {} field isn't an object. {:?}", field_name, json_value);
            false
        }
    } else {
        error!("the value isn't an type of object. {:?}", json_value);
        false
    }
}

pub fn response_result_field_has_properties(resp: &protocol::Response, field_name: &str, field_names: Vec<&str>) -> bool {
        if let Some(result_value) = &resp.result {
            if let Some(result_str) = result_value.as_str() {
                if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(result_str) {
                    return json_field_has_properties(&json_value, field_name, field_names)
                } else {
                    error!("parse {} field failed. {:?}",field_name, resp);
                }
            } else {
                error!("{} field isn't a string. {:?}", field_name, resp);
            }
        } else {
            error!("got response with no result field. {:?}", resp);
        }
        false
}