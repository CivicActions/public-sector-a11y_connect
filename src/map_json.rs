use itertools::Itertools;
use serde_json::Value;
use std::collections::HashMap;

/*
Overview/Code summary:
    This Rust code implements a JSON mapper that maps JSON data to a new structure based on a given mapping. It takes in a JSON object that represents the mapping and returns a JsonMapper instance, which can then be used to map other JSON data. The mapper uses a hashmap to store the mapping information and the serde_json crate for JSON serialization and deserialization.

Variables:
    keys: a HashMap that stores the mapping information, with source JSON paths as keys and target names as values.

Functions:

    new(mapping: Value):
        This function creates a new JsonMapper instance from the provided JSON object that represents the mapping. It reads the mapping data from the object and stores it in the keys hashmap.

    map(&self, input: &Value):
        This function maps the provided JSON input to the target structure based on the stored mapping information in keys.

    map_internal(&self, keys: &HashMap<Vec<String>, String>, input: &Value):
        This function is a recursive helper function that maps a JSON input to the target structure based on the provided keys hashmap.

Docker Vars:
    N/A

Output:
        The map function returns a Result with either the mapped JSON data or a JsonMapperError if an error occurs.

Error Messages:
    ParallelListMapping:
        Error occurs if the input data includes parallel lists, which cannot be mapped using the current implementation.

    ExpectedArrayOrObject:
        Error occurs if the provided JSON input is not an object or array when the mapping expects it to be one.

    MapInternalReturnedInvalidData:
        Error occurs when the map_internal function returns an invalid JSON data structure.

    InvalidInput:
        Error occurs when the input JSON data is invalid.

    Empty:
        Error occurs when the input data is empty.

*/

#[derive(Debug)]
pub enum JsonMapperError {
    ParallelListMapping,
    ExpectedArrayOrObject,
    MapInternalReturnedInvalidData,
    InvalidInput,
    Empty,
}

pub struct JsonMapper {
    keys: HashMap<Vec<String>, String>,
}
impl JsonMapper {
    /// Create a new `JsonMapper` instance with the specified `mapping`.
    pub fn new(mapping: Value) -> Self {
        let mut keys = HashMap::new();
        for (key_v, value_v) in mapping.as_object().unwrap().iter() {
            let target_name = key_v.as_str();
            let source_path: Vec<String> = value_v
                .as_str()
                .unwrap()
                .split('.')
                .map(|x| x.to_owned())
                .collect();
            keys.insert(source_path.to_owned(), target_name.to_owned());
        }
        JsonMapper { keys }
    }

    /// Maps the given `input` `serde_json::Value` to a new `serde_json::Value`
    /// according to the mapping configuration.
    ///
    /// # Arguments
    ///
    /// * `input` - A `serde_json::Value` object containing the input data to be mapped.
    ///
    /// # Returns
    ///
    /// A `serde_json::Value` object containing the mapped data.
    ///
    /// # Errors
    ///
    /// Returns an `JsonMapperError` if the mapping fails.

    /// Map the `input` JSON value to the target schema.
    pub fn map(&self, input: &Value) -> Result<Value, JsonMapperError> {
        self.map_internal(&self.keys, input)
            .map(|x| x.ok_or(JsonMapperError::Empty))
            .flatten()
    }

    fn map_internal(
        &self,
        keys: &HashMap<Vec<String>, String>,
        input: &Value,
    ) -> Result<Option<Value>, JsonMapperError> {
        // Check if there is no path left in the `keys` hash map
        let mut finished = true;
        for k in keys.keys() {
            if !k.is_empty() {
                finished = false;
                break;
            }
        }

        // If there is no path left, then we return a successful empty value.
        if finished {
            return Ok(None);
        }

        // Traverse the input JSON value recursively.
        match input {
            Value::Array(items) => {
                let mut new_items = Vec::new();

                // Map each item of the input array recursively.
                for item_opt_res in items.iter().map(|i| self.map_internal(keys, i)) {
                    let item_opt = item_opt_res?;
                    // unwrap: `keys` is unchanged for this recursive call, so if the call
                    // would return `Ok(None)`, we would have returned `Ok(None)` aswell
                    // before reaching this point.
                    let item = item_opt.unwrap();
                    match item {
                        Value::Array(items) => {
                            // If the item is also an array, append each item in it to the new_items list.
                            for item in items {
                                new_items.push(item);
                            }
                        }
                        Value::Object(obj) => {
                            // If the item is an object, just append it to the new_items list.
                            new_items.push(Value::Object(obj));
                        }
                        _ => return Err(JsonMapperError::ExpectedArrayOrObject),
                    }
                }

                Ok(Some(Value::Array(new_items)))
            }
            Value::Object(obj) => {
                let mut map = serde_json::Map::new();
                let mut merge_array = None;

                // For each unique top-level key in the `keys` hash map.
                for k in keys.keys().filter_map(|x| x.get(0)).unique() {
                    let mapped_name = keys.iter().find(|(k2, _v)| k2.get(0) == Some(k)).unwrap().1;
                    if let Some(mapping_value) = obj.get(k) {
                        let new_keys = keys
                            .iter()
                            .filter_map(|(k, v)| {
                                if k.is_empty() {
                                    None
                                } else {
                                    let mut new_k = k.clone();
                                    new_k.remove(0);
                                    Some((new_k, v.to_owned()))
                                }
                            })
                            .collect();
                        match mapping_value {
                            Value::Object(_) | Value::Array(_) => {}
                            v => {
                                map.insert(mapped_name.clone(), v.clone());
                                continue;
                            }
                        }
                        match self.map_internal(&new_keys, mapping_value)? {
                            Some(Value::Object(output)) => {
                                for (k, v) in output.iter() {
                                    map.insert(k.to_owned(), v.clone());
                                }
                            }
                            Some(Value::Array(outputs)) => {
                                if merge_array.is_some() {
                                    return Err(JsonMapperError::ParallelListMapping);
                                }
                                merge_array = Some(outputs);
                            }
                            Some(_) => return Err(JsonMapperError::MapInternalReturnedInvalidData),
                            None => {
                                map.insert(mapped_name.clone(), mapping_value.clone());
                            }
                        }
                    }
                }

                if let Some(array) = merge_array {
                    let mut new_items = Vec::new();
                    for item in array.iter() {
                        let mut new_item = item.clone();
                        for (k, v) in map.iter() {
                            new_item[k] = v.clone();
                        }
                        new_items.push(new_item);
                    }
                    Ok(Some(Value::Array(new_items)))
                } else {
                    Ok(Some(Value::Object(map)))
                }
            }
            _ => Err(JsonMapperError::InvalidInput),
        }
    }
}
