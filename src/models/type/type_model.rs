use std::collections::HashMap;

use super::*;

#[derive(Debug)]
pub struct TypeModel {
    pub super_type_key: Option<TypeKey>,
    pub r#type: TypeEnum,
    pub validators: Vec<ValidatorEnum>,
    pub properties: HashMap<String, TypeKey>,
}