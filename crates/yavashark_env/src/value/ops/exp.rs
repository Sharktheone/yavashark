use crate::Realm;
use crate::error::Error;
use crate::value::ops::BigIntOrNumber;
use crate::value::Value;
use num_traits::ToPrimitive;

impl Value {
    pub fn exp(&self, other: &Self, realm: &mut Realm) -> Result<Self, Error> {
        match (self, other) {
            (Self::Number(left), Self::Number(right)) => return Ok(left.powf(*right).into()),
            (Self::BigInt(left), Self::BigInt(right)) => {
                let Some(right) = right.to_u32() else {
                    return Err(Error::range("exponent too large"));
                };

                return Ok(left.pow(right).into());
            }
            _ => {}
        }

        let left_num = self.to_numeric(realm)?;
        let right_num = other.to_numeric(realm)?;

        Ok(match (left_num, right_num) {
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
        })
    }
}
