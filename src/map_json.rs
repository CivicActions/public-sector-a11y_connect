use itertools::Itertools;
use serde_json::Value;
use std::collections::HashMap;

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
        let mut finished = true;
        for k in keys.keys() {
            if !k.is_empty() {
                finished = false;
                break;
            }
        }
        if finished {
            return Ok(None);
        }

        match input {
            Value::Array(items) => {
                let mut new_items = Vec::new();
                for item_opt_res in items.iter().map(|i| self.map_internal(keys, i)) {
                    let item_opt = item_opt_res?;
                    // unwrap: `keys` is unchanged for this recursive call, so if the call
                    // would return `Ok(None)`, we would have returned `Ok(None)` aswell
                    // before reaching this point.
                    let item = item_opt.unwrap();
                    match item {
                        Value::Array(items) => {
                            for item in items {
                                new_items.push(item);
                            }
                        }
                        Value::Object(obj) => {
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
                                for (_, v) in output.iter() {
                                    map.insert(mapped_name.to_owned(), v.clone());
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
