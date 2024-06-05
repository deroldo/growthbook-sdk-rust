use serde_json::{Map, Value};

use crate::error::{GrowthbookError, GrowthbookErrorCode};

#[derive(Clone, PartialEq, Debug)]
pub struct GrowthBookAttribute {
    pub key: String,
    pub value: GrowthBookAttributeValue,
}

#[derive(Clone, PartialEq, Debug)]
pub enum GrowthBookAttributeValue {
    Empty,
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Array(Vec<GrowthBookAttributeValue>),
    Object(Vec<GrowthBookAttribute>),
}

impl GrowthBookAttribute {
    pub fn new(key: String, value: GrowthBookAttributeValue) -> Self {
        GrowthBookAttribute { key, value }
    }

    pub fn from(value: Value) -> Result<Vec<Self>, GrowthbookError> {
        if !value.is_object() {
            return Err(GrowthbookError::new(
                GrowthbookErrorCode::GrowthBookAttributeIsNotObject,
                "GrowthBookAttribute must be an object with at leat one key value pair",
            ));
        }

        let default_map = Map::new();
        let map = value.as_object().unwrap_or(&default_map);
        let mut attributes = Vec::new();
        for (key, value) in map {
            attributes.push(GrowthBookAttribute {
                key: key.clone(),
                value: GrowthBookAttributeValue::from(value.clone()),
            });
        }
        Ok(attributes)
    }
}

impl From<Value> for GrowthBookAttributeValue {
    fn from(value: Value) -> Self {
        if value.is_string() {
            GrowthBookAttributeValue::String(value.as_str().unwrap_or_default().to_string())
        } else if value.is_boolean() {
            GrowthBookAttributeValue::Bool(value.as_bool().unwrap_or_default())
        } else if value.is_i64() {
            GrowthBookAttributeValue::Int(value.as_i64().unwrap_or_default())
        } else if value.is_f64() {
            GrowthBookAttributeValue::Float(value.as_f64().unwrap_or_default())
        } else if value.is_array() {
            let vec: Vec<GrowthBookAttributeValue> = value
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .map(|item| GrowthBookAttributeValue::from(item.clone()))
                .collect();
            GrowthBookAttributeValue::Array(vec)
        } else {
            let objects = value
                .as_object()
                .unwrap_or(&Map::new())
                .iter()
                .map(|(k, v)| {
                    GrowthBookAttribute::new(k.clone(), GrowthBookAttributeValue::from(v.clone()))
                })
                .collect();
            GrowthBookAttributeValue::Object(objects)
        }
    }
}

impl GrowthBookAttributeValue {
    pub fn to_string(&self) -> String {
        match self {
            GrowthBookAttributeValue::Empty => String::new(),
            GrowthBookAttributeValue::Array(it) => it.iter().fold(String::new(), |acc, value| {
                format!("{acc}{}", value.to_string())
            }),
            GrowthBookAttributeValue::Object(it) => it.iter().fold(String::new(), |acc, att| {
                format!("{acc}{}", att.value.to_string())
            }),
            GrowthBookAttributeValue::String(it) => it.clone(),
            GrowthBookAttributeValue::Int(it) => it.to_string(),
            GrowthBookAttributeValue::Float(it) => it.to_string(),
            GrowthBookAttributeValue::Bool(it) => it.to_string(),
        }
    }
}
