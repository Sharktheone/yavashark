use crate::error::Error;
use crate::value::ops::BigIntOrNumber;
use crate::value::{Hint, Value};
use crate::Realm;
use num_traits::ToPrimitive;

impl Value {
    pub fn exp(&self, other: &Self, realm: &mut Realm) -> Result<Self, Error> {
        //TODO: maybe in the future we could make this more performant by just matching against both types (just like the old Add trait), but this is what the spec says
        let left = self.to_primitive(Hint::None, realm)?;
        let right = other.to_primitive(Hint::None, realm)?;

        Ok(if left.is_string() || right.is_string() {
            let left_str = left.into_string(realm)?;
            let right_str = right.into_string(realm)?;
            Self::from(left_str + &right_str)
        } else {
            let left_num = left.to_numeric(realm)?;
            let right_num = right.to_numeric(realm)?;

            match (left_num, right_num) {
                (BigIntOrNumber::Number(left), BigIntOrNumber::Number(right)) => {
                    Self::from(left.powf(right))
                }
                (BigIntOrNumber::BigInt(left), BigIntOrNumber::BigInt(right)) => {
                    let Some(right) = right.to_u32() else {
                        return Err(Error::range("exponent too large"));
                    };

                    Self::from(left.pow(right))
                }

                _ => return Err(Error::ty("cannot mix BigInt and Number")),
            }
        })
    }
}
