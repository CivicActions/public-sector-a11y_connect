use rocket::{http::Status, response::status};
use serde_json::Value as JsonValue;

/*
Code Summary:
    This Rust code defines a function named check_for_error, which takes a reference to a JSON value as input and returns a Result. The function checks if the input JSON data contains an error by checking if the value of the key "success" is true or false. If an error is detected, the function returns a custom error message.

Variables:
    data:
        A reference to a JSON value that is passed as input to the check_for_error function.

Functions:
    check_for_error(&JsonValue) -> Result<(), rocket::response::status::Custom<std::string::String>>:
        This function takes a reference to a JSON value as input and returns a Result. It checks if the input JSON data contains an error by checking if the value of the key "success" is true or false. If an error is detected, the function returns a custom error message.


Docker Vars:



Output:
    Result<(), rocket::response::status::Custom<std::string::String>>:
        This is the output of the check_for_error function. It returns an empty Ok value if the input JSON data does not contain an error. Otherwise, it returns a custom error message in the Err variant of the Result.

Errors:
    status::Custom(Status::FailedDependency, format!("{}", data)):
        This is a custom error message that is returned if an error is detected in the input JSON data. The error message contains the input JSON data that was checked.

*/

// Define a function check_for_error that takes a JSON value as input
// and returns a Result<(), Custom<String>> where () is an empty tuple
// and the error message is a custom status with a string payload
pub fn check_for_error(
    data: &JsonValue,
) -> Result<(), rocket::response::status::Custom<std::string::String>> {
    // Match on the JSON value to check for errors
    match data {
        // If the JSON value is an object with a "success" key set to true, return Ok(())
        JsonValue::Object(map) if map.get("success") == Some(&JsonValue::Bool(true)) => Ok(()),

        // If the JSON value is an array, check the "success" key for each item
        JsonValue::Array(v) => {
            for item in v.iter() {
                if item.as_object().map(|obj| obj.get("success")).flatten()
                    != Some(&JsonValue::Bool(true))
                {
                    // If any item does not have "success" set to true, return an error
                    return Err(status::Custom(
                        Status::FailedDependency,
                        format!("{}", data),
                    ));
                }
            }
            Ok(())
        }

        // If the JSON value is neither an object nor an array with the "success" key, return an error
        _ => Err(status::Custom(
            Status::FailedDependency,
            format!("{}", data),
        )),
    }
}
