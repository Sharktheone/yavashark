use crate::builtins::temporal::utils::difference_settings;
use crate::conversion::FromValueOutput;
use crate::{Res, Value};

impl FromValueOutput for temporal_rs::options::DifferenceSettings {
    type Output = Self;

    fn from_value_out(value: Value, realm: &mut crate::Realm) -> Res<Self> {
        let opts = value.to_object()?;
        
        difference_settings(opts, realm)
    }
}