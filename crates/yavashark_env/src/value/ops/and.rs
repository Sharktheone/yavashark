use num_traits::Zero;
use crate::Realm;
use crate::error::Error;
use crate::value::Value;
use crate::value::ops::BigIntOrNumber;

impl Value {
    pub fn and(&self, other: &Self, realm: &mut Realm) -> Result<Self, Error> {
        match (self, other) {
            (Self::Number(left), Self::Number(right)) => {
                Self::from(left as i64 & right as i64)
            }
            (Self::BigInt(left), Self::BigInt(right)) => {
                Self::from((&*left) & (&*right))
            }
            _ => {}
        }

        //TODO: maybe in the future we could make this more performant by just matching against both types (just like the old Add trait), but this is what the spec says
        let left_num = self.to_numeric(realm)?;
        let right_num = other.to_numeric(realm)?;

        Ok(match (left_num, right_num) {
            (BigIntOrNumber::Number(left), BigIntOrNumber::Number(right)) => {
                Self::from(left as i64 & right as i64)
            }
            (BigIntOrNumber::BigInt(left), BigIntOrNumber::BigInt(right)) => {
                Self::from((&*left) & (&*right))
            }

            _ => return Err(Error::ty("cannot mix BigInt and Number")),
        })
    }
}
