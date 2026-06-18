use crate::conversion::FromValueOutput;
use crate::{Realm, Res, Value};
use crate::builtins::value_to_plain_year_month;

impl FromValueOutput for temporal_rs::PlainYearMonth {
    type Output = Self;

    fn from_value_out(value: Value, realm: &mut Realm) -> Res<Self::Output> {
        value_to_plain_year_month(value, None, realm)
    }
}