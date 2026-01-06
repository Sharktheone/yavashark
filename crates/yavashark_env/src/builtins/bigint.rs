use crate::builtins::{check_radix, NumberConstructor};
use crate::conversion::downcast_obj;
use crate::error::Error;
use crate::value::Obj;
use crate::{MutObject, ObjectHandle, Realm, Res, Value, ValueResult};
use num_bigint::BigInt;
use num_traits::{Signed, Zero};
use std::cell::RefCell;
use std::rc::Rc;
use yavashark_macro::{object, props};


/// ToIndex(value) - Converts a value to a non-negative integer index.
/// Returns RangeError if the value is negative or > 2^53-1.
fn to_index(value: &Value, realm: &mut Realm) -> Res<u64> {
    // 1. If value is undefined, return 0.
    if value.is_undefined() {
        return Ok(0);
    }

    // 2. Let integerIndex be ? ToIntegerOrInfinity(value).
    let number = value.to_number(realm)?;

    // Handle NaN -> 0
    if number.is_nan() {
        return Ok(0);
    }

    // Truncate towards zero (ToIntegerOrInfinity behavior)
    let integer = number.trunc();

    // 3. If integerIndex < 0, throw a RangeError exception.
    if integer < 0.0 {
        return Err(Error::range("Invalid index: negative value"));
    }

    // 4. If integerIndex > 2^53 - 1, throw a RangeError exception.
    if integer > NumberConstructor::MAX_SAFE_INTEGER {
        return Err(Error::range("Invalid index: value too large"));
    }

    Ok(integer as u64)
}

/// Mathematical modulo operation that always returns a non-negative result.
/// This differs from Rust's `%` operator which can return negative values.
fn modulo(n: &BigInt, d: &BigInt) -> BigInt {
    let rem = n % d;
    if rem.is_negative() {
        rem + d
    } else {
        rem
    }
}

#[object]
#[derive(Debug)]
pub struct BigIntObj {
    #[mutable]
    #[primitive]
    big_int: Rc<BigInt>,
}

impl BigIntObj {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(realm: &mut Realm, big_int: Rc<BigInt>) -> Res<ObjectHandle> {
        Ok(Self {
            inner: RefCell::new(MutableBigIntObj {
                object: MutObject::with_proto(
                    realm.intrinsics.clone_public().bigint.get(realm)?.clone(),
                ),
                big_int,
            }),
        }
        .into_object())
    }
}

#[props(intrinsic_name = bigint)]
impl BigIntObj {
    #[call_constructor]
    fn call(realm: &mut Realm, bint: Value) -> ValueResult {
        Ok(bint.to_big_int(realm)?.into())
    }

    #[prop("asIntN")]
    pub fn int_n(bits: Value, bigint: Value, realm: &mut Realm) -> ValueResult {
        // 1. Set bits to ? ToIndex(bits).
        let bits = to_index(&bits, realm)?;

        // 2. Set bigint to ? ToBigInt(bigint).
        // Note: ToBigInt throws TypeError for Number values
        let bigint = bigint.to_big_int_strict(realm)?;

        // Handle special case: bits == 0
        if bits == 0 {
            return Ok(BigInt::zero().into());
        }

        // 3. Let mod be ℝ(bigint) modulo 2^bits.
        let two_pow_bits = BigInt::from(1) << bits;
        let mod_value = modulo(&bigint, &two_pow_bits);

        // 4. If mod ≥ 2^(bits-1), return ℤ(mod - 2^bits); otherwise return ℤ(mod).
        let two_pow_bits_minus_1 = BigInt::from(1) << (bits - 1);
        if mod_value >= two_pow_bits_minus_1 {
            Ok((mod_value - two_pow_bits).into())
        } else {
            Ok(mod_value.into())
        }
    }

    #[prop("asUintN")]
    pub fn uint_n(bits: Value, bigint: Value, realm: &mut Realm) -> ValueResult {
        // 1. Set bits to ? ToIndex(bits).
        let bits = to_index(&bits, realm)?;

        // 2. Set bigint to ? ToBigInt(bigint).
        // Note: ToBigInt throws TypeError for Number values
        let bigint = bigint.to_big_int_strict(realm)?;

        // Handle special case: bits == 0
        if bits == 0 {
            return Ok(BigInt::zero().into());
        }

        // 3. Return ℤ(ℝ(bigint) modulo 2^bits).
        let two_pow_bits = BigInt::from(1) << bits;
        let mod_value = modulo(&bigint, &two_pow_bits);

        Ok(mod_value.into())
    }

    #[prop("toString")]
    fn to_string(&self, radix: Option<u32>) -> ValueResult {
        let inner = self.inner.try_borrow()?;

        Ok(if let Some(radix) = radix {
            check_radix(radix)?;

            inner.big_int.to_str_radix(radix)
        } else {
            inner.big_int.to_string()
        }
        .into())
    }

    #[prop("valueOf")]
    #[nonstatic]
    fn value_of(#[this] this: Value) -> ValueResult {
        if let Value::BigInt(bi) = this {
            return Ok(bi.into());
        }

        let this = downcast_obj::<Self>(this)?;

        let inner = this.inner.try_borrow()?;

        Ok(inner.big_int.clone().into())
    }

    #[prop("toLocaleString")]
    fn to_locale_string(&self) -> ValueResult {
        let inner = self.inner.try_borrow()?;
        Ok(inner.big_int.to_string().into())
    }

    #[prop(Symbol::TO_STRING_TAG)]
    #[nonstatic]
    #[configurable]
    const TO_STRING_TAG: &'static str = "BigInt";
}
