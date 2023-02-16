use rocket::{http::Status, response::status};
use serde_json::Value as JsonValue;

pub fn check_for_error(
    data: &JsonValue,
) -> Result<(), rocket::response::status::Custom<std::string::String>> {
    match data {
        JsonValue::Object(map) if map.get("success") == Some(&JsonValue::Bool(true)) => Ok(()),
        JsonValue::Array(v) => {
            for item in v.iter() {
                if item.as_object().map(|obj| obj.get("success")).flatten()
                    != Some(&JsonValue::Bool(true))
                {
                    return Err(status::Custom(
                        Status::FailedDependency,
                        format!("{}", data),
                    ));
                }
            }
            Ok(())
        }
        _ => Err(status::Custom(
            Status::FailedDependency,
            format!("{}", data),
        )),
    }
}
