use serde_json::Value;

use crate::dto::GrowthBookFeatureRuleRollout;
use crate::extensions::FindGrowthBookAttribute;
use crate::feature::condition::ConditionEnabledCheck;
use crate::hash::{HashCode, HashCodeVersion};
use crate::model_public::{GrowthBookAttribute, GrowthBookAttributeValue};

impl GrowthBookFeatureRuleRollout {
    pub fn get_match_value(
        &self,
        feature_name: &str,
        option_user_attributes: Option<&Vec<GrowthBookAttribute>>,
    ) -> Option<Value> {
        if let Some(feature_attributes) = &self.conditions() {
            if let Some(user_attributes) = option_user_attributes {
                if feature_attributes.is_on(user_attributes) {
                    if let Some(user_value) = user_attributes.find_value(&self.hash_attribute) {
                        return self.check_coverage(&user_value, feature_name);
                    }
                }
            }
        } else if let Some(user_attributes) = option_user_attributes {
            if let Some(user_value) = user_attributes.find_value(&self.hash_attribute) {
                return self.check_coverage(&user_value, feature_name);
            }
        }

        None
    }

    fn check_coverage(
        &self,
        value: &GrowthBookAttributeValue,
        feature_name: &str,
    ) -> Option<Value> {
        if self.coverage.gt(&HashCode::hash_code(
            &value.to_string(),
            feature_name,
            HashCodeVersion::V1,
        )) {
            Some(self.force.clone())
        } else {
            None
        }
    }
}
