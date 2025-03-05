use crate::js::ops::BigIntOrNumber;
use crate::{Error, Realm, Value};

impl<R: Realm> Value<R> {
    pub fn ushr(&self, other: &Self, realm: &mut R) -> Result<Self, Error<R>> {
        //TODO: maybe in the future we could make this more performant by just matching against both types (just like the old Add trait), but this is what the spec says
        let left_num = self.to_numeric(realm)?;
        let right_num = other.to_numeric(realm)?;

        Ok(match (left_num, right_num) {
            (BigIntOrNumber::Number(left), BigIntOrNumber::Number(right)) => {
                Self::from((left as u64) >> (right as u64 % 64))
            }
            (BigIntOrNumber::BigInt(_), BigIntOrNumber::BigInt(_)) => {
                return Err(Error::ty("cannot perform unsigned right shift on BigInt"));
                
            }

            _ => return Err(Error::ty("cannot mix BigInt and Number")),
        })
    }
}
