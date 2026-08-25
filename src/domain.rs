//! Pure protocol transformations used by the client boundary.
//!
//! These functions deliberately do not read the clock, random state, files, or
//! network. Keeping them as values-in/values-out makes the protocol executable
//! in deterministic tests.

use serde_json::{Map, Value};

pub(crate) fn merge_json_objects(base: Value, overlay: Value) -> Value {
    let mut result = match base {
        Value::Object(object) => object,
        _ => Map::new(),
    };

    if let Value::Object(overlay) = overlay {
        result.extend(overlay);
    }

    Value::Object(result)
}

pub(crate) fn insert_json_field(base: Value, key: String, value: Value) -> Value {
    merge_json_objects(base, Value::Object(Map::from_iter([(key, value)])))
}

pub(crate) fn request_id(timestamp_millis: u128, random_suffix: u16) -> String {
    format!("{timestamp_millis}_{random_suffix:04}")
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use serde_json::json;

    use super::{insert_json_field, merge_json_objects, request_id};

    #[test]
    fn overlay_wins_without_mutating_the_input_value() {
        let base = json!({"limit": 30, "offset": 0});
        assert_eq!(
            merge_json_objects(base, json!({"limit": 10})),
            json!({"limit": 10, "offset": 0})
        );
    }

    #[test]
    fn request_id_is_deterministic_at_the_functional_core() {
        assert_eq!(request_id(1_724_000_000_000, 7), "1724000000000_0007");
    }

    proptest! {
        #[test]
        fn inserting_a_field_preserves_its_value(key in "[a-z]{1,12}", value in any::<i64>()) {
            let merged = insert_json_field(json!({"existing": true}), key.clone(), json!(value));
            prop_assert_eq!(&merged[&key], &json!(value));
            prop_assert_eq!(&merged["existing"], &json!(true));
        }
    }
}
