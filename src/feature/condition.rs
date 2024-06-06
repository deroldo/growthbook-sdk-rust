use regex::Regex;
use serde::Deserialize;
use serde_json::Value;

use crate::extensions::FindGrowthBookAttribute;
use crate::model_public::{GrowthBookAttribute, GrowthBookAttributeValue};

pub trait ConditionEnabledCheck {
    fn is_on(&self, user_attributes: &[GrowthBookAttribute]) -> bool;
}

impl ConditionEnabledCheck for Vec<GrowthBookAttribute> {
    fn is_on(&self, user_attributes: &[GrowthBookAttribute]) -> bool {
        self.iter()
            .all(|feature_attribute| is_on(None, feature_attribute, user_attributes, false))
    }
}

fn is_on(
    parent_attribute: Option<&GrowthBookAttribute>,
    feature_attribute: &GrowthBookAttribute,
    user_attributes: &[GrowthBookAttribute],
    array_size: bool,
) -> bool {
    match feature_attribute.key.as_str() {
        "$not" => not_condition(parent_attribute, feature_attribute, user_attributes),
        "$ne" => ne_condition(parent_attribute, feature_attribute, user_attributes),
        "$and" => and_condition(parent_attribute, feature_attribute, user_attributes),
        "$nor" => nor_condition(parent_attribute, feature_attribute, user_attributes),
        "$or" => or_condition(parent_attribute, feature_attribute, user_attributes),
        "$in" => in_condition(parent_attribute, feature_attribute, user_attributes),
        "$nin" => nin_condition(parent_attribute, feature_attribute, user_attributes),
        "$gt" => gt_condition(parent_attribute, feature_attribute, user_attributes, array_size),
        "$gte" => gte_condition(parent_attribute, feature_attribute, user_attributes, array_size),
        "$lt" => lt_condition(parent_attribute, feature_attribute, user_attributes, array_size),
        "$lte" => lte_condition(parent_attribute, feature_attribute, user_attributes, array_size),
        "$eq" => eq_condition(parent_attribute, feature_attribute, user_attributes),
        "$exists" => exists_condition(parent_attribute, feature_attribute, user_attributes),
        "$regex" => regex_condition(parent_attribute, feature_attribute, user_attributes),
        "$type" => type_condition(parent_attribute, feature_attribute, user_attributes),
        "$size" => size_condition(parent_attribute, feature_attribute, user_attributes),
        "$all" => all_condition(parent_attribute, feature_attribute, user_attributes),
        "$vgt" => vgt_condition(parent_attribute, feature_attribute, user_attributes),
        "$vgte" => vgte_condition(parent_attribute, feature_attribute, user_attributes),
        "$vlt" => vlt_condition(parent_attribute, feature_attribute, user_attributes),
        "$vlte" => vlte_condition(parent_attribute, feature_attribute, user_attributes),
        "$veq" => veq_condition(parent_attribute, feature_attribute, user_attributes),
        "$vne" => vne_condition(parent_attribute, feature_attribute, user_attributes),
        "$elemMatch" => elem_match_condition(parent_attribute, feature_attribute, user_attributes, array_size),
        _ => match &feature_attribute.value {
            GrowthBookAttributeValue::String(_) => {
                if feature_attribute.key.starts_with('$') {
                    println!("is unknown operator={:?}", &feature_attribute.key);
                    false
                } else {
                    println!("is string={:?}", &feature_attribute.key);
                    eq_condition(parent_attribute, feature_attribute, user_attributes)
                }
            }
            GrowthBookAttributeValue::Array(feature_values) => {
                let a = if let Some(GrowthBookAttributeValue::Array(user_values)) = user_attributes.find_value(&parent_attribute.unwrap_or(feature_attribute).key) {
                    if feature_values.len() == user_values.len() {
                        feature_values.iter().enumerate().all(|(index, value)| {
                            value == &user_values[index]
                        })
                    } else {
                        false
                    }
                } else {
                    false
                };
                println!("is array={:?}", &feature_attribute.key);
                a
            }
            GrowthBookAttributeValue::Object(it) => {
                println!("is object={:?}", &feature_attribute.key);
                if it.is_empty() {
                    user_attributes.find_value(&parent_attribute.unwrap_or(feature_attribute).key).is_none()
                } else {
                    it.iter()
                        .all(|next| {
                            let parent = feature_attribute.aggregate_key(parent_attribute);
                            is_on(Some(&parent), next, user_attributes, false)
                        })
                }
            }
            it => {
                println!("key not found={:?} is={:?}", &feature_attribute.key, it);
                if let Some(it) = user_attributes.find_value(&parent_attribute.unwrap_or(feature_attribute).key) {
                    it == GrowthBookAttributeValue::Empty
                } else {
                    true
                }
            }
        },
    }
}

impl GrowthBookAttribute {
    fn aggregate_key(&self, parent_attribute: Option<&GrowthBookAttribute>) -> Self {
        let key = parent_attribute.map(|parent| format!("{}.{}", parent.key, self.key)).unwrap_or(self.key.clone());
        GrowthBookAttribute {
            key,
            value: self.value.clone(),
        }
    }
}

fn in_condition(
    parent_attribute: Option<&GrowthBookAttribute>,
    feature_attribute: &GrowthBookAttribute,
    user_attributes: &[GrowthBookAttribute],
) -> bool {
    let a = if let Some(user_value) =
        user_attributes.find_value(&parent_attribute.unwrap_or(feature_attribute).key)
    {
        match &feature_attribute.value {
            GrowthBookAttributeValue::Array(feature_array) => feature_array.iter().any(|feature_item| {
                match &user_value {
                    GrowthBookAttributeValue::Array(user_array) => {
                        user_array.iter().any(|user_item| feature_item.to_string() == user_item.to_string())
                    }
                    GrowthBookAttributeValue::Empty => false,
                    it => feature_item.to_string() == it.to_string(),
                }
            }),
            _ => false,
        }
    } else {
        false
    };
    println!("in_condition={a}");
    a
}

fn nin_condition(
    parent_attribute: Option<&GrowthBookAttribute>,
    feature_attribute: &GrowthBookAttribute,
    user_attributes: &[GrowthBookAttribute],
) -> bool {
    let a = if let Some(user_value) =
        user_attributes.find_value(&parent_attribute.unwrap_or(feature_attribute).key)
    {
        match &feature_attribute.value {
            GrowthBookAttributeValue::Array(feature_array) => feature_array.iter().all(|feature_item| {
                !match &user_value {
                    GrowthBookAttributeValue::Array(user_array) => {
                        user_array.iter().any(|user_item| feature_item.to_string() == user_item.to_string())
                    }
                    GrowthBookAttributeValue::Empty => false,
                    it => feature_item.to_string() == it.to_string(),
                }
            }),
            _ => false,
        }
    } else {
        false
    };
    println!("nin_condition={a}");
    a
}

fn or_condition(
    _parent_attribute: Option<&GrowthBookAttribute>,
    feature_attribute: &GrowthBookAttribute,
    user_attributes: &[GrowthBookAttribute],
) -> bool {
    let a = match &feature_attribute.value {
        GrowthBookAttributeValue::Array(it) => {
            if it.is_empty() {
                true
            } else {
                it.iter().any(|next_value| match next_value {
                    GrowthBookAttributeValue::Object(feature_value) => feature_value
                        .iter()
                        .all(|next_attribute| is_on(None, next_attribute, user_attributes, false)),
                    _ => false,
                })
            }
        },
        GrowthBookAttributeValue::Empty => true,
        _ => false,
    };
    println!("or_condition={a}");
    a
}

fn not_condition(
    parent_attribute: Option<&GrowthBookAttribute>,
    feature_attribute: &GrowthBookAttribute,
    user_attributes: &[GrowthBookAttribute],
) -> bool {
    let a = match &feature_attribute.value {
        GrowthBookAttributeValue::Object(it) => it
            .iter()
            .all(|next| !is_on(parent_attribute, next, user_attributes, false)),
        _ => false,
    };
    println!("not_condition={a}");
    a
}

fn and_condition(
    _parent_attribute: Option<&GrowthBookAttribute>,
    feature_attribute: &GrowthBookAttribute,
    user_attributes: &[GrowthBookAttribute],
) -> bool {
    let a = match &feature_attribute.value {
        GrowthBookAttributeValue::Array(it) => it.iter().all(|next_value| match next_value {
            GrowthBookAttributeValue::Object(feature_value) => feature_value
                .iter()
                .all(|next_attribute| is_on(None, next_attribute, user_attributes, false)),
            _ => false,
        }),
        _ => false,
    };
    println!("and_condition={a}");
    a
}

fn nor_condition(
    _parent_attribute: Option<&GrowthBookAttribute>,
    feature_attribute: &GrowthBookAttribute,
    user_attributes: &[GrowthBookAttribute],
) -> bool {
    let a = match &feature_attribute.value {
        GrowthBookAttributeValue::Array(it) => it.iter().all(|next_value| match next_value {
            GrowthBookAttributeValue::Object(feature_value) => !feature_value
                .iter()
                .all(|next_attribute| is_on(None, next_attribute, user_attributes, false)),
            _ => false,
        }),
        _ => false,
    };
    println!("nor_condition={a}");
    a
}

fn ne_condition(
    parent_attribute: Option<&GrowthBookAttribute>,
    feature_attribute: &GrowthBookAttribute,
    user_attributes: &[GrowthBookAttribute],
) -> bool {
    let a = if let Some(user_value) =
        user_attributes.find_value(&parent_attribute.unwrap_or(feature_attribute).key)
    {
        !match &user_value {
            GrowthBookAttributeValue::Array(it) => {
                it.iter().any(|item| item == &feature_attribute.value)
            }
            GrowthBookAttributeValue::Empty => true,
            it => it == &feature_attribute.value,
        }
    } else {
        true
    };
    println!("ne_condition={a}");
    a
}

fn gt_condition(
    parent_attribute: Option<&GrowthBookAttribute>,
    feature_attribute: &GrowthBookAttribute,
    user_attributes: &[GrowthBookAttribute],
    array_size: bool,
) -> bool {
    let a = if feature_attribute.value.is_number() {
        number_condition_evaluate(
            parent_attribute,
            feature_attribute,
            user_attributes,
            array_size,
            |feature_number, user_number| user_number.gt(feature_number),
        )
    } else {
        string_condition_evaluate(
            parent_attribute,
            feature_attribute,
            user_attributes,
            |feature_string, user_string| user_string.gt(feature_string),
        )
    };
    println!("gt_condition={a}");
    a
}

fn gte_condition(
    parent_attribute: Option<&GrowthBookAttribute>,
    feature_attribute: &GrowthBookAttribute,
    user_attributes: &[GrowthBookAttribute],
    array_size: bool,
) -> bool {
    let a = if feature_attribute.value.is_number() {
        number_condition_evaluate(
            parent_attribute,
            feature_attribute,
            user_attributes,
            array_size,
            |feature_number, user_number| user_number.ge(feature_number),
        )
    } else {
        string_condition_evaluate(
            parent_attribute,
            feature_attribute,
            user_attributes,
            |feature_string, user_string| user_string.ge(feature_string),
        )
    };
    println!("gte_condition={a}");
    a
}

fn lt_condition(
    parent_attribute: Option<&GrowthBookAttribute>,
    feature_attribute: &GrowthBookAttribute,
    user_attributes: &[GrowthBookAttribute],
    array_size: bool,
) -> bool {
    let a = if feature_attribute.value.is_number() {
        number_condition_evaluate(
            parent_attribute,
            feature_attribute,
            user_attributes,
            array_size,
            |feature_number, user_number| user_number.lt(feature_number),
        )
    } else {
        string_condition_evaluate(
            parent_attribute,
            feature_attribute,
            user_attributes,
            |feature_string, user_string| user_string.lt(feature_string),
        )
    };
    println!("lt_condition={a}");
    a
}

fn lte_condition(
    parent_attribute: Option<&GrowthBookAttribute>,
    feature_attribute: &GrowthBookAttribute,
    user_attributes: &[GrowthBookAttribute],
    array_size: bool,
) -> bool {
    let a = if feature_attribute.value.is_number() {
        number_condition_evaluate(
            parent_attribute,
            feature_attribute,
            user_attributes,
            array_size,
            |feature_number, user_number| user_number.le(feature_number),
        )
    } else {
        string_condition_evaluate(
            parent_attribute,
            feature_attribute,
            user_attributes,
            |feature_string, user_string| user_string.le(feature_string),
        )
    };
    println!("lte_condition={a}");
    a
}

fn number_condition_evaluate(
    parent_attribute: Option<&GrowthBookAttribute>,
    feature_attribute: &GrowthBookAttribute,
    user_attributes: &[GrowthBookAttribute],
    array_size: bool,
    condition: fn(&f64, &f64) -> bool,
) -> bool {
    let a = if let Some(user_value) =
        user_attributes.find_value(&parent_attribute.unwrap_or(feature_attribute).key)
    {
        let feature_number = if let GrowthBookAttributeValue::Int(it) = feature_attribute.value {
            it as f64
        } else if let GrowthBookAttributeValue::Float(it) = feature_attribute.value {
            it
        } else if let GrowthBookAttributeValue::String(string_number) = &feature_attribute.value {
            if let Ok(it) = string_number.replace('.', "").parse::<f64>() {
                it
            } else {
                return false;
            }
        } else {
            return false;
        };

        let user_numbers = if let GrowthBookAttributeValue::Int(it) = user_value {
            vec![it as f64]
        } else if let GrowthBookAttributeValue::Float(it) = user_value {
            vec![it]
        } else if let GrowthBookAttributeValue::Array(it) = user_value {
            if array_size {
                vec![it.len() as f64]
            } else {
                it.iter()
                    .filter(|item| item.is_number())
                    .map(|item| item.as_f64().expect("Failed to convert to f64"))
                    .collect()
            }
        } else if let GrowthBookAttributeValue::String(string_number) = &user_value {
            if let Ok(it) = string_number.replace('.', "").parse::<f64>() {
                vec![it]
            } else {
                return false;
            }
        } else {
            return false;
        };

        user_numbers.iter().any(|number| condition(&feature_number, number))
    } else {
        true
    };
    println!("number_condition_evaluate={a}");
    a
}

fn string_condition_evaluate(
    parent_attribute: Option<&GrowthBookAttribute>,
    feature_attribute: &GrowthBookAttribute,
    user_attributes: &[GrowthBookAttribute],
    condition: fn(&str, &str) -> bool,
) -> bool {
    let a = if let Some(user_value) =
        user_attributes.find_value(&parent_attribute.unwrap_or(feature_attribute).key)
    {
        let feature_value = feature_attribute.value.to_string();
        match user_value {
            GrowthBookAttributeValue::Array(it) => {
                it.iter().any(|item| condition(&feature_value, &item.to_string()))
            }
            it => {
                condition(&feature_value, &it.to_string())
            }
        }
    } else {
        true
    };
    println!("string_condition_evaluate={a}");
    a
}

fn eq_condition(
    parent_attribute: Option<&GrowthBookAttribute>,
    feature_attribute: &GrowthBookAttribute,
    user_attributes: &[GrowthBookAttribute],
) -> bool {
    let a = if let Some(user_value) =
        user_attributes.find_value(&parent_attribute.unwrap_or(feature_attribute).key)
    {
        match &user_value {
            GrowthBookAttributeValue::Array(it) => {
                it.iter().any(|item| item == &feature_attribute.value)
            }
            GrowthBookAttributeValue::Empty => false,
            it => {
                it.to_string() == feature_attribute.value.to_string()
            },
        }
    } else {
        false
    };
    println!("eq_condition={a}");
    a
}

fn exists_condition(
    parent_attribute: Option<&GrowthBookAttribute>,
    feature_attribute: &GrowthBookAttribute,
    user_attributes: &[GrowthBookAttribute],
) -> bool {
    let a = if let GrowthBookAttributeValue::Bool(it) = feature_attribute.value {
        if user_attributes
            .find_value(&parent_attribute.unwrap_or(feature_attribute).key)
            .is_some()
        {
            it
        } else {
            !it
        }
    } else {
        true
    };
    println!("exists_condition={a}");
    a
}

fn regex_condition(
    parent_attribute: Option<&GrowthBookAttribute>,
    feature_attribute: &GrowthBookAttribute,
    user_attributes: &[GrowthBookAttribute],
) -> bool {
    let a = if let GrowthBookAttributeValue::String(feature_value) = &feature_attribute.value {
        if let Ok(regex) = Regex::new(feature_value) {
            if let Some(user_value) =
                user_attributes.find_value(&parent_attribute.unwrap_or(feature_attribute).key)
            {
                match &user_value {
                    GrowthBookAttributeValue::Array(it) => {
                        it.iter().any(|item| regex.is_match(&item.to_string()))
                    }
                    it => regex.is_match(&it.to_string()),
                }
            } else {
                false
            }
        } else {
            false
        }
    } else {
        true
    };
    println!("regex_condition={a}");
    a
}

fn type_condition(
    parent_attribute: Option<&GrowthBookAttribute>,
    feature_attribute: &GrowthBookAttribute,
    user_attributes: &[GrowthBookAttribute],
) -> bool {
    let a = if let GrowthBookAttributeValue::String(feature_type) = &feature_attribute.value {
        if let Some(user_value) = user_attributes.find_value(&parent_attribute.unwrap_or(feature_attribute).key) {
            match user_value {
                GrowthBookAttributeValue::String(_) => feature_type == "string",
                GrowthBookAttributeValue::Int(_) => feature_type == "number",
                GrowthBookAttributeValue::Float(_) => feature_type == "number",
                GrowthBookAttributeValue::Bool(_) => feature_type == "boolean",
                GrowthBookAttributeValue::Array(_) => feature_type == "array",
                GrowthBookAttributeValue::Object(it) => {
                    if it.is_empty() {
                        feature_type == "null"
                    } else {
                        feature_type == "object"
                    }
                },
                GrowthBookAttributeValue::Empty => feature_type == "null",
            }
        } else {
            feature_type == "null"
        }
    } else {
        false
    };
    println!("type_condition={a}");
    a
}

fn size_condition(
    parent_attribute: Option<&GrowthBookAttribute>,
    feature_attribute: &GrowthBookAttribute,
    user_attributes: &[GrowthBookAttribute],
) -> bool {
    let a = match &feature_attribute.value {
        GrowthBookAttributeValue::Int(feature_value) => {
            if let Some(user_value) = user_attributes.find_value(&parent_attribute.unwrap_or(feature_attribute).key) {
                match user_value {
                    GrowthBookAttributeValue::Array(it) => feature_value == &(it.len() as i64),
                    _ => false,
                }
            } else {
                false
            }
        }
        GrowthBookAttributeValue::Object(feature_value) => {
            feature_value.iter().all(|next| is_on(parent_attribute, next, user_attributes, true))
        }
        _ => false,
    };
    println!("size_condition={a}");
    a
}

fn all_condition(
    parent_attribute: Option<&GrowthBookAttribute>,
    feature_attribute: &GrowthBookAttribute,
    user_attributes: &[GrowthBookAttribute],
) -> bool {
    let a = match &feature_attribute.value {
        GrowthBookAttributeValue::Array(feature_values) => {
            if let Some(GrowthBookAttributeValue::Array(user_values)) = user_attributes.find_value(&parent_attribute.unwrap_or(feature_attribute).key) {
                feature_values.iter().all(|feature_item| user_values.iter().any(|user_item| feature_item == user_item))
            } else {
                false
            }
        }
        _ => false,
    };
    println!("size_condition={a}");
    a
}

fn vgt_condition(
    parent_attribute: Option<&GrowthBookAttribute>,
    feature_attribute: &GrowthBookAttribute,
    user_attributes: &[GrowthBookAttribute],
) -> bool {
    let a = version_condition_evaluate(
        parent_attribute,
        feature_attribute,
        user_attributes,
        |feature_version, user_version| user_version.gt(feature_version),
    );
    println!("vgt_condition={a}");
    a
}

fn vgte_condition(
    parent_attribute: Option<&GrowthBookAttribute>,
    feature_attribute: &GrowthBookAttribute,
    user_attributes: &[GrowthBookAttribute],
) -> bool {
    let a = version_condition_evaluate(
        parent_attribute,
        feature_attribute,
        user_attributes,
        |feature_version, user_version| user_version.ge(feature_version),
    );
    println!("vgte_condition={a}");
    a
}

fn vlt_condition(
    parent_attribute: Option<&GrowthBookAttribute>,
    feature_attribute: &GrowthBookAttribute,
    user_attributes: &[GrowthBookAttribute],
) -> bool {
    let a = version_condition_evaluate(
        parent_attribute,
        feature_attribute,
        user_attributes,
        |feature_version, user_version| user_version.lt(feature_version),
    );
    println!("vlt_condition={a}");
    a
}

fn vlte_condition(
    parent_attribute: Option<&GrowthBookAttribute>,
    feature_attribute: &GrowthBookAttribute,
    user_attributes: &[GrowthBookAttribute],
) -> bool {
    let a = version_condition_evaluate(
        parent_attribute,
        feature_attribute,
        user_attributes,
        |feature_version, user_version| user_version.le(feature_version),
    );
    println!("vlte_condition={a}");
    a
}

fn veq_condition(
    parent_attribute: Option<&GrowthBookAttribute>,
    feature_attribute: &GrowthBookAttribute,
    user_attributes: &[GrowthBookAttribute],
) -> bool {
    let a = version_condition_evaluate(
        parent_attribute,
        feature_attribute,
        user_attributes,
        |feature_version, user_version| user_version.eq(feature_version),
    );
    println!("veq_condition={a}");
    a
}

fn vne_condition(
    parent_attribute: Option<&GrowthBookAttribute>,
    feature_attribute: &GrowthBookAttribute,
    user_attributes: &[GrowthBookAttribute],
) -> bool {
    let a = version_condition_evaluate(
        parent_attribute,
        feature_attribute,
        user_attributes,
        |feature_version, user_version| user_version.ne(feature_version),
    );
    println!("vne_condition={a}");
    a
}

fn version_condition_evaluate(
    parent_attribute: Option<&GrowthBookAttribute>,
    feature_attribute: &GrowthBookAttribute,
    user_attributes: &[GrowthBookAttribute],
    condition: fn(&str, &str) -> bool,
) -> bool {
    let a = if let Some(GrowthBookAttributeValue::String(user_version)) =
        user_attributes.find_value(&parent_attribute.unwrap_or(feature_attribute).key)
    {
        let feature_version = feature_attribute.value.to_string();
        println!("{}", &normalize_version(&feature_version));
        println!("{}", &normalize_version(&user_version));
        condition(&normalize_version(&feature_version), &normalize_version(&user_version))
    } else {
        true
    };
    println!("version_condition_evaluate={a}");
    a
}

fn normalize_version(version: &str) -> String {
    if let Ok(regex1) = Regex::new("(^v|\\+.*$)"){
        if let Ok(regex2) = Regex::new("[-.]"){
            if let Ok(regex3) = Regex::new("^\\d+"){
                let string = regex1.replace_all(version, "").to_string();
                let mut split = regex2.split(&string).filter(|item| !item.is_empty()).collect::<Vec<&str>>();
                if split.len().clone() == 3 {
                    split.push("~");
                }
                split.iter()
                    .map(|part| {
                        if regex3.is_match(part) {
                            format!("{:0>5}", part)
                        } else {
                            part.to_string()
                        }
                    }).filter(|part| !part.is_empty())
                    .reduce(|a, b| format!("{a}-{b}"))
                    .unwrap_or(version.to_string())
            } else {
                version.to_string()
            }
        } else {
            version.to_string()
        }
    } else {
        version.to_string()
    }
}

fn elem_match_condition(
    parent_attribute: Option<&GrowthBookAttribute>,
    feature_attribute: &GrowthBookAttribute,
    user_attributes: &[GrowthBookAttribute],
    array_size: bool,
) -> bool {
    let a = match &feature_attribute.value {
        GrowthBookAttributeValue::Object(it) => it.iter().any(|condition_attribute| {
            is_on(parent_attribute, condition_attribute, user_attributes, array_size)
        }),
        _ => false,
    };
    println!("elem_match_condition={a}");
    a
}

#[cfg(test)]
mod test {
    use crate::feature::condition::{is_on, Cases, EvalCondition, EvalConditionValue};
    use crate::model_public::GrowthBookAttribute;

    #[tokio::test]
    async fn eval_conditions() -> Result<(), Box<dyn std::error::Error>> {
        let cases = Cases::new();

        for value in cases.eval_condition {
            let eval_condition = value_to_eval_condition(value);
            println!("--------------------");
            println!("eval_condition={}", eval_condition.name);
            let vec_condition = GrowthBookAttribute::from(eval_condition.condition)
                .expect("Failed to create attributes");
            let vec_attributes = GrowthBookAttribute::from(eval_condition.attribute)
                .expect("Failed to create attributes");
            println!("conditions={:?}", vec_condition);
            println!("attributes={:?}", vec_attributes);
            println!();
            let enabled = is_on(None, &vec_condition[0], &vec_attributes, false);
            println!("--------------------");
            if enabled != eval_condition.result {
                panic!("EvalCondition failed: {}", eval_condition.name)
            }
        }

        Ok(())
    }

    fn value_to_eval_condition(value: EvalConditionValue) -> EvalCondition {
        match value {
            EvalConditionValue::Condition(condition) => EvalCondition::new(condition),
        }
    }
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Cases {
    #[allow(dead_code)]
    eval_condition: Vec<EvalConditionValue>,
}

#[derive(Deserialize, Clone)]
#[serde(untagged)]
enum EvalConditionValue {
    Condition(Value),
}

#[allow(dead_code)]
pub struct EvalCondition {
    name: String,
    condition: Value,
    attribute: Value,
    result: bool,
}

impl EvalCondition {
    #[allow(dead_code)]
    fn new(value: Value) -> Self {
        let array = value.as_array().expect("Failed to convert to array");
        Self {
            name: array[0]
                .as_str()
                .expect("Failed to convert do str")
                .to_string(),
            condition: array[1].clone(),
            attribute: array[2].clone(),
            result: array[3].as_bool().expect("Failed to convert to bool"),
        }
    }
}

impl Cases {
    #[allow(dead_code)]
    pub fn new() -> Self {
        serde_json::from_str(&json()).expect("Failed to create cases")
    }
}

#[allow(dead_code)]
fn json() -> String {
    r#"
{
  "specVersion": "0.6.0",
  "evalCondition": [
    [
      "$not - pass",
      {
        "$not": {
          "name": "hello"
        }
      },
      {
        "name": "world"
      },
      true
    ],
    [
      "$not - fail",
      {
        "$not": {
          "name": "hello"
        }
      },
      {
        "name": "hello"
      },
      false
    ],
    [
      "$and/$or - all true",
      {
        "$and": [
          {
            "father.age": {
              "$gt": 65
            }
          },
          {
            "$or": [
              {
                "bday": {
                  "$regex": "-12-25$"
                }
              },
              {
                "name": "santa"
              }
            ]
          }
        ]
      },
      {
        "name": "santa",
        "bday": "1980-12-25",
        "father": {
          "age": 70
        }
      },
      true
    ],
    [
      "$groups - match",
      {
        "$and": [
          {
            "$groups": {
              "$elemMatch": { "$eq": "a" }
            }
          },
          {
            "$groups": {
              "$elemMatch": { "$eq": "b" }
            }
          },
          {
            "$or": [
              {
                "$groups": {
                  "$elemMatch": { "$eq": "c" }
                }
              },
              {
                "$groups": {
                  "$elemMatch": { "$eq": "e" }
                }
              }
            ]
          },
          {
            "$not": {
              "$groups": {
                "$elemMatch": { "$eq": "f" }
              }
            }
          },
          {
            "$not": {
              "$groups": {
                "$elemMatch": { "$eq": "g" }
              }
            }
          }
        ]
      },
      {
        "$groups": ["a", "b", "c", "d"]
      },
      true
    ],
    [
      "$groups - no match",
      {
        "$and": [
          {
            "$groups": {
              "$elemMatch": { "$eq": "a" }
            }
          },
          {
            "$groups": {
              "$elemMatch": { "$eq": "b" }
            }
          },
          {
            "$or": [
              {
                "$groups": {
                  "$elemMatch": { "$eq": "c" }
                }
              },
              {
                "$groups": {
                  "$elemMatch": { "$eq": "e" }
                }
              }
            ]
          },
          {
            "$not": {
              "$groups": {
                "$elemMatch": { "$eq": "d" }
              }
            }
          },
          {
            "$not": {
              "$groups": {
                "$elemMatch": { "$eq": "g" }
              }
            }
          }
        ]
      },
      {
        "$groups": ["a", "b", "c", "d"]
      },
      false
    ],
    [
      "$and/$or - first or true",
      {
        "$and": [
          {
            "father.age": {
              "$gt": 65
            }
          },
          {
            "$or": [
              {
                "bday": {
                  "$regex": "-12-25$"
                }
              },
              {
                "name": "santa"
              }
            ]
          }
        ]
      },
      {
        "name": "santa",
        "bday": "1980-12-20",
        "father": {
          "age": 70
        }
      },
      true
    ],
    [
      "$and/$or - second or true",
      {
        "$and": [
          {
            "father.age": {
              "$gt": 65
            }
          },
          {
            "$or": [
              {
                "bday": {
                  "$regex": "-12-25$"
                }
              },
              {
                "name": "santa"
              }
            ]
          }
        ]
      },
      {
        "name": "barbara",
        "bday": "1980-12-25",
        "father": {
          "age": 70
        }
      },
      true
    ],
    [
      "$and/$or - first and false",
      {
        "$and": [
          {
            "father.age": {
              "$gt": 65
            }
          },
          {
            "$or": [
              {
                "bday": {
                  "$regex": "-12-25$"
                }
              },
              {
                "name": "santa"
              }
            ]
          }
        ]
      },
      {
        "name": "santa",
        "bday": "1980-12-25",
        "father": {
          "age": 65
        }
      },
      false
    ],
    [
      "$and/$or - both or false",
      {
        "$and": [
          {
            "father.age": {
              "$gt": 65
            }
          },
          {
            "$or": [
              {
                "bday": {
                  "$regex": "-12-25$"
                }
              },
              {
                "name": "santa"
              }
            ]
          }
        ]
      },
      {
        "name": "barbara",
        "bday": "1980-11-25",
        "father": {
          "age": 70
        }
      },
      false
    ],
    [
      "$and/$or - both and false",
      {
        "$and": [
          {
            "father.age": {
              "$gt": 65
            }
          },
          {
            "$or": [
              {
                "bday": {
                  "$regex": "-12-25$"
                }
              },
              {
                "name": "santa"
              }
            ]
          }
        ]
      },
      {
        "name": "john smith",
        "bday": "1956-12-20",
        "father": {
          "age": 40
        }
      },
      false
    ],
    [
      "$exists - false pass",
      {
        "pets.dog.name": {
          "$exists": false
        }
      },
      {
        "hello": "world"
      },
      true
    ],
    [
      "$exists - false fail",
      {
        "pets.dog.name": {
          "$exists": false
        }
      },
      {
        "pets": {
          "dog": {
            "name": "fido"
          }
        }
      },
      false
    ],
    [
      "$exists - true fail",
      {
        "pets.dog.name": {
          "$exists": true
        }
      },
      {
        "hello": "world"
      },
      false
    ],
    [
      "$exists - true pass",
      {
        "pets.dog.name": {
          "$exists": true
        }
      },
      {
        "pets": {
          "dog": {
            "name": "fido"
          }
        }
      },
      true
    ],
    [
      "equals - multiple datatypes",
      {
        "str": "str",
        "num": 10,
        "flag": false
      },
      {
        "str": "str",
        "num": 10,
        "flag": false
      },
      true
    ],
    [
      "$in - pass",
      {
        "num": {
          "$in": [1, 2, 3]
        }
      },
      {
        "num": 2
      },
      true
    ],
    [
      "$in - fail",
      {
        "num": {
          "$in": [1, 2, 3]
        }
      },
      {
        "num": 4
      },
      false
    ],
    [
      "$in - not array",
      {
        "num": {
          "$in": 1
        }
      },
      {
        "num": 1
      },
      false
    ],
    [
      "$in - array pass 1",
      {
        "tags": {
          "$in": ["a", "b"]
        }
      },
      {
        "tags": ["d", "e", "a"]
      },
      true
    ],
    [
      "$in - array pass 2",
      {
        "tags": {
          "$in": ["a", "b"]
        }
      },
      {
        "tags": ["d", "b", "f"]
      },
      true
    ],
    [
      "$in - array pass 3",
      {
        "tags": {
          "$in": ["a", "b"]
        }
      },
      {
        "tags": ["d", "b", "a"]
      },
      true
    ],
    [
      "$in - array fail 1",
      {
        "tags": {
          "$in": ["a", "b"]
        }
      },
      {
        "tags": ["d", "e", "f"]
      },
      false
    ],
    [
      "$in - array fail 2",
      {
        "tags": {
          "$in": ["a", "b"]
        }
      },
      {
        "tags": []
      },
      false
    ],
    [
      "$nin - pass",
      {
        "num": {
          "$nin": [1, 2, 3]
        }
      },
      {
        "num": 4
      },
      true
    ],
    [
      "$nin - fail",
      {
        "num": {
          "$nin": [1, 2, 3]
        }
      },
      {
        "num": 2
      },
      false
    ],
    [
      "$nin - not array",
      {
        "num": {
          "$nin": 1
        }
      },
      {
        "num": 1
      },
      false
    ],
    [
      "$nin - array fail 1",
      {
        "tags": {
          "$nin": ["a", "b"]
        }
      },
      {
        "tags": ["d", "e", "a"]
      },
      false
    ],
    [
      "$nin - array fail 2",
      {
        "tags": {
          "$nin": ["a", "b"]
        }
      },
      {
        "tags": ["d", "b", "f"]
      },
      false
    ],
    [
      "$nin - array fail 3",
      {
        "tags": {
          "$nin": ["a", "b"]
        }
      },
      {
        "tags": ["d", "b", "a"]
      },
      false
    ],
    [
      "$nin - array pass 1",
      {
        "tags": {
          "$nin": ["a", "b"]
        }
      },
      {
        "tags": ["d", "e", "f"]
      },
      true
    ],
    [
      "$nin - array pass 2",
      {
        "tags": {
          "$nin": ["a", "b"]
        }
      },
      {
        "tags": []
      },
      true
    ],
    [
      "$elemMatch - pass - flat arrays",
      {
        "nums": {
          "$elemMatch": {
            "$gt": 10
          }
        }
      },
      {
        "nums": [0, 5, -20, 15]
      },
      true
    ],
    [
      "$elemMatch - fail - flat arrays",
      {
        "nums": {
          "$elemMatch": {
            "$gt": 10
          }
        }
      },
      {
        "nums": [0, 5, -20, 8]
      },
      false
    ],
    [
      "missing attribute - fail",
      {
        "pets.dog.name": {
          "$in": ["fido"]
        }
      },
      {
        "hello": "world"
      },
      false
    ],
    [
      "missing attribute with comparison operators",
      {
        "age": {
          "$gt": -10,
          "$lt": 10,
          "$gte": -9,
          "$lte": 9,
          "$ne": 10
        }
      },
      {},
      true
    ],
    [
      "comparing numbers and strings",
      {
        "n": {
          "$gt": 5,
          "$lt": 10
        }
      },
      {
        "n": "8"
      },
      true
    ],
    [
      "comparing numbers and strings - v2",
      {
        "n": {
          "$gt": "5",
          "$lt": "10"
        }
      },
      {
        "n": 8
      },
      true
    ],
    [
      "empty $or - pass",
      {
        "$or": []
      },
      {
        "hello": "world"
      },
      true
    ],
    [
      "empty $and - pass",
      {
        "$and": []
      },
      {
        "hello": "world"
      },
      true
    ],
    [
      "$eq - pass",
      {
        "occupation": {
          "$eq": "engineer"
        }
      },
      {
        "occupation": "engineer"
      },
      true
    ],
    [
      "$eq - fail",
      {
        "occupation": {
          "$eq": "engineer"
        }
      },
      {
        "occupation": "civil engineer"
      },
      false
    ],
    [
      "$ne - pass",
      {
        "level": {
          "$ne": "senior"
        }
      },
      {
        "level": "junior"
      },
      true
    ],
    [
      "$ne - fail",
      {
        "level": {
          "$ne": "senior"
        }
      },
      {
        "level": "senior"
      },
      false
    ],
    [
      "$regex - pass",
      {
        "userAgent": {
          "$regex": "(Mobile|Tablet)"
        }
      },
      {
        "userAgent": "Android Mobile Browser"
      },
      true
    ],
    [
      "$regex - fail",
      {
        "userAgent": {
          "$regex": "(Mobile|Tablet)"
        }
      },
      {
        "userAgent": "Chrome Desktop Browser"
      },
      false
    ],
    [
      "$gt/$lt numbers - pass",
      {
        "age": {
          "$gt": 30,
          "$lt": 60
        }
      },
      {
        "age": 50
      },
      true
    ],
    [
      "$gt/$lt numbers - fail $lt",
      {
        "age": {
          "$gt": 30,
          "$lt": 60
        }
      },
      {
        "age": 60
      },
      false
    ],
    [
      "$gt/$lt numbers - fail $gt",
      {
        "age": {
          "$gt": 30,
          "$lt": 60
        }
      },
      {
        "age": 30
      },
      false
    ],
    [
      "$gte/$lte numbers - pass",
      {
        "age": {
          "$gte": 30,
          "$lte": 60
        }
      },
      {
        "age": 50
      },
      true
    ],
    [
      "$gte/$lte numbers - pass $gte",
      {
        "age": {
          "$gte": 30,
          "$lte": 60
        }
      },
      {
        "age": 30
      },
      true
    ],
    [
      "$gte/$lte numbers - pass $lte",
      {
        "age": {
          "$gte": 30,
          "$lte": 60
        }
      },
      {
        "age": 60
      },
      true
    ],
    [
      "$gte/$lte numbers - fail $lte",
      {
        "age": {
          "$gte": 30,
          "$lte": 60
        }
      },
      {
        "age": 61
      },
      false
    ],
    [
      "$gte/$lte numbers - fail $gte",
      {
        "age": {
          "$gt": 30,
          "$lt": 60
        }
      },
      {
        "age": 29
      },
      false
    ],
    [
      "$gt/$lt strings - fail $gt",
      {
        "word": {
          "$gt": "alphabet",
          "$lt": "zebra"
        }
      },
      {
        "word": "alphabet"
      },
      false
    ],
    [
      "$gt/$lt strings - fail $lt",
      {
        "word": {
          "$gt": "alphabet",
          "$lt": "zebra"
        }
      },
      {
        "word": "zebra"
      },
      false
    ],
    [
      "$gt/$lt strings - pass",
      {
        "word": {
          "$gt": "alphabet",
          "$lt": "zebra"
        }
      },
      {
        "word": "always"
      },
      true
    ],
    [
      "$gt/$lt strings - fail uppercase",
      {
        "word": {
          "$gt": "alphabet",
          "$lt": "zebra"
        }
      },
      {
        "word": "AZL"
      },
      false
    ],
    [
      "nested value is null",
      {
        "address.state": "CA"
      },
      {
        "address": null
      },
      false
    ],
    [
      "nested value is integer",
      {
        "address.state": "CA"
      },
      {
        "address": 123
      },
      false
    ],
    [
      "$type string - pass",
      {
        "a": {
          "$type": "string"
        }
      },
      {
        "a": "a"
      },
      true
    ],
    [
      "$type string - fail",
      {
        "a": {
          "$type": "string"
        }
      },
      {
        "a": 1
      },
      false
    ],
    [
      "$type null - pass",
      {
        "a": {
          "$type": "null"
        }
      },
      {
        "a": null
      },
      true
    ],
    [
      "$type null - fail",
      {
        "a": {
          "$type": "null"
        }
      },
      {
        "a": 1
      },
      false
    ],
    [
      "$type boolean - pass",
      {
        "a": {
          "$type": "boolean"
        }
      },
      {
        "a": false
      },
      true
    ],
    [
      "$type boolean - fail",
      {
        "a": {
          "$type": "boolean"
        }
      },
      {
        "a": 1
      },
      false
    ],
    [
      "$type number - pass",
      {
        "a": {
          "$type": "number"
        }
      },
      {
        "a": 1
      },
      true
    ],
    [
      "$type number - fail",
      {
        "a": {
          "$type": "number"
        }
      },
      {
        "a": "a"
      },
      false
    ],
    [
      "$type object - pass",
      {
        "a": {
          "$type": "object"
        }
      },
      {
        "a": {
          "a": "b"
        }
      },
      true
    ],
    [
      "$type object - fail",
      {
        "a": {
          "$type": "object"
        }
      },
      {
        "a": 1
      },
      false
    ],
    [
      "$type array - pass",
      {
        "a": {
          "$type": "array"
        }
      },
      {
        "a": [1, 2]
      },
      true
    ],
    [
      "$type array - fail",
      {
        "a": {
          "$type": "array"
        }
      },
      {
        "a": 1
      },
      false
    ],
    [
      "unknown operator - pass",
      {
        "name": {
          "$regx": "hello"
        }
      },
      {
        "name": "hello"
      },
      false
    ],
    [
      "$regex invalid - pass",
      {
        "name": {
          "$regex": "/???***[)"
        }
      },
      {
        "name": "hello"
      },
      false
    ],
    [
      "$regex invalid - fail",
      {
        "name": {
          "$regex": "/???***[)"
        }
      },
      {
        "hello": "hello"
      },
      false
    ],
    [
      "$size empty - pass",
      {
        "tags": { "$size": 0 }
      },
      {
        "tags": []
      },
      true
    ],
    [
      "$size empty - fail",
      {
        "tags": { "$size": 0 }
      },
      {
        "tags": [10]
      },
      false
    ],
    [
      "$size number - pass",
      {
        "tags": {
          "$size": 3
        }
      },
      {
        "tags": ["a", "b", "c"]
      },
      true
    ],
    [
      "$size number - fail small",
      {
        "tags": {
          "$size": 3
        }
      },
      {
        "tags": ["a", "b"]
      },
      false
    ],
    [
      "$size number - fail large",
      {
        "tags": {
          "$size": 3
        }
      },
      {
        "tags": ["a", "b", "c", "d"]
      },
      false
    ],
    [
      "$size number - fail not array",
      {
        "tags": {
          "$size": 3
        }
      },
      {
        "tags": "abc"
      },
      false
    ],
    [
      "$size nested - pass",
      {
        "tags": {
          "$size": {
            "$gt": 2
          }
        }
      },
      {
        "tags": [0, 1, 2]
      },
      true
    ],
    [
      "$size nested - fail equal",
      {
        "tags": {
          "$size": {
            "$gt": 2
          }
        }
      },
      {
        "tags": [0, 1]
      },
      false
    ],
    [
      "$size nested - fail less than",
      {
        "tags": {
          "$size": {
            "$gt": 2
          }
        }
      },
      {
        "tags": [0]
      },
      false
    ],
    [
      "$elemMatch contains - pass",
      {
        "tags": {
          "$elemMatch": {
            "$eq": "bar"
          }
        }
      },
      {
        "tags": ["foo", "bar", "baz"]
      },
      true
    ],
    [
      "$elemMatch contains - false",
      {
        "tags": {
          "$elemMatch": {
            "$eq": "bar"
          }
        }
      },
      {
        "tags": ["foo", "baz"]
      },
      false
    ],
    [
      "$elemMatch intersection - pass",
      {
        "tags": {
          "$elemMatch": {
            "$in": ["a", "b"]
          }
        }
      },
      {
        "tags": ["d", "e", "b"]
      },
      true
    ],
    [
      "$elemMatch intersection - fail",
      {
        "tags": {
          "$elemMatch": {
            "$in": ["a", "b"]
          }
        }
      },
      {
        "tags": ["d", "e", "f"]
      },
      false
    ],
    [
      "$elemMatch not contains - pass",
      {
        "tags": {
          "$not": {
            "$elemMatch": {
              "$eq": "bar"
            }
          }
        }
      },
      {
        "tags": ["foo", "baz"]
      },
      true
    ],
    [
      "$elemMatch not contains - fail",
      {
        "tags": {
          "$not": {
            "$elemMatch": {
              "$eq": "bar"
            }
          }
        }
      },
      {
        "tags": ["foo", "bar", "baz"]
      },
      false
    ],
    [
      "$elemMatch nested - pass",
      {
        "hobbies": {
          "$elemMatch": {
            "name": {
              "$regex": "^ping"
            }
          }
        }
      },
      {
        "hobbies": [
          {
            "name": "bowling"
          },
          {
            "name": "pingpong"
          },
          {
            "name": "tennis"
          }
        ]
      },
      true
    ],
    [
      "$elemMatch nested - fail",
      {
        "hobbies": {
          "$elemMatch": {
            "name": {
              "$regex": "^ping"
            }
          }
        }
      },
      {
        "hobbies": [
          {
            "name": "bowling"
          },
          {
            "name": "tennis"
          }
        ]
      },
      false
    ],
    [
      "$elemMatch nested - fail not array",
      {
        "hobbies": {
          "$elemMatch": {
            "name": {
              "$regex": "^ping"
            }
          }
        }
      },
      {
        "hobbies": "all"
      },
      false
    ],
    [
      "$not - pass",
      {
        "name": {
          "$not": {
            "$regex": "^hello"
          }
        }
      },
      {
        "name": "world"
      },
      true
    ],
    [
      "$not - fail",
      {
        "name": {
          "$not": {
            "$regex": "^hello"
          }
        }
      },
      {
        "name": "hello world"
      },
      false
    ],
    [
      "$all - pass",
      {
        "tags": {
          "$all": ["one", "three"]
        }
      },
      {
        "tags": ["one", "two", "three"]
      },
      true
    ],
    [
      "$all - fail",
      {
        "tags": {
          "$all": ["one", "three"]
        }
      },
      {
        "tags": ["one", "two", "four"]
      },
      false
    ],
    [
      "$all - fail not array",
      {
        "tags": {
          "$all": ["one", "three"]
        }
      },
      {
        "tags": "hello"
      },
      false
    ],
    [
      "$nor - pass",
      {
        "$nor": [
          {
            "name": "john"
          },
          {
            "age": {
              "$lt": 30
            }
          }
        ]
      },
      {
        "name": "jim",
        "age": 40
      },
      true
    ],
    [
      "$nor - fail both",
      {
        "$nor": [
          {
            "name": "john"
          },
          {
            "age": {
              "$lt": 30
            }
          }
        ]
      },
      {
        "name": "john",
        "age": 20
      },
      false
    ],
    [
      "$nor - fail first",
      {
        "$nor": [
          {
            "name": "john"
          },
          {
            "age": {
              "$lt": 30
            }
          }
        ]
      },
      {
        "name": "john",
        "age": 40
      },
      false
    ],
    [
      "$nor - fail second",
      {
        "$nor": [
          {
            "name": "john"
          },
          {
            "age": {
              "$lt": 30
            }
          }
        ]
      },
      {
        "name": "jim",
        "age": 20
      },
      false
    ],
    [
      "equals array - pass",
      {
        "tags": ["hello", "world"]
      },
      {
        "tags": ["hello", "world"]
      },
      true
    ],
    [
      "equals array - fail order",
      {
        "tags": ["hello", "world"]
      },
      {
        "tags": ["world", "hello"]
      },
      false
    ],
    [
      "equals array - fail missing item",
      {
        "tags": ["hello", "world"]
      },
      {
        "tags": ["hello"]
      },
      false
    ],
    [
      "equals array - fail extra item",
      {
        "tags": ["hello", "world"]
      },
      {
        "tags": ["hello", "world", "foo"]
      },
      false
    ],
    [
      "equals array - fail type mismatch",
      {
        "tags": ["hello", "world"]
      },
      {
        "tags": "hello world"
      },
      false
    ],
    [
      "equals object - pass",
      {
        "tags": {
          "hello": "world"
        }
      },
      {
        "tags": {
          "hello": "world"
        }
      },
      true
    ],
    [
      "equals object - fail extra property",
      {
        "tags": {
          "hello": "world"
        }
      },
      {
        "tags": {
          "hello": "world",
          "yes": "please"
        }
      },
      false
    ],
    [
      "equals object - fail missing property",
      {
        "tags": {
          "hello": "world"
        }
      },
      {
        "tags": {}
      },
      false
    ],
    [
      "equals object - fail type mismatch",
      {
        "tags": {
          "hello": "world"
        }
      },
      {
        "tags": "hello world"
      },
      false
    ],
    [
      "null condition - null attribute",
      {
        "userId": null
      },
      {
        "userId": null
      },
      true
    ],
    [
      "null condition - missing attribute",
      {
        "userId": null
      },
      {},
      true
    ],
    [
      "null condition - string attribute",
      {
        "userId": null
      },
      {
        "userId": "123"
      },
      false
    ],
    [
      "null condition - zero attribute",
      {
        "userId": null
      },
      {
        "userId": 0
      },
      false
    ],
    [
      "null condition - empty string attribute",
      {
        "userId": null
      },
      {
        "userId": ""
      },
      false
    ],
    [
      "$vgt/$vlt - pass - major",
      {
        "version": {
          "$vgt": "9.99.8",
          "$vlt": "11.0.1"
        }
      },
      {
        "version": "10.12.13"
      },
      true
    ],
    [
      "$vgt/$vlt - pass - minor",
      {
        "version": {
          "$vgt": "10.2.11",
          "$vlt": "10.20.11"
        }
      },
      {
        "version": "10.12.11"
      },
      true
    ],
    [
      "$vgt/$vlt - pass - patch",
      {
        "version": {
          "$vgt": "10.0.2",
          "$vlt": "10.0.20"
        }
      },
      {
        "version": "10.0.12"
      },
      true
    ],
    [
      "$vgt/$vlt - fail $vlt - major",
      {
        "version": {
          "$vgt": "30.0.0",
          "$vlt": "50.0.0"
        }
      },
      {
        "version": "60.0.0"
      },
      false
    ],
    [
      "$vgt/$vlt - fail $vlt - minor",
      {
        "version": {
          "$vgt": "10.30.0",
          "$vlt": "10.50.0"
        }
      },
      {
        "version": "10.60.0"
      },
      false
    ],
    [
      "$vgt/$vlt - fail $vlt - patch",
      {
        "version": {
          "$vgt": "10.2.30",
          "$vlt": "10.2.50"
        }
      },
      {
        "version": "10.2.60"
      },
      false
    ],
    [
      "$vgt/$vlt - fail $vgt - major",
      {
        "version": {
          "$vgt": "30.0.16",
          "$vlt": "50.0.16"
        }
      },
      {
        "version": "20.0.16"
      },
      false
    ],
    [
      "$vgt/$vlt - fail $vgt - minor",
      {
        "version": {
          "$vgt": "10.30.0",
          "$vlt": "10.50.0"
        }
      },
      {
        "version": "10.20.0"
      },
      false
    ],
    [
      "$vgt/$vlt - fail $vgt - patch",
      {
        "version": {
          "$vgt": "10.30.10",
          "$vlt": "10.30.20"
        }
      },
      {
        "version": "10.30.2"
      },
      false
    ],
    [
      "$vgte/$vlte - pass $vgte - major",
      {
        "version": {
          "$vgte": "30.1.2",
          "$vlte": "60.1.2"
        }
      },
      {
        "version": "30.1.2"
      },
      true
    ],
    [
      "$vgte/$vlte - pass $vgte - minor",
      {
        "version": {
          "$vgte": "5.30.2",
          "$vlte": "5.60.2"
        }
      },
      {
        "version": "5.30.2"
      },
      true
    ],
    [
      "$vgte/$vlte - pass $vgte - patch",
      {
        "version": {
          "$vgte": "5.10.30",
          "$vlte": "5.10.60"
        }
      },
      {
        "version": "5.10.30"
      },
      true
    ],
    [
      "$vgte/$vlte - pass $vlte - major",
      {
        "version": {
          "$vgte": "30.1.2",
          "$vlte": "60.1.2"
        }
      },
      {
        "version": "60.1.2"
      },
      true
    ],
    [
      "$vgte/$vlte - pass $vlte - minor",
      {
        "version": {
          "$vgte": "1.30.2",
          "$vlte": "1.60.2"
        }
      },
      {
        "version": "1.60.2"
      },
      true
    ],
    [
      "$vgte/$vlte - pass $vlte - patch",
      {
        "version": {
          "$vgte": "1.2.30",
          "$vlte": "1.2.60"
        }
      },
      {
        "version": "1.2.60"
      },
      true
    ],
    [
      "$vgte/$vlte - fail $vlte - major",
      {
        "version": {
          "$vgte": "30.1.2",
          "$vlte": "60.1.2"
        }
      },
      {
        "version": "61.1.2"
      },
      false
    ],
    [
      "$vgte/$vlte - fail $vgt - minor",
      {
        "version": {
          "$vgte": "30.1.2",
          "$vlte": "60.1.2"
        }
      },
      {
        "version": "29.1.2"
      },
      false
    ],
    [
      "$vgte/$vlte - fail $vgt - patch",
      {
        "version": {
          "$vgte": "1.2.30",
          "$vlte": "1.2.60"
        }
      },
      {
        "version": "1.2.29"
      },
      false
    ],
    [
      "$vgt/$vlt prerelease - fail $vgt",
      {
        "v": {
          "$vgt": "1.0.0-alpha",
          "$vlt": "1.0.0-beta"
        }
      },
      {
        "v": "1.0.0-alpha"
      },
      false
    ],
    [
      "$vgt/$vlt prerelease  w/ multiple fields - fail $vgt",
      {
        "v": {
          "$vgt": "1.0.0-alpha.2",
          "$vlt": "1.0.0-beta.1"
        }
      },
      {
        "v": "1.0.0-alpha.1"
      },
      false
    ],
    [
      "$vgt/$vlt prerelease - fail $vlt",
      {
        "v": {
          "$vgt": "1.0.0-alpha",
          "$vlt": "1.0.0-beta"
        }
      },
      {
        "v": "1.0.0-beta"
      },
      false
    ],
    [
      "$vgt/$vlt prerelease - pass",
      {
        "v": {
          "$vgt": "1.0.0-alpha",
          "$vlt": "1.0.0-beta"
        }
      },
      {
        "v": "1.0.0-alpha.10"
      },
      true
    ],
    [
      "$vgt/$vlt prerelease - fail uppercase",
      {
        "v": {
          "$vgt": "1.0.0-alpha",
          "$vlt": "1.0.0-beta"
        }
      },
      {
        "v": "1.0.0-ALPHA"
      },
      false
    ],
    [
      "$veq - pass",
      {
        "v": {
          "$veq": "1.2.3"
        }
      },
      {
        "v": "1.2.3"
      },
      true
    ],
    [
      "$veq - pass (with build)",
      {
        "v": {
          "$veq": "1.2.3"
        }
      },
      {
        "v": "1.2.3+build.abc.123"
      },
      true
    ],
    [
      "$vne - pass",
      {
        "v": {
          "$vne": "1.2.3"
        }
      },
      {
        "v": "2.2.3"
      },
      true
    ],
    [
      "$vne - pass (prerelease)",
      {
        "v": {
          "$vne": "1.2.3"
        }
      },
      {
        "v": "1.2.3-alpha"
      },
      true
    ],
    [
      "version 0.9.99 < 1.0.0",
      {
        "version": {
          "$vlt": "1.0.0"
        }
      },
      {
        "version": "0.9.99"
      },
      true
    ],
    [
      "version 0.9.0 < 0.10.0",
      {
        "version": {
          "$vlt": "0.10.0"
        }
      },
      {
        "version": "0.9.0"
      },
      true
    ],
    [
      "version 1.0.0-0.0 < 1.0.0-0.0.0",
      {
        "version": {
          "$vlt": "1.0.0-0.0.0"
        }
      },
      {
        "version": "1.0.0-0.0"
      },
      true
    ],
    [
      "version 1.0.0-9999 < 1.0.0--",
      {
        "version": {
          "$vlt": "1.0.0--"
        }
      },
      {
        "version": "1.0.0-9999"
      },
      true
    ],
    [
      "version 1.0.0-99 < 1.0.0-100",
      {
        "version": {
          "$vlt": "1.0.0-100"
        }
      },
      {
        "version": "1.0.0-99"
      },
      true
    ],
    [
      "version 1.0.0-alpha < 1.0.0-alpha.1",
      {
        "version": {
          "$vlt": "1.0.0-alpha.1"
        }
      },
      {
        "version": "1.0.0-alpha"
      },
      true
    ],
    [
      "version 1.0.0-alpha.1 < 1.0.0-alpha.beta",
      {
        "version": {
          "$vlt": "1.0.0-alpha.beta"
        }
      },
      {
        "version": "1.0.0-alpha.1"
      },
      true
    ],
    [
      "version 1.0.0-alpha.beta < 1.0.0-beta",
      {
        "version": {
          "$vlt": "1.0.0-beta"
        }
      },
      {
        "version": "1.0.0-alpha.beta"
      },
      true
    ],
    [
      "version 1.0.0-beta < 1.0.0-beta.2",
      {
        "version": {
          "$vlt": "1.0.0-beta.2"
        }
      },
      {
        "version": "1.0.0-beta"
      },
      true
    ],
    [
      "version 1.0.0-beta.2 < 1.0.0-beta.11",
      {
        "version": {
          "$vlt": "1.0.0-beta.11"
        }
      },
      {
        "version": "1.0.0-beta.2"
      },
      true
    ],
    [
      "version 1.0.0-beta.11 < 1.0.0-rc.1",
      {
        "version": {
          "$vlt": "1.0.0-rc.1"
        }
      },
      {
        "version": "1.0.0-beta.11"
      },
      true
    ],
    [
      "version 1.0.0-rc.1 < 1.0.0",
      {
        "version": {
          "$vlt": "1.0.0"
        }
      },
      {
        "version": "1.0.0-rc.1"
      },
      true
    ],
    [
      "version 1.0.0-0 < 1.0.0--1",
      {
        "version": {
          "$vlt": "1.0.0--1"
        }
      },
      {
        "version": "1.0.0-0"
      },
      true
    ],
    [
      "version 1.0.0-0 < 1.0.0-1",
      {
        "version": {
          "$vlt": "1.0.0-1"
        }
      },
      {
        "version": "1.0.0-0"
      },
      true
    ],
    [
      "version 1.0.0-1.0 < 1.0.0-1.-1",
      {
        "version": {
          "$vlt": "1.0.0-1.-1"
        }
      },
      {
        "version": "1.0.0-1.0"
      },
      true
    ],
    [
      "version 1.2.3-a.b.c < 1.2.3-a.b.c.d",
      {
        "version": {
          "$vlt": "1.2.3-a.b.c.d"
        }
      },
      {
        "version": "1.2.3-a.b.c"
      },
      true
    ],
    [
      "version 0.0.0 > 0.0.0-foo",
      {
        "version": {
          "$vgt": "0.0.0-foo"
        }
      },
      {
        "version": "0.0.0"
      },
      true
    ],
    [
      "version 0.0.1 > 0.0.0",
      {
        "version": {
          "$vgt": "0.0.0"
        }
      },
      {
        "version": "0.0.1"
      },
      true
    ],
    [
      "version 1.0.0 > 0.9.9",
      {
        "version": {
          "$vgt": "0.9.9"
        }
      },
      {
        "version": "1.0.0"
      },
      true
    ],
    [
      "version 0.10.0 > 0.9.0",
      {
        "version": {
          "$vgt": "0.9.0"
        }
      },
      {
        "version": "0.10.0"
      },
      true
    ],
    [
      "version 0.99.0 > 0.10.0",
      {
        "version": {
          "$vgt": "0.10.0"
        }
      },
      {
        "version": "0.99.0"
      },
      true
    ],
    [
      "version 2.0.0 > 1.2.3",
      {
        "version": {
          "$vgt": "1.2.3"
        }
      },
      {
        "version": "2.0.0"
      },
      true
    ],
    [
      "version v0.0.0 > 0.0.0-foo",
      {
        "version": {
          "$vgt": "0.0.0-foo"
        }
      },
      {
        "version": "v0.0.0"
      },
      true
    ],
    [
      "version v0.0.1 > 0.0.0",
      {
        "version": {
          "$vgt": "0.0.0"
        }
      },
      {
        "version": "v0.0.1"
      },
      true
    ],
    [
      "version v1.0.0 > 0.9.9",
      {
        "version": {
          "$vgt": "0.9.9"
        }
      },
      {
        "version": "v1.0.0"
      },
      true
    ],
    [
      "version v0.10.0 > 0.9.0",
      {
        "version": {
          "$vgt": "0.9.0"
        }
      },
      {
        "version": "v0.10.0"
      },
      true
    ],
    [
      "version v0.99.0 > 0.10.0",
      {
        "version": {
          "$vgt": "0.10.0"
        }
      },
      {
        "version": "v0.99.0"
      },
      true
    ],
    [
      "version v2.0.0 > 1.2.3",
      {
        "version": {
          "$vgt": "1.2.3"
        }
      },
      {
        "version": "v2.0.0"
      },
      true
    ],
    [
      "version 0.0.0 > v0.0.0-foo",
      {
        "version": {
          "$vgt": "v0.0.0-foo"
        }
      },
      {
        "version": "0.0.0"
      },
      true
    ],
    [
      "version 0.0.1 > v0.0.0",
      {
        "version": {
          "$vgt": "v0.0.0"
        }
      },
      {
        "version": "0.0.1"
      },
      true
    ],
    [
      "version 1.0.0 > v0.9.9",
      {
        "version": {
          "$vgt": "v0.9.9"
        }
      },
      {
        "version": "1.0.0"
      },
      true
    ],
    [
      "version 0.10.0 > v0.9.0",
      {
        "version": {
          "$vgt": "v0.9.0"
        }
      },
      {
        "version": "0.10.0"
      },
      true
    ],
    [
      "version 0.99.0 > v0.10.0",
      {
        "version": {
          "$vgt": "v0.10.0"
        }
      },
      {
        "version": "0.99.0"
      },
      true
    ],
    [
      "version 2.0.0 > v1.2.3",
      {
        "version": {
          "$vgt": "v1.2.3"
        }
      },
      {
        "version": "2.0.0"
      },
      true
    ],
    [
      "version 1.2.3 > 1.2.3-asdf",
      {
        "version": {
          "$vgt": "1.2.3-asdf"
        }
      },
      {
        "version": "1.2.3"
      },
      true
    ],
    [
      "version 1.2.3 > 1.2.3-4",
      {
        "version": {
          "$vgt": "1.2.3-4"
        }
      },
      {
        "version": "1.2.3"
      },
      true
    ],
    [
      "version 1.2.3 > 1.2.3-4-foo",
      {
        "version": {
          "$vgt": "1.2.3-4-foo"
        }
      },
      {
        "version": "1.2.3"
      },
      true
    ],
    [
      "version 1.2.3-5-foo > 1.2.3-5",
      {
        "version": {
          "$vgt": "1.2.3-5"
        }
      },
      {
        "version": "1.2.3-5-foo"
      },
      true
    ],
    [
      "version 1.2.3-5 > 1.2.3-4",
      {
        "version": {
          "$vgt": "1.2.3-4"
        }
      },
      {
        "version": "1.2.3-5"
      },
      true
    ],
    [
      "version 1.2.3-5-foo > 1.2.3-5-Foo",
      {
        "version": {
          "$vgt": "1.2.3-5-Foo"
        }
      },
      {
        "version": "1.2.3-5-foo"
      },
      true
    ],
    [
      "version 3.0.0 > 2.7.2+asdf",
      {
        "version": {
          "$vgt": "2.7.2+asdf"
        }
      },
      {
        "version": "3.0.0"
      },
      true
    ],
    [
      "version 1.2.3-a.10 > 1.2.3-a.5",
      {
        "version": {
          "$vgt": "1.2.3-a.5"
        }
      },
      {
        "version": "1.2.3-a.10"
      },
      true
    ],
    [
      "version 1.2.3-a.b > 1.2.3-a.5",
      {
        "version": {
          "$vgt": "1.2.3-a.5"
        }
      },
      {
        "version": "1.2.3-a.b"
      },
      true
    ],
    [
      "version 1.2.3-a.b > 1.2.3-a",
      {
        "version": {
          "$vgt": "1.2.3-a"
        }
      },
      {
        "version": "1.2.3-a.b"
      },
      true
    ],
    [
      "version 1.2.3-a.b.c.10.d.5 > 1.2.3-a.b.c.5.d.100",
      {
        "version": {
          "$vgt": "1.2.3-a.b.c.5.d.100"
        }
      },
      {
        "version": "1.2.3-a.b.c.10.d.5"
      },
      true
    ],
    [
      "version 1.2.3-r2 > 1.2.3-r100",
      {
        "version": {
          "$vgt": "1.2.3-r100"
        }
      },
      {
        "version": "1.2.3-r2"
      },
      true
    ],
    [
      "version 1.2.3-r100 > 1.2.3-R2",
      {
        "version": {
          "$vgt": "1.2.3-R2"
        }
      },
      {
        "version": "1.2.3-r100"
      },
      true
    ],
    [
      "version a.b.c.d.e.f > 1.2.3",
      {
        "version": {
          "$vgt": "1.2.3"
        }
      },
      {
        "version": "a.b.c.d.e.f"
      },
      true
    ],
    [
      "version 10.0.0 > 9.0.0",
      {
        "version": {
          "$vgt": "9.0.0"
        }
      },
      {
        "version": "10.0.0"
      },
      true
    ],
    [
      "version 10000.0.0 > 9999.0.0",
      {
        "version": {
          "$vgt": "9999.0.0"
        }
      },
      {
        "version": "10000.0.0"
      },
      true
    ],
    [
      "version 1.2.3 == 1.2.3",
      {
        "version": {
          "$veq": "1.2.3"
        }
      },
      {
        "version": "1.2.3"
      },
      true
    ],
    [
      "version 1.2.3 == v1.2.3",
      {
        "version": {
          "$veq": "v1.2.3"
        }
      },
      {
        "version": "1.2.3"
      },
      true
    ],
    [
      "version 1.2.3-0 == v1.2.3-0",
      {
        "version": {
          "$veq": "v1.2.3-0"
        }
      },
      {
        "version": "1.2.3-0"
      },
      true
    ],
    [
      "version 1.2.3-1 == 1.2.3-1",
      {
        "version": {
          "$veq": "1.2.3-1"
        }
      },
      {
        "version": "1.2.3-1"
      },
      true
    ],
    [
      "version 1.2.3-1 == v1.2.3-1",
      {
        "version": {
          "$veq": "v1.2.3-1"
        }
      },
      {
        "version": "1.2.3-1"
      },
      true
    ],
    [
      "version 1.2.3-beta == 1.2.3-beta",
      {
        "version": {
          "$veq": "1.2.3-beta"
        }
      },
      {
        "version": "1.2.3-beta"
      },
      true
    ],
    [
      "version 1.2.3-beta == v1.2.3-beta",
      {
        "version": {
          "$veq": "v1.2.3-beta"
        }
      },
      {
        "version": "1.2.3-beta"
      },
      true
    ],
    [
      "version 1.2.3-beta+build == 1.2.3-beta+otherbuild",
      {
        "version": {
          "$veq": "1.2.3-beta+otherbuild"
        }
      },
      {
        "version": "1.2.3-beta+build"
      },
      true
    ],
    [
      "version 1.2.3-beta+build == v1.2.3-beta+otherbuild",
      {
        "version": {
          "$veq": "v1.2.3-beta+otherbuild"
        }
      },
      {
        "version": "1.2.3-beta+build"
      },
      true
    ],
    [
      "version 1-2-3 == 1.2.3",
      {
        "version": {
          "$veq": "1.2.3"
        }
      },
      {
        "version": "1-2-3"
      },
      true
    ],
    [
      "version 1-2-3 == 1-2.3+build99",
      {
        "version": {
          "$veq": "1-2.3+build99"
        }
      },
      {
        "version": "1-2-3"
      },
      true
    ],
    [
      "version 1-2-3 == v1.2.3",
      {
        "version": {
          "$veq": "v1.2.3"
        }
      },
      {
        "version": "1-2-3"
      },
      true
    ],
    [
      "version 1.2.3.4 == 1.2.3-4",
      {
        "version": {
          "$veq": "1.2.3-4"
        }
      },
      {
        "version": "1.2.3.4"
      },
      true
    ]
  ],
  "hash": [
    ["", "a", 1, 0.22],
    ["", "b", 1, 0.077],
    ["b", "a", 1, 0.946],
    ["ef", "d", 1, 0.652],
    ["asdf", "8952klfjas09ujk", 1, 0.549],
    ["", "123", 1, 0.011],
    ["", "___)((*\":&", 1, 0.563],
    ["seed", "a", 2, 0.0505],
    ["seed", "b", 2, 0.2696],
    ["foo", "ab", 2, 0.2575],
    ["foo", "def", 2, 0.2019],
    ["89123klj", "8952klfjas09ujkasdf", 2, 0.124],
    ["90850943850283058242805", "123", 2, 0.7516],
    ["()**(%$##$%#$#", "___)((*\":&", 2, 0.0128],
    ["abc", "def", 99, null]
  ],
  "getBucketRange": [
    [
      "normal 50/50",
      [2, 1, null],
      [
        [0, 0.5],
        [0.5, 1]
      ]
    ],
    [
      "reduced coverage",
      [2, 0.5, null],
      [
        [0, 0.25],
        [0.5, 0.75]
      ]
    ],
    [
      "zero coverage",
      [2, 0, null],
      [
        [0, 0],
        [0.5, 0.5]
      ]
    ],
    [
      "4 variations",
      [4, 1, null],
      [
        [0, 0.25],
        [0.25, 0.5],
        [0.5, 0.75],
        [0.75, 1]
      ]
    ],
    [
      "uneven weights",
      [2, 1, [0.4, 0.6]],
      [
        [0, 0.4],
        [0.4, 1]
      ]
    ],
    [
      "uneven weights, 3 variations",
      [3, 1, [0.2, 0.3, 0.5]],
      [
        [0, 0.2],
        [0.2, 0.5],
        [0.5, 1]
      ]
    ],
    [
      "uneven weights, reduced coverage, 3 variations",
      [3, 0.2, [0.2, 0.3, 0.5]],
      [
        [0, 0.04],
        [0.2, 0.26],
        [0.5, 0.6]
      ]
    ],
    [
      "negative coverage",
      [2, -0.2, null],
      [
        [0, 0],
        [0.5, 0.5]
      ]
    ],
    [
      "coverage above 1",
      [2, 1.5, null],
      [
        [0, 0.5],
        [0.5, 1]
      ]
    ],
    [
      "weights sum below 1",
      [2, 1, [0.4, 0.1]],
      [
        [0, 0.5],
        [0.5, 1]
      ]
    ],
    [
      "weights sum above 1",
      [2, 1, [0.7, 0.6]],
      [
        [0, 0.5],
        [0.5, 1]
      ]
    ],
    [
      "weights.length not equal to num variations",
      [4, 1, [0.4, 0.4, 0.2]],
      [
        [0, 0.25],
        [0.25, 0.5],
        [0.5, 0.75],
        [0.75, 1]
      ]
    ],
    [
      "weights sum almost equals 1",
      [2, 1, [0.4, 0.5999]],
      [
        [0, 0.4],
        [0.4, 0.9999]
      ]
    ]
  ],
  "feature": [
    [
      "unknown feature key",
      {},
      "my-feature",
      {
        "value": null,
        "on": false,
        "off": true,
        "source": "unknownFeature"
      }
    ],
    [
      "defaults when empty",
      { "features": { "feature": {} } },
      "feature",
      {
        "value": null,
        "on": false,
        "off": true,
        "source": "defaultValue"
      }
    ],
    [
      "uses defaultValue - number",
      { "features": { "feature": { "defaultValue": 1 } } },
      "feature",
      {
        "value": 1,
        "on": true,
        "off": false,
        "source": "defaultValue"
      }
    ],
    [
      "uses custom values - string",
      { "features": { "feature": { "defaultValue": "yes" } } },
      "feature",
      {
        "value": "yes",
        "on": true,
        "off": false,
        "source": "defaultValue"
      }
    ],
    [
      "force rules",
      {
        "features": {
          "feature": {
            "defaultValue": 2,
            "rules": [
              {
                "force": 1
              }
            ]
          }
        }
      },
      "feature",
      {
        "value": 1,
        "on": true,
        "off": false,
        "source": "force"
      }
    ],
    [
      "force rules - force false",
      {
        "features": {
          "feature": {
            "defaultValue": true,
            "rules": [
              {
                "force": false
              }
            ]
          }
        }
      },
      "feature",
      {
        "value": false,
        "on": false,
        "off": true,
        "source": "force"
      }
    ],
    [
      "force rules - coverage included",
      {
        "attributes": {
          "id": "3"
        },
        "features": {
          "feature": {
            "defaultValue": 2,
            "rules": [
              {
                "force": 1,
                "coverage": 0.5
              }
            ]
          }
        }
      },
      "feature",
      {
        "value": 1,
        "on": true,
        "off": false,
        "source": "force"
      }
    ],
    [
      "force rule - coverage with integer hash attribute",
      {
        "attributes": {
          "id": 3
        },
        "features": {
          "feature": {
            "defaultValue": 2,
            "rules": [
              {
                "force": 1,
                "coverage": 0.5
              }
            ]
          }
        }
      },
      "feature",
      {
        "value": 1,
        "on": true,
        "off": false,
        "source": "force"
      }
    ],
    [
      "force rules - coverage excluded",
      {
        "attributes": {
          "id": "1"
        },
        "features": {
          "feature": {
            "defaultValue": 2,
            "rules": [
              {
                "force": 1,
                "coverage": 0.5
              }
            ]
          }
        }
      },
      "feature",
      {
        "value": 2,
        "on": true,
        "off": false,
        "source": "defaultValue"
      }
    ],
    [
      "force rules - coverage missing hashAttribute",
      {
        "attributes": {},
        "features": {
          "feature": {
            "defaultValue": 2,
            "rules": [
              {
                "force": 1,
                "coverage": 0.5
              }
            ]
          }
        }
      },
      "feature",
      {
        "value": 2,
        "on": true,
        "off": false,
        "source": "defaultValue"
      }
    ],
    [
      "force rules - coverage 0",
      {
        "attributes": {
          "id": "d0bc0a5a"
        },
        "features": {
          "8d156": {
            "defaultValue": 0,
            "rules": [
              {
                "force": 1,
                "coverage": 0,
                "hashVersion": 2
              }
            ]
          }
        }
      },
      "8d156",
      {
        "value": 0,
        "on": false,
        "off": true,
        "source": "defaultValue"
      }
    ],
    [
      "force rules - condition pass",
      {
        "attributes": {
          "country": "US",
          "browser": "firefox"
        },
        "features": {
          "feature": {
            "defaultValue": 2,
            "rules": [
              {
                "force": 1,
                "condition": {
                  "country": { "$in": ["US", "CA"] },
                  "browser": "firefox"
                }
              }
            ]
          }
        }
      },
      "feature",
      {
        "value": 1,
        "on": true,
        "off": false,
        "source": "force"
      }
    ],
    [
      "force rules - condition fail",
      {
        "attributes": {
          "country": "US",
          "browser": "chrome"
        },
        "features": {
          "feature": {
            "defaultValue": 2,
            "rules": [
              {
                "force": 1,
                "condition": {
                  "country": { "$in": ["US", "CA"] },
                  "browser": "firefox"
                }
              }
            ]
          }
        }
      },
      "feature",
      {
        "value": 2,
        "on": true,
        "off": false,
        "source": "defaultValue"
      }
    ],
    [
      "force rules - coverage with bad hash version",
      {
        "attributes": {
          "id": "1"
        },
        "features": {
          "feature": {
            "defaultValue": 2,
            "rules": [
              {
                "force": 1,
                "coverage": 1.0,
                "hashVersion": 99
              }
            ]
          }
        }
      },
      "feature",
      {
        "value": 2,
        "on": true,
        "off": false,
        "source": "defaultValue"
      }
    ],
    [
      "ignores empty rules",
      {
        "features": {
          "feature": {
            "rules": [{}]
          }
        }
      },
      "feature",
      {
        "value": null,
        "on": false,
        "off": true,
        "source": "defaultValue"
      }
    ],
    [
      "empty experiment rule - c",
      {
        "attributes": {
          "id": "123"
        },
        "features": {
          "feature": {
            "rules": [
              {
                "variations": ["a", "b", "c"]
              }
            ]
          }
        }
      },
      "feature",
      {
        "value": "c",
        "on": true,
        "off": false,
        "experiment": {
          "key": "feature",
          "variations": ["a", "b", "c"]
        },
        "experimentResult": {
          "featureId": "feature",
          "value": "c",
          "variationId": 2,
          "inExperiment": true,
          "hashUsed": true,
          "hashAttribute": "id",
          "hashValue": "123",
          "bucket": 0.863,
          "key": "2",
          "stickyBucketUsed": false
        },
        "source": "experiment"
      }
    ],
    [
      "empty experiment rule - a",
      {
        "attributes": {
          "id": "456"
        },
        "features": {
          "feature": {
            "rules": [
              {
                "variations": ["a", "b", "c"]
              }
            ]
          }
        }
      },
      "feature",
      {
        "value": "a",
        "on": true,
        "off": false,
        "experiment": {
          "key": "feature",
          "variations": ["a", "b", "c"]
        },
        "experimentResult": {
          "featureId": "feature",
          "value": "a",
          "variationId": 0,
          "inExperiment": true,
          "hashUsed": true,
          "hashAttribute": "id",
          "hashValue": "456",
          "bucket": 0.178,
          "key": "0",
          "stickyBucketUsed": false
        },
        "source": "experiment"
      }
    ],
    [
      "empty experiment rule - b",
      {
        "attributes": {
          "id": "fds"
        },
        "features": {
          "feature": {
            "rules": [
              {
                "variations": ["a", "b", "c"]
              }
            ]
          }
        }
      },
      "feature",
      {
        "value": "b",
        "on": true,
        "off": false,
        "experiment": {
          "key": "feature",
          "variations": ["a", "b", "c"]
        },
        "experimentResult": {
          "featureId": "feature",
          "value": "b",
          "variationId": 1,
          "inExperiment": true,
          "hashUsed": true,
          "hashAttribute": "id",
          "hashValue": "fds",
          "bucket": 0.514,
          "key": "1",
          "stickyBucketUsed": false
        },
        "source": "experiment"
      }
    ],
    [
      "creates experiments properly",
      {
        "attributes": {
          "anonId": "123",
          "premium": true
        },
        "features": {
          "feature": {
            "rules": [
              {
                "coverage": 0.99,
                "hashAttribute": "anonId",
                "seed": "feature",
                "hashVersion": 2,
                "name": "Test",
                "phase": "1",
                "ranges": [
                  [0, 0.1],
                  [0.1, 1.0]
                ],
                "meta": [
                  {
                    "key": "v0",
                    "name": "variation 0"
                  },
                  {
                    "key": "v1",
                    "name": "variation 1"
                  }
                ],
                "filters": [
                  {
                    "attribute": "anonId",
                    "seed": "pricing",
                    "ranges": [[0, 1]]
                  }
                ],
                "namespace": ["pricing", 0, 1],
                "key": "hello",
                "variations": [true, false],
                "weights": [0.1, 0.9],
                "condition": { "premium": true }
              }
            ]
          }
        }
      },
      "feature",
      {
        "value": false,
        "on": false,
        "off": true,
        "source": "experiment",
        "experiment": {
          "coverage": 0.99,
          "ranges": [
            [0, 0.1],
            [0.1, 1.0]
          ],
          "meta": [
            {
              "key": "v0",
              "name": "variation 0"
            },
            {
              "key": "v1",
              "name": "variation 1"
            }
          ],
          "filters": [
            {
              "attribute": "anonId",
              "seed": "pricing",
              "ranges": [[0, 1]]
            }
          ],
          "name": "Test",
          "phase": "1",
          "seed": "feature",
          "hashVersion": 2,
          "hashAttribute": "anonId",
          "namespace": ["pricing", 0, 1],
          "key": "hello",
          "variations": [true, false],
          "weights": [0.1, 0.9],
          "condition": { "premium": true }
        },
        "experimentResult": {
          "featureId": "feature",
          "value": false,
          "variationId": 1,
          "inExperiment": true,
          "hashUsed": true,
          "hashAttribute": "anonId",
          "hashValue": "123",
          "bucket": 0.5231,
          "key": "v1",
          "name": "variation 1",
          "stickyBucketUsed": false
        }
      }
    ],
    [
      "rule orders - skip 1",
      {
        "attributes": {
          "browser": "firefox"
        },
        "features": {
          "feature": {
            "defaultValue": 0,
            "rules": [
              {
                "force": 1,
                "condition": { "browser": "chrome" }
              },
              {
                "force": 2,
                "condition": { "browser": "firefox" }
              },
              {
                "force": 3,
                "condition": { "browser": "safari" }
              }
            ]
          }
        }
      },
      "feature",
      {
        "value": 2,
        "on": true,
        "off": false,
        "source": "force"
      }
    ],
    [
      "rule orders - skip 1,2",
      {
        "attributes": {
          "browser": "safari"
        },
        "features": {
          "feature": {
            "defaultValue": 0,
            "rules": [
              {
                "force": 1,
                "condition": { "browser": "chrome" }
              },
              {
                "force": 2,
                "condition": { "browser": "firefox" }
              },
              {
                "force": 3,
                "condition": { "browser": "safari" }
              }
            ]
          }
        }
      },
      "feature",
      {
        "value": 3,
        "on": true,
        "off": false,
        "source": "force"
      }
    ],
    [
      "rule orders - skip all",
      {
        "attributes": {
          "browser": "ie"
        },
        "features": {
          "feature": {
            "defaultValue": 0,
            "rules": [
              {
                "force": 1,
                "condition": { "browser": "chrome" }
              },
              {
                "force": 2,
                "condition": { "browser": "firefox" }
              },
              {
                "force": 3,
                "condition": { "browser": "safari" }
              }
            ]
          }
        }
      },
      "feature",
      {
        "value": 0,
        "on": false,
        "off": true,
        "source": "defaultValue"
      }
    ],
    [
      "skips experiment on coverage",
      {
        "attributes": { "id": "123" },
        "features": {
          "feature": {
            "defaultValue": 0,
            "rules": [
              {
                "variations": [0, 1, 2, 3],
                "coverage": 0.01
              },
              {
                "force": 3
              }
            ]
          }
        }
      },
      "feature",
      {
        "value": 3,
        "on": true,
        "off": false,
        "source": "force"
      }
    ],
    [
      "skips experiment on namespace",
      {
        "attributes": { "id": "123" },
        "features": {
          "feature": {
            "defaultValue": 0,
            "rules": [
              {
                "variations": [0, 1, 2, 3],
                "namespace": ["pricing", 0, 0.01]
              },
              {
                "force": 3
              }
            ]
          }
        }
      },
      "feature",
      {
        "value": 3,
        "on": true,
        "off": false,
        "source": "force"
      }
    ],
    [
      "handles integer hashAttribute",
      {
        "attributes": { "id": 123 },
        "features": {
          "feature": {
            "defaultValue": 0,
            "rules": [
              {
                "variations": [0, 1]
              }
            ]
          }
        }
      },
      "feature",
      {
        "value": 1,
        "on": true,
        "off": false,
        "source": "experiment",
        "experiment": {
          "key": "feature",
          "variations": [0, 1]
        },
        "experimentResult": {
          "featureId": "feature",
          "hashAttribute": "id",
          "hashValue": 123,
          "hashUsed": true,
          "inExperiment": true,
          "value": 1,
          "variationId": 1,
          "key": "1",
          "bucket": 0.863,
          "stickyBucketUsed": false
        }
      }
    ],
    [
      "skip experiment on missing hashAttribute",
      {
        "attributes": { "id": "123" },
        "features": {
          "feature": {
            "defaultValue": 0,
            "rules": [
              {
                "variations": [0, 1, 2, 3],
                "hashAttribute": "company"
              },
              {
                "force": 3
              }
            ]
          }
        }
      },
      "feature",
      {
        "value": 3,
        "on": true,
        "off": false,
        "source": "force"
      }
    ],
    [
      "include experiments when forced",
      {
        "attributes": { "id": "123" },
        "forcedVariations": {
          "feature": 1
        },
        "features": {
          "feature": {
            "defaultValue": 0,
            "rules": [
              {
                "variations": [0, 1, 2, 3]
              },
              {
                "force": 3
              }
            ]
          }
        }
      },
      "feature",
      {
        "value": 1,
        "on": true,
        "off": false,
        "source": "experiment",
        "experiment": {
          "key": "feature",
          "variations": [0, 1, 2, 3]
        },
        "experimentResult": {
          "featureId": "feature",
          "value": 1,
          "variationId": 1,
          "inExperiment": true,
          "hashUsed": false,
          "hashAttribute": "id",
          "hashValue": "123",
          "key": "1",
          "stickyBucketUsed": false
        }
      }
    ],
    [
      "Force rule with range, ignores coverage",
      {
        "attributes": {
          "id": "1"
        },
        "features": {
          "feature": {
            "defaultValue": 0,
            "rules": [
              {
                "force": 2,
                "coverage": 0.01,
                "range": [0, 0.99]
              }
            ]
          }
        }
      },
      "feature",
      {
        "value": 2,
        "on": true,
        "off": false,
        "source": "force"
      }
    ],
    [
      "Force rule, hash version 2",
      {
        "attributes": {
          "id": "1"
        },
        "features": {
          "feature": {
            "defaultValue": 0,
            "rules": [
              {
                "force": 2,
                "hashVersion": 2,
                "range": [0.96, 0.97]
              }
            ]
          }
        }
      },
      "feature",
      {
        "value": 2,
        "on": true,
        "off": false,
        "source": "force"
      }
    ],
    [
      "Force rule, skip due to range",
      {
        "attributes": {
          "id": "1"
        },
        "features": {
          "feature": {
            "defaultValue": 0,
            "rules": [
              {
                "force": 2,
                "range": [0, 0.01]
              }
            ]
          }
        }
      },
      "feature",
      {
        "value": 0,
        "on": false,
        "off": true,
        "source": "defaultValue"
      }
    ],
    [
      "Force rule, skip due to filter",
      {
        "attributes": {
          "id": "1"
        },
        "features": {
          "feature": {
            "defaultValue": 0,
            "rules": [
              {
                "force": 2,
                "filters": [
                  {
                    "seed": "seed",
                    "ranges": [[0, 0.01]]
                  }
                ]
              }
            ]
          }
        }
      },
      "feature",
      {
        "value": 0,
        "on": false,
        "off": true,
        "source": "defaultValue"
      }
    ],
    [
      "Force rule, use seed with range",
      {
        "attributes": {
          "id": "1"
        },
        "features": {
          "feature": {
            "defaultValue": 0,
            "rules": [
              {
                "force": 2,
                "range": [0, 0.5],
                "seed": "fjdslafdsa",
                "hashVersion": 2
              }
            ]
          }
        }
      },
      "feature",
      {
        "value": 2,
        "on": true,
        "off": false,
        "source": "force"
      }
    ],
    [
      "Support passthrough variations",
      {
        "attributes": {
          "id": "1"
        },
        "features": {
          "feature": {
            "defaultValue": 0,
            "rules": [
              {
                "key": "holdout",
                "variations": [1, 2],
                "hashVersion": 2,
                "ranges": [
                  [0, 0.01],
                  [0.01, 1.0]
                ],
                "meta": [
                  {},
                  {
                    "passthrough": true
                  }
                ]
              },
              {
                "key": "experiment",
                "variations": [3, 4],
                "hashVersion": 2,
                "ranges": [
                  [0, 0.5],
                  [0.5, 1.0]
                ]
              }
            ]
          }
        }
      },
      "feature",
      {
        "value": 3,
        "on": true,
        "off": false,
        "source": "experiment",
        "experiment": {
          "key": "experiment",
          "hashVersion": 2,
          "variations": [3, 4],
          "ranges": [
            [0, 0.5],
            [0.5, 1.0]
          ]
        },
        "experimentResult": {
          "featureId": "feature",
          "hashAttribute": "id",
          "hashUsed": true,
          "hashValue": "1",
          "inExperiment": true,
          "key": "0",
          "value": 3,
          "variationId": 0,
          "bucket": 0.4413,
          "stickyBucketUsed": false
        }
      }
    ],
    [
      "Support holdout groups",
      {
        "attributes": {
          "id": "1"
        },
        "features": {
          "feature": {
            "defaultValue": 0,
            "rules": [
              {
                "key": "holdout",
                "hashVersion": 2,
                "variations": [1, 2],
                "ranges": [
                  [0, 0.99],
                  [0.99, 1.0]
                ],
                "meta": [
                  {},
                  {
                    "passthrough": true
                  }
                ]
              },
              {
                "key": "experiment",
                "hashVersion": 2,
                "variations": [3, 4],
                "ranges": [
                  [0, 0.5],
                  [0.5, 1.0]
                ]
              }
            ]
          }
        }
      },
      "feature",
      {
        "value": 1,
        "on": true,
        "off": false,
        "source": "experiment",
        "experiment": {
          "hashVersion": 2,
          "ranges": [
            [0, 0.99],
            [0.99, 1.0]
          ],
          "meta": [
            {},
            {
              "passthrough": true
            }
          ],
          "key": "holdout",
          "variations": [1, 2]
        },
        "experimentResult": {
          "featureId": "feature",
          "hashAttribute": "id",
          "hashUsed": true,
          "hashValue": "1",
          "inExperiment": true,
          "key": "0",
          "value": 1,
          "variationId": 0,
          "bucket": 0.8043,
          "stickyBucketUsed": false
        }
      }
    ],
    [
      "Prerequisite flag off, block dependent flag",
      {
        "attributes": {
          "id": "123",
          "memberType": "basic",
          "country": "Canada"
        },
        "features": {
          "parentFlag": {
            "defaultValue": "silver",
            "rules": [
              {
                "condition": { "country": "Canada" },
                "force": "red"
              },
              {
                "condition": { "country": { "$in": ["USA", "Mexico"] } },
                "force": "green"
              }
            ]
          },
          "childFlag": {
            "defaultValue": "default",
            "rules": [
              {
                "parentConditions": [
                  {
                    "id": "parentFlag",
                    "condition": { "value": "green" },
                    "gate": true
                  }
                ]
              },
              {
                "condition": { "memberType": "basic" },
                "force": "success"
              }
            ]
          }
        }
      },
      "childFlag",
      {
        "value": null,
        "on": false,
        "off": true,
        "source": "prerequisite"
      }
    ],
    [
      "Prerequisite flag missing, block dependent flag",
      {
        "attributes": {
          "id": "123",
          "memberType": "basic",
          "country": "Canada"
        },
        "features": {
          "childFlag": {
            "defaultValue": "default",
            "rules": [
              {
                "parentConditions": [
                  {
                    "id": "parentFlag",
                    "condition": { "value": "green" },
                    "gate": true
                  }
                ]
              },
              {
                "condition": { "memberType": "basic" },
                "force": "success"
              }
            ]
          }
        }
      },
      "childFlag",
      {
        "value": null,
        "on": false,
        "off": true,
        "source": "prerequisite"
      }
    ],
    [
      "Prerequisite flag on, evaluate dependent flag",
      {
        "attributes": {
          "id": "123",
          "memberType": "basic",
          "country": "USA"
        },
        "features": {
          "parentFlag": {
            "defaultValue": "silver",
            "rules": [
              {
                "condition": { "country": "Canada" },
                "force": "red"
              },
              {
                "condition": { "country": { "$in": ["USA", "Mexico"] } },
                "force": "green"
              }
            ]
          },
          "childFlag": {
            "defaultValue": "default",
            "rules": [
              {
                "parentConditions": [
                  {
                    "id": "parentFlag",
                    "condition": { "value": "green" },
                    "gate": true
                  }
                ]
              },
              {
                "condition": { "memberType": "basic" },
                "force": "success"
              }
            ]
          }
        }
      },
      "childFlag",
      {
        "value": "success",
        "on": true,
        "off": false,
        "source": "force"
      }
    ],
    [
      "Multiple parallel prerequisite flags on, evaluate dependent flag",
      {
        "attributes": {
          "id": "123",
          "memberType": "basic",
          "country": "USA"
        },
        "features": {
          "parentFlag1": {
            "defaultValue": "silver",
            "rules": [
              {
                "condition": { "country": "Canada" },
                "force": "red"
              },
              {
                "condition": { "country": { "$in": ["USA", "Mexico"] } },
                "force": "green"
              }
            ]
          },
          "parentFlag2": {
            "defaultValue": 0,
            "rules": [
              {
                "condition": { "id": "123" },
                "force": 2
              }
            ]
          },
          "childFlag": {
            "defaultValue": "default",
            "rules": [
              {
                "parentConditions": [
                  {
                    "id": "parentFlag1",
                    "condition": { "value": "green" },
                    "gate": true
                  }
                ]
              },
              {
                "parentConditions": [
                  {
                    "id": "parentFlag2",
                    "condition": { "value": { "$gt": 1 } },
                    "gate": true
                  }
                ]
              },
              {
                "condition": { "memberType": "basic" },
                "force": "success"
              }
            ]
          }
        }
      },
      "childFlag",
      {
        "value": "success",
        "on": true,
        "off": false,
        "source": "force"
      }
    ],
    [
      "Multiple nested prerequisite flags on, evaluate dependent flag",
      {
        "attributes": {
          "id": "123",
          "memberType": "basic",
          "country": "USA"
        },
        "features": {
          "parentFlag1": {
            "defaultValue": "silver",
            "rules": [
              {
                "parentConditions": [
                  {
                    "id": "parentFlag2",
                    "condition": { "value": { "$gt": 1 } },
                    "gate": true
                  }
                ]
              },
              {
                "condition": { "country": "Canada" },
                "force": "red"
              },
              {
                "condition": { "country": { "$in": ["USA", "Mexico"] } },
                "force": "green"
              }
            ]
          },
          "parentFlag2": {
            "defaultValue": 0,
            "rules": [
              {
                "condition": { "id": "123" },
                "force": 2
              }
            ]
          },
          "childFlag": {
            "defaultValue": "default",
            "rules": [
              {
                "parentConditions": [
                  {
                    "id": "parentFlag1",
                    "condition": { "value": "green" },
                    "gate": true
                  }
                ]
              },
              {
                "condition": { "memberType": "basic" },
                "force": "success"
              }
            ]
          }
        }
      },
      "childFlag",
      {
        "value": "success",
        "on": true,
        "off": false,
        "source": "force"
      }
    ],
    [
      "Prerequisite experiment flag in target bucket, evaluate dependent flag",
      {
        "attributes": {
          "id": "1234",
          "memberType": "basic",
          "country": "USA"
        },
        "features": {
          "parentExperimentFlag": {
            "defaultValue": 0,
            "rules": [
              {
                "key": "experiment",
                "variations": [0, 1],
                "hashAttribute": "id",
                "hashVersion": 2,
                "ranges": [
                  [0, 0.5],
                  [0.5, 1.0]
                ]
              }
            ]
          },
          "childFlag": {
            "defaultValue": "default",
            "rules": [
              {
                "parentConditions": [
                  {
                    "id": "parentExperimentFlag",
                    "condition": { "value": 1 },
                    "gate": true
                  }
                ]
              },
              {
                "condition": { "memberType": "basic" },
                "force": "success"
              }
            ]
          }
        }
      },
      "childFlag",
      {
        "value": "success",
        "on": true,
        "off": false,
        "source": "force"
      }
    ],
    [
      "Prerequisite cycle detected, break",
      {
        "attributes": {
          "id": "123"
        },
        "features": {
          "flag1": {
            "defaultValue": true,
            "rules": [
              {
                "parentConditions": [
                  {
                    "id": "flag2",
                    "condition": { "value": true },
                    "gate": true
                  }
                ]
              }
            ]
          },
          "flag2": {
            "defaultValue": true,
            "rules": [
              {
                "parentConditions": [
                  {
                    "id": "flag1",
                    "condition": { "value": true },
                    "gate": true
                  }
                ]
              }
            ]
          }
        }
      },
      "flag1",
      {
        "value": null,
        "on": false,
        "off": true,
        "source": "cyclicPrerequisite"
      }
    ]
  ],
  "run": [
    [
      "default weights - 1",
      { "attributes": { "id": "1" } },
      { "key": "my-test", "variations": [0, 1] },
      1,
      true,
      true
    ],
    [
      "default weights - 2",
      { "attributes": { "id": "2" } },
      { "key": "my-test", "variations": [0, 1] },
      0,
      true,
      true
    ],
    [
      "default weights - 3",
      { "attributes": { "id": "3" } },
      { "key": "my-test", "variations": [0, 1] },
      0,
      true,
      true
    ],
    [
      "default weights - 4",
      { "attributes": { "id": "4" } },
      { "key": "my-test", "variations": [0, 1] },
      1,
      true,
      true
    ],
    [
      "default weights - 5",
      { "attributes": { "id": "5" } },
      { "key": "my-test", "variations": [0, 1] },
      1,
      true,
      true
    ],
    [
      "default weights - 6",
      { "attributes": { "id": "6" } },
      { "key": "my-test", "variations": [0, 1] },
      1,
      true,
      true
    ],
    [
      "default weights - 7",
      { "attributes": { "id": "7" } },
      { "key": "my-test", "variations": [0, 1] },
      0,
      true,
      true
    ],
    [
      "default weights - 8",
      { "attributes": { "id": "8" } },
      { "key": "my-test", "variations": [0, 1] },
      1,
      true,
      true
    ],
    [
      "default weights - 9",
      { "attributes": { "id": "9" } },
      { "key": "my-test", "variations": [0, 1] },
      0,
      true,
      true
    ],
    [
      "uneven weights - 1",
      { "attributes": { "id": "1" } },
      { "key": "my-test", "variations": [0, 1], "weights": [0.1, 0.9] },
      1,
      true,
      true
    ],
    [
      "uneven weights - 2",
      { "attributes": { "id": "2" } },
      { "key": "my-test", "variations": [0, 1], "weights": [0.1, 0.9] },
      1,
      true,
      true
    ],
    [
      "uneven weights - 3",
      { "attributes": { "id": "3" } },
      { "key": "my-test", "variations": [0, 1], "weights": [0.1, 0.9] },
      0,
      true,
      true
    ],
    [
      "uneven weights - 4",
      { "attributes": { "id": "4" } },
      { "key": "my-test", "variations": [0, 1], "weights": [0.1, 0.9] },
      1,
      true,
      true
    ],
    [
      "uneven weights - 5",
      { "attributes": { "id": "5" } },
      { "key": "my-test", "variations": [0, 1], "weights": [0.1, 0.9] },
      1,
      true,
      true
    ],
    [
      "uneven weights - 6",
      { "attributes": { "id": "6" } },
      { "key": "my-test", "variations": [0, 1], "weights": [0.1, 0.9] },
      1,
      true,
      true
    ],
    [
      "uneven weights - 7",
      { "attributes": { "id": "7" } },
      { "key": "my-test", "variations": [0, 1], "weights": [0.1, 0.9] },
      0,
      true,
      true
    ],
    [
      "uneven weights - 8",
      { "attributes": { "id": "8" } },
      { "key": "my-test", "variations": [0, 1], "weights": [0.1, 0.9] },
      1,
      true,
      true
    ],
    [
      "uneven weights - 9",
      { "attributes": { "id": "9" } },
      { "key": "my-test", "variations": [0, 1], "weights": [0.1, 0.9] },
      1,
      true,
      true
    ],
    [
      "coverage - 1",
      { "attributes": { "id": "1" } },
      { "key": "my-test", "variations": [0, 1], "coverage": 0.4 },
      0,
      false,
      false
    ],
    [
      "coverage - 2",
      { "attributes": { "id": "2" } },
      { "key": "my-test", "variations": [0, 1], "coverage": 0.4 },
      0,
      true,
      true
    ],
    [
      "coverage - 3",
      { "attributes": { "id": "3" } },
      { "key": "my-test", "variations": [0, 1], "coverage": 0.4 },
      0,
      true,
      true
    ],
    [
      "coverage - 4",
      { "attributes": { "id": "4" } },
      { "key": "my-test", "variations": [0, 1], "coverage": 0.4 },
      0,
      false,
      false
    ],
    [
      "coverage - 5",
      { "attributes": { "id": "5" } },
      { "key": "my-test", "variations": [0, 1], "coverage": 0.4 },
      1,
      true,
      true
    ],
    [
      "coverage - 6",
      { "attributes": { "id": "6" } },
      { "key": "my-test", "variations": [0, 1], "coverage": 0.4 },
      0,
      false,
      false
    ],
    [
      "coverage - 7",
      { "attributes": { "id": "7" } },
      { "key": "my-test", "variations": [0, 1], "coverage": 0.4 },
      0,
      true,
      true
    ],
    [
      "coverage - 8",
      { "attributes": { "id": "8" } },
      { "key": "my-test", "variations": [0, 1], "coverage": 0.4 },
      1,
      true,
      true
    ],
    [
      "coverage - 9",
      { "attributes": { "id": "9" } },
      { "key": "my-test", "variations": [0, 1], "coverage": 0.4 },
      0,
      false,
      false
    ],
    [
      "three way test - 1",
      { "attributes": { "id": "1" } },
      { "key": "my-test", "variations": [0, 1, 2] },
      2,
      true,
      true
    ],
    [
      "three way test - 2",
      { "attributes": { "id": "2" } },
      { "key": "my-test", "variations": [0, 1, 2] },
      0,
      true,
      true
    ],
    [
      "three way test - 3",
      { "attributes": { "id": "3" } },
      { "key": "my-test", "variations": [0, 1, 2] },
      0,
      true,
      true
    ],
    [
      "three way test - 4",
      { "attributes": { "id": "4" } },
      { "key": "my-test", "variations": [0, 1, 2] },
      2,
      true,
      true
    ],
    [
      "three way test - 5",
      { "attributes": { "id": "5" } },
      { "key": "my-test", "variations": [0, 1, 2] },
      1,
      true,
      true
    ],
    [
      "three way test - 6",
      { "attributes": { "id": "6" } },
      { "key": "my-test", "variations": [0, 1, 2] },
      2,
      true,
      true
    ],
    [
      "three way test - 7",
      { "attributes": { "id": "7" } },
      { "key": "my-test", "variations": [0, 1, 2] },
      0,
      true,
      true
    ],
    [
      "three way test - 8",
      { "attributes": { "id": "8" } },
      { "key": "my-test", "variations": [0, 1, 2] },
      1,
      true,
      true
    ],
    [
      "three way test - 9",
      { "attributes": { "id": "9" } },
      { "key": "my-test", "variations": [0, 1, 2] },
      0,
      true,
      true
    ],
    [
      "test name - my-test",
      { "attributes": { "id": "1" } },
      { "key": "my-test", "variations": [0, 1] },
      1,
      true,
      true
    ],
    [
      "test name - my-test-3",
      { "attributes": { "id": "1" } },
      { "key": "my-test-3", "variations": [0, 1] },
      0,
      true,
      true
    ],
    [
      "empty id",
      { "attributes": { "id": "" } },
      { "key": "my-test", "variations": [0, 1] },
      0,
      false,
      false
    ],
    [
      "null id",
      { "attributes": { "id": null } },
      { "key": "my-test", "variations": [0, 1] },
      0,
      false,
      false
    ],
    [
      "missing id",
      { "attributes": {} },
      { "key": "my-test", "variations": [0, 1] },
      0,
      false,
      false
    ],
    [
      "missing attributes",
      {},
      { "key": "my-test", "variations": [0, 1] },
      0,
      false,
      false
    ],
    [
      "single variation",
      { "attributes": { "id": "1" } },
      { "key": "my-test", "variations": [0] },
      0,
      false,
      false
    ],
    [
      "negative forced variation",
      { "attributes": { "id": "1" } },
      { "key": "my-test", "variations": [0, 1], "force": -8 },
      0,
      false,
      false
    ],
    [
      "high forced variation",
      { "attributes": { "id": "1" } },
      { "key": "my-test", "variations": [0, 1], "force": 25 },
      0,
      false,
      false
    ],
    [
      "evaluates conditions - pass",
      {
        "attributes": {
          "id": "1",
          "browser": "firefox"
        }
      },
      {
        "key": "my-test",
        "variations": [0, 1],
        "condition": {
          "browser": "firefox"
        }
      },
      1,
      true,
      true
    ],
    [
      "evaluates conditions - fail",
      {
        "attributes": {
          "id": "1",
          "browser": "chrome"
        }
      },
      {
        "key": "my-test",
        "variations": [0, 1],
        "condition": {
          "browser": "firefox"
        }
      },
      0,
      false,
      false
    ],
    [
      "custom hashAttribute",
      {
        "attributes": {
          "id": "2",
          "companyId": "1"
        }
      },
      {
        "key": "my-test",
        "variations": [0, 1],
        "hashAttribute": "companyId"
      },
      1,
      true,
      true
    ],
    [
      "globally disabled",
      {
        "attributes": {
          "id": "1"
        },
        "enabled": false
      },
      {
        "key": "my-test",
        "variations": [0, 1]
      },
      0,
      false,
      false
    ],
    [
      "querystring force",
      {
        "attributes": {
          "id": "1"
        },
        "url": "http://example.com?forced-test-qs=1#someanchor"
      },
      {
        "key": "forced-test-qs",
        "variations": [0, 1]
      },
      1,
      true,
      false
    ],
    [
      "run active experiments",
      {
        "attributes": {
          "id": "1"
        }
      },
      {
        "key": "my-test",
        "active": true,
        "variations": [0, 1]
      },
      1,
      true,
      true
    ],
    [
      "skip inactive experiments",
      {
        "attributes": {
          "id": "1"
        }
      },
      {
        "key": "my-test",
        "active": false,
        "variations": [0, 1]
      },
      0,
      false,
      false
    ],
    [
      "querystring force with inactive",
      {
        "attributes": {
          "id": "1"
        },
        "url": "http://example.com/?my-test=1"
      },
      {
        "key": "my-test",
        "active": false,
        "variations": [0, 1]
      },
      1,
      true,
      false
    ],
    [
      "coverage take precendence over forced",
      {
        "attributes": {
          "id": "1"
        }
      },
      {
        "key": "my-test",
        "force": 1,
        "coverage": 0.01,
        "variations": [0, 1]
      },
      0,
      false,
      false
    ],
    [
      "JSON values for experiments",
      {
        "attributes": {
          "id": "1"
        }
      },
      {
        "key": "my-test",
        "variations": [
          {
            "color": "blue",
            "size": "small"
          },
          {
            "color": "green",
            "size": "large"
          }
        ]
      },
      {
        "color": "green",
        "size": "large"
      },
      true,
      true
    ],
    [
      "Force variation from context",
      {
        "attributes": { "id": "1" },
        "forcedVariations": { "my-test": 0 }
      },
      {
        "key": "my-test",
        "variations": [0, 1]
      },
      0,
      true,
      false
    ],
    [
      "Skips experiments in QA mode",
      {
        "attributes": { "id": "1" },
        "qaMode": true
      },
      {
        "key": "my-test",
        "variations": [0, 1]
      },
      0,
      false,
      false
    ],
    [
      "Works in QA mode if forced in context",
      {
        "attributes": { "id": "1" },
        "qaMode": true,
        "forcedVariations": { "my-test": 1 }
      },
      {
        "key": "my-test",
        "variations": [0, 1]
      },
      1,
      true,
      false
    ],
    [
      "Works in QA mode if forced in experiment",
      {
        "attributes": { "id": "1" },
        "qaMode": true
      },
      {
        "key": "my-test",
        "variations": [0, 1],
        "force": 1
      },
      1,
      true,
      false
    ],
    [
      "Experiment namespace - pass",
      {
        "attributes": {
          "id": "1"
        }
      },
      {
        "key": "my-test",
        "variations": [0, 1],
        "namespace": ["namespace", 0.1, 1]
      },
      1,
      true,
      true
    ],
    [
      "Experiment namespace - fail",
      {
        "attributes": {
          "id": "1"
        }
      },
      {
        "key": "my-test",
        "variations": [0, 1],
        "namespace": ["namespace", 0, 0.1]
      },
      0,
      false,
      false
    ],
    [
      "Experiment coverage - Works when 0",
      {
        "attributes": {
          "id": "1"
        }
      },
      {
        "key": "no-coverage",
        "variations": [0, 1],
        "coverage": 0
      },
      0,
      false,
      false
    ],
    [
      "Filtered, included",
      {
        "attributes": {
          "id": "1",
          "anonId": "fsdafsda"
        }
      },
      {
        "key": "filtered",
        "variations": [0, 1],
        "filters": [
          {
            "seed": "seed",
            "ranges": [
              [0, 0.1],
              [0.2, 0.4]
            ]
          },
          {
            "seed": "seed",
            "attribute": "anonId",
            "ranges": [[0.8, 1.0]]
          }
        ]
      },
      1,
      true,
      true
    ],
    [
      "Filtered, excluded",
      {
        "attributes": {
          "id": "1",
          "anonId": "fsdafsda"
        }
      },
      {
        "key": "filtered",
        "variations": [0, 1],
        "filters": [
          {
            "seed": "seed",
            "ranges": [
              [0, 0.1],
              [0.2, 0.4]
            ]
          },
          {
            "seed": "seed",
            "attribute": "anonId",
            "ranges": [[0.6, 0.8]]
          }
        ]
      },
      0,
      false,
      false
    ],
    [
      "Filtered, ignore namespace",
      {
        "attributes": {
          "id": "1"
        }
      },
      {
        "key": "filtered",
        "variations": [0, 1],
        "filters": [
          {
            "seed": "seed",
            "ranges": [
              [0, 0.1],
              [0.2, 0.4]
            ]
          }
        ],
        "namespace": ["test", 0, 0.001]
      },
      1,
      true,
      true
    ],
    [
      "Ranges, ignore coverage and weights",
      {
        "attributes": {
          "id": "1"
        }
      },
      {
        "key": "ranges",
        "variations": [0, 1],
        "ranges": [
          [0.99, 1.0],
          [0.0, 0.99]
        ],
        "coverage": 0.01,
        "weights": [0.99, 0.01]
      },
      1,
      true,
      true
    ],
    [
      "Ranges, partial coverage",
      {
        "attributes": {
          "id": "1"
        }
      },
      {
        "key": "configs",
        "variations": [0, 1],
        "ranges": [
          [0, 0.1],
          [0.9, 1.0]
        ]
      },
      0,
      false,
      false
    ],
    [
      "Uses seed and hash version",
      {
        "attributes": {
          "id": "1"
        }
      },
      {
        "key": "key",
        "seed": "foo",
        "hashVersion": 2,
        "variations": [0, 1],
        "ranges": [
          [0, 0.5],
          [0.5, 1.0]
        ]
      },
      1,
      true,
      true
    ],
    [
      "Uses seed with default weights/coverage",
      {
        "attributes": {
          "id": "1"
        }
      },
      {
        "key": "key",
        "seed": "foo",
        "hashVersion": 2,
        "variations": [0, 1]
      },
      1,
      true,
      true
    ],
    [
      "Uses seed with weights/coverage",
      {
        "attributes": {
          "id": "1"
        }
      },
      {
        "key": "key",
        "seed": "foo",
        "hashVersion": 2,
        "variations": [0, 1],
        "weights": [0.5, 0.5],
        "coverage": 0.99
      },
      1,
      true,
      true
    ],
    [
      "Prerequisite condition passes",
      {
        "attributes": { "id": "1" },
        "features": {
          "parentFlag": {
            "defaultValue": true
          }
        }
      },
      {
        "key": "my-test",
        "variations": [0, 1],
        "parentConditions": [
          {
            "id": "parentFlag",
            "condition": {
              "value": true
            }
          }
        ]
      },
      1,
      true,
      true
    ],
    [
      "Prerequisite condition fails",
      {
        "attributes": { "id": "1" },
        "features": {
          "parentFlag": {
            "defaultValue": false
          }
        }
      },
      {
        "key": "my-test",
        "variations": [0, 1],
        "parentConditions": [
          {
            "id": "parentFlag",
            "condition": {
              "value": true
            }
          }
        ]
      },
      0,
      false,
      false
    ]
  ],
  "chooseVariation": [
    [
      "even range, 0.2",
      0.2,
      [
        [0, 0.5],
        [0.5, 1]
      ],
      0
    ],
    [
      "even range, 0.4",
      0.4,
      [
        [0, 0.5],
        [0.5, 1]
      ],
      0
    ],
    [
      "even range, 0.6",
      0.6,
      [
        [0, 0.5],
        [0.5, 1]
      ],
      1
    ],
    [
      "even range, 0.8",
      0.8,
      [
        [0, 0.5],
        [0.5, 1]
      ],
      1
    ],
    [
      "even range, 0",
      0,
      [
        [0, 0.5],
        [0.5, 1]
      ],
      0
    ],
    [
      "even range, 0.5",
      0.5,
      [
        [0, 0.5],
        [0.5, 1]
      ],
      1
    ],
    [
      "reduced range, 0.2",
      0.2,
      [
        [0, 0.25],
        [0.5, 0.75]
      ],
      0
    ],
    [
      "reduced range, 0.4",
      0.4,
      [
        [0, 0.25],
        [0.5, 0.75]
      ],
      -1
    ],
    [
      "reduced range, 0.6",
      0.6,
      [
        [0, 0.25],
        [0.5, 0.75]
      ],
      1
    ],
    [
      "reduced range, 0.8",
      0.8,
      [
        [0, 0.25],
        [0.5, 0.75]
      ],
      -1
    ],
    [
      "reduced range, 0.25",
      0.25,
      [
        [0, 0.25],
        [0.5, 0.75]
      ],
      -1
    ],
    [
      "reduced range, 0.5",
      0.5,
      [
        [0, 0.25],
        [0.5, 0.75]
      ],
      1
    ],
    [
      "zero range",
      0.5,
      [
        [0, 0.5],
        [0.5, 0.5],
        [0.5, 1]
      ],
      2
    ]
  ],
  "getQueryStringOverride": [
    ["empty url", "my-test", "", 2, null],
    ["no query string", "my-test", "http://example.com", 2, null],
    ["empty query string", "my-test", "http://example.com?", 2, null],
    [
      "no query string match",
      "my-test",
      "http://example.com?somequery",
      2,
      null
    ],
    ["invalid query string", "my-test", "http://example.com??&&&?#", 2, null],
    ["simple match 0", "my-test", "http://example.com?my-test=0", 2, 0],
    ["simple match 1", "my-test", "http://example.com?my-test=1", 2, 1],
    ["negative variation", "my-test", "http://example.com?my-test=-1", 2, null],
    ["float", "my-test", "http://example.com?my-test=2.054", 2, null],
    ["string", "my-test", "http://example.com?my-test=foo", 2, null],
    ["variation too high", "my-test", "http://example.com?my-test=5", 2, null],
    ["high numVariations", "my-test", "http://example.com?my-test=5", 6, 5],
    [
      "equal to numVariations",
      "my-test",
      "http://example.com?my-test=5",
      5,
      null
    ],
    [
      "other query string before",
      "my-test",
      "http://example.com?foo=bar&my-test=1",
      2,
      1
    ],
    [
      "other query string after",
      "my-test",
      "http://example.com?foo=bar&my-test=1&bar=baz",
      2,
      1
    ],
    ["anchor", "my-test", "http://example.com?my-test=1#foo", 2, 1]
  ],
  "inNamespace": [
    ["user 1, namespace1, 1", "1", ["namespace1", 0, 0.4], false],
    ["user 1, namespace1, 2", "1", ["namespace1", 0.4, 1], true],
    ["user 1, namespace2, 1", "1", ["namespace2", 0, 0.4], false],
    ["user 1, namespace2, 2", "1", ["namespace2", 0.4, 1], true],
    ["user 2, namespace1, 1", "2", ["namespace1", 0, 0.4], false],
    ["user 2, namespace1, 2", "2", ["namespace1", 0.4, 1], true],
    ["user 2, namespace2, 1", "2", ["namespace2", 0, 0.4], false],
    ["user 2, namespace2, 2", "2", ["namespace2", 0.4, 1], true],
    ["user 3, namespace1, 1", "3", ["namespace1", 0, 0.4], false],
    ["user 3, namespace1, 2", "3", ["namespace1", 0.4, 1], true],
    ["user 3, namespace2, 1", "3", ["namespace2", 0, 0.4], true],
    ["user 3, namespace2, 2", "3", ["namespace2", 0.4, 1], false],
    ["user 4, namespace1, 1", "4", ["namespace1", 0, 0.4], false],
    ["user 4, namespace1, 2", "4", ["namespace1", 0.4, 1], true],
    ["user 4, namespace2, 1", "4", ["namespace2", 0, 0.4], true],
    ["user 4, namespace2, 2", "4", ["namespace2", 0.4, 1], false]
  ],
  "getEqualWeights": [
    [-1, []],
    [0, []],
    [1, [1]],
    [2, [0.5, 0.5]],
    [3, [0.33333333, 0.33333333, 0.33333333]],
    [4, [0.25, 0.25, 0.25, 0.25]]
  ],
  "decrypt": [
    [
      "Valid feature",
      "m5ylFM6ndyOJA2OPadubkw==.Uu7ViqgKEt/dWvCyhI46q088PkAEJbnXKf3KPZjf9IEQQ+A8fojNoxw4wIbPX3aj",
      "Zvwv/+uhpFDznZ6SX28Yjg==",
      "{\"feature\":{\"defaultValue\":true}}"
    ],
    [
      "Broken JSON",
      "SVZIM2oKD1JoHNIeeoW3Uw==.AGbRiGAHf2f6/ziVr9UTIy+bVFmVli6+bHZ2jnCm9N991ITv1ROvOEjxjLSmgEpv",
      "UQD0Qqw7fM1bhfKKPH8TGw==",
      "{\"feature\":{\"defaultValue\":true?5%"
    ],
    [
      "Wrong key",
      "m5ylFM6ndyOJA2OPadubkw==.Uu7ViqgKEt/dWvCyhI46q088PkAEJbnXKf3KPZjf9IEQQ+A8fojNoxw4wIbPX3aj",
      "Zvwv/+uhpFDznZ6SX39Yjg==",
      null
    ],
    [
      "Invalid key length",
      "m5ylFM6ndyOJA2OPadubkw==.Uu7ViqgKEt/dWvCyhI46q088PkAEJbnXKf3KPZjf9IEQQ+A8fojNoxw4wIbPX3aj",
      "Zvwv/+uhpFDznSX39Yjg==",
      null
    ],
    [
      "Invalid key characters",
      "m5ylFM6ndyOJA2OPadubkw==.Uu7ViqgKEt/dWvCyhI46q088PkAEJbnXKf3KPZjf9IEQQ+A8fojNoxw4wIbPX3aj",
      "Zvwv/%!(pFDznZ6SX39Yjg==",
      null
    ],
    [
      "Invalid body",
      "m5ylFM6ndyOJA2OPadubkw==.Uu7ViqgKEt/dWvCyhI46q0!*&()f3KPZjf9IEQQ+A8fojNoxw4wIbPX3aj",
      "Zvwv/+uhpFDznZ6SX28Yjg==",
      null
    ],
    [
      "Invalid iv length",
      "m5ylFM6ndyOPadubkw==.Uu7ViqgKEt/dWvCyhI46q088PkAEJbnXKf3KPZjf9IEQQ+A8fojNoxw4wIbPX3aj",
      "Zvwv/+uhpFDznZ6SX28Yjg==",
      null
    ],
    [
      "Invalid iv",
      "m5ylFM6*&(OJA2OPadubkw==.Uu7ViqgKEt/dWvCyhI46q088PkAEJbnXKf3KPZjf9IEQQ+A8fojNoxw4wIbPX3aj",
      "Zvwv/+uhpFDznZ6SX28Yjg==",
      null
    ],
    [
      "Missing delimiter",
      "m5ylFM6ndyOJA2OPadubkw==Uu7ViqgKEt/dWvCyhI46q088PkAEJbnXKf3KPZjf9IEQQ+A8fojNoxw4wIbPX3aj",
      "Zvwv/+uhpFDznZ6SX28Yjg==",
      null
    ],
    [
      "Corrupted payload",
      "fsa*(&(SF*&F&SF^SD&*FS&*6fsdkajfd",
      "Zvwv/+uhpFDznZ6SX28Yjg==",
      null
    ]
  ],
  "stickyBucket": [
    [
      "use fallbackAttribute when missing hashAttribute",
      {
        "attributes": { "anonymousId": "123" },
        "features": {
          "feature": {
            "defaultValue": 0,
            "rules": [
              {
                "variations": [0, 1, 2, 3],
                "hashAttribute": "id",
                "fallbackAttribute": "anonymousId"
              }
            ]
          }
        }
      },
      [],
      "feature",
      {
        "bucket": 0.863,
        "featureId": "feature",
        "hashAttribute": "anonymousId",
        "hashUsed": true,
        "hashValue": "123",
        "inExperiment": true,
        "key": "3",
        "stickyBucketUsed": false,
        "value": 3,
        "variationId": 3
      },
      {
        "anonymousId||123": {
          "assignments": { "feature__0": "3" },
          "attributeName": "anonymousId",
          "attributeValue": "123"
        }
      }
    ],
    [
      "performs evaluation without sticky bucket",
      {
        "attributes": {
          "deviceId": "d123",
          "anonymousId": "ses123",
          "foo": "bar",
          "country": "USA"
        },
        "features": {
          "exp1": {
            "defaultValue": "control",
            "rules": [
              {
                "key": "feature-exp",
                "seed": "feature-exp",
                "hashAttribute": "id",
                "fallbackAttribute": "deviceId",
                "hashVersion": 2,
                "bucketVersion": 0,
                "condition": { "country": "USA" },
                "variations": ["control", "red", "blue"],
                "meta": [{ "key": "0" }, { "key": "1" }, { "key": "2" }],
                "coverage": 1,
                "weights": [0.3334, 0.3333, 0.3333],
                "phase": "0"
              }
            ]
          }
        },
        "stickyBucketAssignmentDocs": {}
      },
      [],
      "exp1",
      {
        "bucket": 0.6468,
        "featureId": "exp1",
        "hashAttribute": "deviceId",
        "hashUsed": true,
        "hashValue": "d123",
        "inExperiment": true,
        "key": "1",
        "stickyBucketUsed": false,
        "value": "red",
        "variationId": 1
      },
      {
        "deviceId||d123": {
          "assignments": { "feature-exp__0": "1" },
          "attributeName": "deviceId",
          "attributeValue": "d123"
        }
      }
    ],
    [
      "evaluates based on stored sticky bucket",
      {
        "attributes": {
          "deviceId": "d123",
          "anonymousId": "ses123",
          "foo": "bar",
          "country": "USA"
        },
        "features": {
          "exp1": {
            "defaultValue": "control",
            "rules": [
              {
                "key": "feature-exp",
                "seed": "feature-exp",
                "hashAttribute": "id",
                "fallbackAttribute": "deviceId",
                "hashVersion": 2,
                "bucketVersion": 0,
                "condition": { "country": "USA" },
                "variations": ["control", "red", "blue"],
                "meta": [{ "key": "0" }, { "key": "1" }, { "key": "2" }],
                "coverage": 1,
                "weights": [0.3334, 0.3333, 0.3333],
                "phase": "0"
              }
            ]
          }
        }
      },
      [
        {
          "attributeName": "deviceId",
          "attributeValue": "d123",
          "assignments": {
            "feature-exp__0": "2"
          }
        }
      ],
      "exp1",
      {
        "bucket": 0.6468,
        "featureId": "exp1",
        "hashAttribute": "deviceId",
        "hashUsed": true,
        "hashValue": "d123",
        "inExperiment": true,
        "key": "2",
        "stickyBucketUsed": true,
        "value": "blue",
        "variationId": 2
      },
      {
        "deviceId||d123": {
          "assignments": { "feature-exp__0": "2" },
          "attributeName": "deviceId",
          "attributeValue": "d123"
        }
      }
    ],
    [
      "does not consume a sticky bucket not belonging to the user",
      {
        "attributes": {
          "deviceId": "d123",
          "anonymousId": "ses123",
          "foo": "bar",
          "country": "USA"
        },
        "features": {
          "exp1": {
            "defaultValue": "control",
            "rules": [
              {
                "key": "feature-exp",
                "seed": "feature-exp",
                "hashAttribute": "id",
                "fallbackAttribute": "deviceId",
                "hashVersion": 2,
                "bucketVersion": 0,
                "condition": { "country": "USA" },
                "variations": ["control", "red", "blue"],
                "meta": [{ "key": "0" }, { "key": "1" }, { "key": "2" }],
                "coverage": 1,
                "weights": [0.3334, 0.3333, 0.3333],
                "phase": "0"
              }
            ]
          }
        }
      },
      [
        {
          "attributeName": "deviceId",
          "attributeValue": "d456",
          "assignments": {
            "feature-exp__0": "2"
          }
        }
      ],
      "exp1",
      {
        "bucket": 0.6468,
        "featureId": "exp1",
        "hashAttribute": "deviceId",
        "hashUsed": true,
        "hashValue": "d123",
        "inExperiment": true,
        "key": "1",
        "stickyBucketUsed": false,
        "value": "red",
        "variationId": 1
      },
      {
        "deviceId||d123": {
          "assignments": { "feature-exp__0": "1" },
          "attributeName": "deviceId",
          "attributeValue": "d123"
        }
      }
    ],
    [
      "upgrades a sticky bucket doc from a fallbackAttribute to a hashAttribute",
      {
        "attributes": {
          "id": "i123",
          "anonymousId": "ses123",
          "foo": "bar",
          "country": "USA"
        },
        "features": {
          "exp1": {
            "defaultValue": "control",
            "rules": [
              {
                "key": "feature-exp",
                "seed": "feature-exp",
                "hashAttribute": "id",
                "fallbackAttribute": "anonymousId",
                "hashVersion": 2,
                "bucketVersion": 0,
                "condition": { "country": "USA" },
                "variations": ["control", "red", "blue"],
                "meta": [{ "key": "0" }, { "key": "1" }, { "key": "2" }],
                "coverage": 1,
                "weights": [0.3334, 0.3333, 0.3333],
                "phase": "0"
              }
            ]
          }
        }
      },
      [
        {
          "attributeName": "anonymousId",
          "attributeValue": "ses123",
          "assignments": {
            "feature-exp__0": "1"
          }
        }
      ],
      "exp1",
      {
        "bucket": 0.9943,
        "featureId": "exp1",
        "hashAttribute": "id",
        "hashUsed": true,
        "hashValue": "i123",
        "inExperiment": true,
        "key": "1",
        "stickyBucketUsed": true,
        "value": "red",
        "variationId": 1
      },
      {
        "anonymousId||ses123": {
          "assignments": { "feature-exp__0": "1" },
          "attributeName": "anonymousId",
          "attributeValue": "ses123"
        },
        "id||i123": {
          "assignments": { "feature-exp__0": "1" },
          "attributeName": "id",
          "attributeValue": "i123"
        }
      }
    ],
    [
      "favors a sticky bucket doc based on hashAttribute over fallbackAttribute",
      {
        "attributes": {
          "id": "i123",
          "anonymousId": "ses123",
          "foo": "bar",
          "country": "USA"
        },
        "features": {
          "exp1": {
            "defaultValue": "control",
            "rules": [
              {
                "key": "feature-exp",
                "seed": "feature-exp",
                "hashAttribute": "id",
                "fallbackAttribute": "anonymousId",
                "hashVersion": 2,
                "bucketVersion": 0,
                "condition": { "country": "USA" },
                "variations": ["control", "red", "blue"],
                "meta": [{ "key": "0" }, { "key": "1" }, { "key": "2" }],
                "coverage": 1,
                "weights": [0.3334, 0.3333, 0.3333],
                "phase": "0"
              }
            ]
          }
        }
      },
      [
        {
          "attributeName": "anonymousId",
          "attributeValue": "ses123",
          "assignments": {
            "feature-exp__0": "2"
          }
        },
        {
          "attributeName": "id",
          "attributeValue": "i123",
          "assignments": {
            "feature-exp__0": "1"
          }
        }
      ],
      "exp1",
      {
        "bucket": 0.9943,
        "featureId": "exp1",
        "hashAttribute": "id",
        "hashUsed": true,
        "hashValue": "i123",
        "inExperiment": true,
        "key": "1",
        "stickyBucketUsed": true,
        "value": "red",
        "variationId": 1
      },
      {
        "anonymousId||ses123": {
          "assignments": { "feature-exp__0": "2" },
          "attributeName": "anonymousId",
          "attributeValue": "ses123"
        },
        "id||i123": {
          "assignments": { "feature-exp__0": "1" },
          "attributeName": "id",
          "attributeValue": "i123"
        }
      }
    ],
    [
      "resets sticky bucketing when the bucketVersion changes",
      {
        "attributes": {
          "id": "i123",
          "foo": "bar",
          "country": "USA"
        },
        "features": {
          "exp1": {
            "defaultValue": "control",
            "rules": [
              {
                "key": "feature-exp",
                "seed": "feature-exp",
                "hashAttribute": "id",
                "fallbackAttribute": "deviceId",
                "hashVersion": 2,
                "bucketVersion": 3,
                "condition": { "country": "USA" },
                "variations": ["control", "red", "blue"],
                "meta": [{ "key": "0" }, { "key": "1" }, { "key": "2" }],
                "coverage": 1,
                "weights": [0.3334, 0.3333, 0.3333],
                "phase": "0"
              }
            ]
          }
        }
      },
      [
        {
          "assignments": { "feature-exp__0": "1" },
          "attributeName": "id",
          "attributeValue": "i123"
        }
      ],
      "exp1",
      {
        "bucket": 0.9943,
        "featureId": "exp1",
        "hashAttribute": "id",
        "hashUsed": true,
        "hashValue": "i123",
        "inExperiment": true,
        "key": "2",
        "stickyBucketUsed": false,
        "value": "blue",
        "variationId": 2
      },
      {
        "id||i123": {
          "assignments": {
            "feature-exp__0": "1",
            "feature-exp__3": "2"
          },
          "attributeName": "id",
          "attributeValue": "i123"
        }
      }
    ],
    [
      "stops test enrollment when and existing sticky bucket is blocked by version",
      {
        "attributes": {
          "id": "i123",
          "foo": "bar",
          "country": "USA"
        },
        "features": {
          "exp1": {
            "defaultValue": "control",
            "rules": [
              {
                "key": "feature-exp",
                "seed": "feature-exp",
                "hashAttribute": "id",
                "fallbackAttribute": "deviceId",
                "hashVersion": 2,
                "bucketVersion": 3,
                "minBucketVersion": 3,
                "condition": { "country": "USA" },
                "variations": ["control", "red", "blue"],
                "meta": [{ "key": "0" }, { "key": "1" }, { "key": "2" }],
                "coverage": 1,
                "weights": [0.3334, 0.3333, 0.3333],
                "phase": "0"
              }
            ]
          }
        }
      },
      [
        {
          "assignments": { "feature-exp__0": "1" },
          "attributeName": "id",
          "attributeValue": "i123"
        }
      ],
      "exp1",
      null,
      {
        "id||i123": {
          "assignments": {
            "feature-exp__0": "1"
          },
          "attributeName": "id",
          "attributeValue": "i123"
        }
      }
    ],
    [
      "disables sticky bucketing when disabled by experiment",
      {
        "attributes": {
          "id": "i123",
          "foo": "bar",
          "country": "USA"
        },
        "features": {
          "exp1": {
            "defaultValue": "control",
            "rules": [
              {
                "key": "feature-exp",
                "seed": "feature-exp",
                "hashAttribute": "id",
                "fallbackAttribute": "deviceId",
                "hashVersion": 2,
                "bucketVersion": 1,
                "disableStickyBucketing": true,
                "condition": { "country": "USA" },
                "variations": ["control", "red", "blue"],
                "meta": [{ "key": "0" }, { "key": "1" }, { "key": "2" }],
                "coverage": 1,
                "weights": [0.3334, 0.3333, 0.3333],
                "phase": "0"
              }
            ]
          }
        }
      },
      [
        {
          "attributeName": "id",
          "attributeValue": "i123",
          "assignments": { "feature-exp__0": "1" }
        }
      ],
      "exp1",
      {
        "bucket": 0.9943,
        "featureId": "exp1",
        "hashAttribute": "id",
        "hashUsed": true,
        "hashValue": "i123",
        "inExperiment": true,
        "key": "2",
        "stickyBucketUsed": false,
        "value": "blue",
        "variationId": 2
      },
      {
        "id||i123": {
          "attributeName": "id",
          "attributeValue": "i123",
          "assignments": { "feature-exp__0": "1" }
        }
      }
    ]
  ],
  "urlRedirect": [
    [
      "redirects correctly without query strings",
      {
        "attributes": { "id": "1" },
        "url": "http://www.example.com/home",
        "experiments": [
          {
            "key": "my-experiment",
            "urlPatterns": [
              {
                "type": "simple",
                "include": true,
                "pattern": "http://www.example.com/home"
              }
            ],
            "weights": [0.1, 0.9],
            "variations": [
              {},
              {
                "urlRedirect": "http://www.example.com/home-new"
              }
            ]
          }
        ]
      },
      [
        {
          "inExperiment": true,
          "urlRedirect": "http://www.example.com/home-new",
          "urlWithParams": "http://www.example.com/home-new"
        }
      ]
    ],
    [
      "redirects with query string on original url and persistQueryString enabled",
      {
        "attributes": { "id": "1" },
        "url": "http://www.example.com/home?color=blue&food=sushi",
        "experiments": [
          {
            "key": "my-experiment",
            "urlPatterns": [
              {
                "type": "simple",
                "include": true,
                "pattern": "http://www.example.com/home"
              }
            ],
            "weights": [0.1, 0.9],
            "variations": [
              {},
              {
                "urlRedirect": "http://www.example.com/home-new"
              }
            ],
            "persistQueryString": true
          }
        ]
      },
      [
        {
          "inExperiment": true,
          "urlRedirect": "http://www.example.com/home-new",
          "urlWithParams": "http://www.example.com/home-new?color=blue&food=sushi"
        }
      ]
    ],
    [
      "merges query strings on original url & redirect url with param conflicts correctly when persistQueryString enabled",
      {
        "attributes": { "id": "1" },
        "url": "http://www.example.com/home?color=blue&food=sushi&title=original",
        "experiments": [
          {
            "key": "my-experiment",
            "urlPatterns": [
              {
                "type": "simple",
                "include": true,
                "pattern": "http://www.example.com/home"
              }
            ],
            "weights": [0.1, 0.9],
            "variations": [
              {},
              {
                "urlRedirect": "http://www.example.com/home-new?name=test&color=red&food=lasagna"
              }
            ],
            "persistQueryString": true
          }
        ]
      },
      [
        {
          "inExperiment": true,
          "urlRedirect": "http://www.example.com/home-new?name=test&color=red&food=lasagna",
          "urlWithParams": "http://www.example.com/home-new?name=test&color=red&food=lasagna&title=original"
        }
      ]
    ],
    [
      "only performs a redirect for first eligible experiment when there are multiple eligible experiments",
      {
        "attributes": { "id": "1" },
        "url": "http://www.example.com/home",
        "experiments": [
          {
            "key": "my-experiment",
            "urlPatterns": [
              {
                "type": "simple",
                "include": true,
                "pattern": "http://www.example.com/"
              }
            ],
            "weights": [0.1, 0.9],
            "variations": [
              {},
              {
                "urlRedirect": "http://www.example.com/home-new"
              }
            ]
          },
          {
            "key": "my-experiment-2",
            "urlPatterns": [
              {
                "type": "simple",
                "include": true,
                "pattern": "http://www.example.com/home"
              }
            ],
            "weights": [0.1, 0.9],
            "variations": [
              {},
              {
                "urlRedirect": "http://www.example.com/home-new-new"
              }
            ]
          },
          {
            "key": "my-experiment-3",
            "urlPatterns": [
              {
                "type": "simple",
                "include": true,
                "pattern": "http://www.example.com/home"
              }
            ],
            "weights": [0.1, 0.9],
            "variations": [
              {},
              {
                "urlRedirect": "http://www.example.com/home-es"
              }
            ]
          }
        ]
      },
      [
        {
          "inExperiment": true,
          "urlRedirect": "http://www.example.com/home-new-new",
          "urlWithParams": "http://www.example.com/home-new-new"
        }
      ]
    ]
  ]
}
    "#.to_string()
}
