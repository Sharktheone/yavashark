use crate::js::ops::BigIntOrNumber;
use crate::{Error, Realm, Value};

impl<R: Realm> Value<R> {
    pub fn xor(&self, other: &Self, realm: &mut R) -> Result<Self, Error<R>> {
        //TODO: maybe in the future we could make this more performant by just matching against both types (just like the old Add trait), but this is what the spec says
        let left_num = self.to_numeric(realm)?;
        let right_num = other.to_numeric(realm)?;

        Ok(match (left_num, right_num) {
            (BigIntOrNumber::Number(left), BigIntOrNumber::Number(right)) => {
                Self::from(left as i64 ^ right as i64)
            }
            (BigIntOrNumber::BigInt(left), BigIntOrNumber::BigInt(right)) => {
                Self::from(&*left ^ &*right)
            }

            _ => return Err(Error::ty("cannot add BigInt and Number")),
        })
    }
}
