use crate::error::Error;
use crate::value::ops::BigIntOrNumber;
use crate::value::Value;
use crate::Realm;
use num_traits::Zero;

impl Value {
    pub fn div(&self, other: &Self, realm: &mut Realm) -> Result<Self, Error> {
        //TODO: maybe in the future we could make this more performant by just matching against both types (just like the old Add trait), but this is what the spec says
        let left_num = self.to_numeric(realm)?;
        let right_num = other.to_numeric(realm)?;

        Ok(match (left_num, right_num) {
            (BigIntOrNumber::Number(left), BigIntOrNumber::Number(right)) => {
                Self::from(left / right)
            }
            (BigIntOrNumber::BigInt(left), BigIntOrNumber::BigInt(right)) => {
                if right.is_zero() {
                    return Err(Error::range("Division by zero"));
                }

                Self::from((&*left) / (&*right))
            }

            _ => return Err(Error::ty("cannot mix BigInt and Number")),
        })
    }
}
