use crate::partial_init::Initializer;
use crate::utils::ValueIterator;
use crate::value::Obj;
use crate::{Error, MutObject, ObjectHandle, Realm, Res, Symbol, Value};
use num_traits::{One, Zero};
use std::cell::RefCell;
use xsum::Xsum;
use yavashark_macro::{object, properties_new};

/// # 21.3 The Math Object
/// https://262.ecma-international.org/#sec-math-object
/// The Math object:
///
///     is %Math%.
///     is the initial value of the "Math" property of the global object.
///     is an ordinary object.
///     has a [[Prototype]] internal slot whose value is %Object.prototype%.
///     is not a function object.
///     does not have a [[Construct]] internal method; it cannot be used as a constructor with the new operator.
///     does not have a [[Call]] internal method; it cannot be invoked as a function.
#[object]
#[derive(Debug)]
pub struct Math {}

impl Math {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(realm: &mut Realm) -> Res<ObjectHandle> {
        let mut this = Self {
            inner: RefCell::new(MutableMath {
                object: MutObject::with_proto(realm.intrinsics.obj.clone()),
            }),
        };

        this.initialize(realm)?;

        Ok(this.into_object())
    }
}

#[properties_new(raw)]
impl Math {
    /// # 21.3.1.1 Math.E
    /// https://262.ecma-international.org/#sec-math.e
    ///
    /// The Number value for e, the base of the natural logarithms, which is approximately 2.7182818284590452354.
    const E: f64 = std::f64::consts::E;

    /// # 21.3.1.2 Math.LN10
    /// https://262.ecma-international.org/#sec-math.ln10
    ///
    /// The Number value for the natural logarithm of 10, which is approximately 2.302585092994046.
    const LN10: f64 = std::f64::consts::LN_10;

    /// # 21.3.1.3 Math.LN2
    /// https://262.ecma-international.org/#sec-math.ln2
    ///
    /// The Number value for the natural logarithm of 2, which is approximately 0.6931471805599453.
    const LN2: f64 = std::f64::consts::LN_2;

    /// # 21.3.1.4 Math.LOG10E
    /// https://262.ecma-international.org/#sec-math.log10e
    ///
    /// The Number value for the base-10 logarithm of e, the base of the natural logarithms; this value is approximately 0.4342944819032518.
    const LOG10E: f64 = std::f64::consts::LOG10_E;

    /// # 21.3.1.5 Math.LOG2E
    /// https://262.ecma-international.org/#sec-math.log2e
    ///
    /// The Number value for the base-2 logarithm of e, the base of the natural logarithms; this value is approximately 1.4426950408889634.
    const LOG2E: f64 = std::f64::consts::LOG2_E;

    /// # 21.3.1.6 Math.PI
    /// https://262.ecma-international.org/#sec-math.pi
    ///
    /// The Number value for π, the ratio of the circumference of a circle to its diameter, which is approximately 3.1415926535897932.
    const PI: f64 = std::f64::consts::PI;

    /// # 21.3.1.7 Math.SQRT1_2
    /// https://262.ecma-international.org/#sec-math.sqrt1_2
    ///
    /// The Number value for the square root of ½, which is approximately 0.7071067811865476.
    const SQRT1_2: f64 = std::f64::consts::FRAC_1_SQRT_2;

    /// # 21.3.1.8 Math.SQRT2
    /// https://262.ecma-international.org/#sec-math.sqrt2
    ///
    /// The Number value for the square root of 2, which is approximately 1.4142135623730951.
    const SQRT2: f64 = std::f64::consts::SQRT_2;

    /// 21.3.1.9 Math [ %Symbol.toStringTag% ]
    /// https://262.ecma-international.org/#sec-math-%symbol.tostringtag%
    ///
    /// The initial value of the %Symbol.toStringTag% property is the String value "Math".
    #[prop(Symbol::TO_STRING_TAG)]
    #[configurable]
    const TO_STRING_TAG: &'static str = "Math";


    /// # 21.3.2.1 Math.abs ( x )
    /// https://262.ecma-international.org/#sec-math.abs
    ///
    /// This function returns the absolute value of x; the result has the same magnitude as x but has positive sign.
    const fn abs(n: f64) -> f64 {
        // 1. Let n be ? ToNumber(x).

        // 2. If n is NaN, return NaN.
        // 3. If n is -0𝔽, return +0𝔽.
        // 4. If n is -∞𝔽, return +∞𝔽.
        // 5. If n < -0𝔽, return -n.
        // 6. Return n.
        n.abs()
    }

    /// # 21.3.2.2 Math.acos ( x )
    /// https://262.ecma-international.org/#sec-math.acos
    ///
    /// This function returns the inverse cosine of x. The result is expressed in radians and is in the inclusive interval from +0𝔽 to 𝔽(π).
    fn acos(n: f64) -> f64 {
        // 1. Let n be ? ToNumber(x).

        // 2. If n is NaN, n > 1𝔽, or n < -1𝔽, return NaN.
        // 3. If n is 1𝔽, return +0𝔽.
        // 4. Return an implementation-approximated Number value representing the inverse cosine of ℝ(n).
        n.acos()
    }

    /// # 21.3.2.3 Math.acosh ( x )
    /// https://262.ecma-international.org/#sec-math.acosh
    ///
    /// This function returns the inverse hyperbolic cosine of x.
    fn acosh(n: f64) -> f64 {
        // 1. Let n be ? ToNumber(x).

        // 2. If n is either NaN or +∞𝔽, return n.
        // 3. If n is 1𝔽, return +0𝔽.
        // 4. If n < 1𝔽, return NaN.
        // 5. Return an implementation-approximated Number value representing the inverse hyperbolic cosine of ℝ(n).
        n.acosh()
    }

    /// # 21.3.2.4 Math.asin ( x )
    /// https://262.ecma-international.org/#sec-math.asin
    ///
    /// This function returns the inverse sine of x. The result is expressed in radians and is in the inclusive interval from 𝔽(-π / 2) to 𝔽(π / 2).
    fn asin(n: f64) -> f64 {
        // 1. Let n be ? ToNumber(x).

        // 2. If n is one of NaN, +0𝔽, or -0𝔽, return n.
        // 3. If n > 1𝔽 or n < -1𝔽, return NaN.
        // 4. Return an implementation-approximated Number value representing the inverse sine of ℝ(n).
        n.asin()
    }

    /// # 21.3.2.5 Math.asinh ( x )
    /// https://262.ecma-international.org/#sec-math.asinh
    ///
    /// This function returns the inverse hyperbolic sine of x.
    fn asinh(n: f64) -> f64 {
        // 1. Let n be ? ToNumber(x).

        // 2. If n is not finite or n is either +0𝔽 or -0𝔽, return n.
        // 3. Return an implementation-approximated Number value representing the inverse hyperbolic sine of ℝ(n).
        n.asinh()
    }

    /// # 21.3.2.6 Math.atan ( x )
    /// https://262.ecma-international.org/#sec-math.atan
    ///
    /// This function returns the inverse tangent of x. The result is expressed in radians and is in the inclusive interval from 𝔽(-π / 2) to 𝔽(π / 2).
    fn atan(n: f64) -> f64 {
        // 1. Let n be ? ToNumber(x).

        // 2. If n is one of NaN, +0𝔽, or -0𝔽, return n.
        // 3. If n is +∞𝔽, return an implementation-approximated Number value representing π / 2.
        // 4. If n is -∞𝔽, return an implementation-approximated Number value representing -π / 2.
        // 5. Return an implementation-approximated Number value representing the inverse tangent of ℝ(n).
        n.atan()
    }

    /// # 21.3.2.7 Math.atanh ( x )
    /// https://262.ecma-international.org/#sec-math.atanh
    ///
    /// This function returns the inverse hyperbolic tangent of x.
    fn atanh(n: f64) -> f64 {
        // 1. Let n be ? ToNumber(x).

        // 2. If n is one of NaN, +0𝔽, or -0𝔽, return n.
        // 3. If n > 1𝔽 or n < -1𝔽, return NaN.
        // 4. If n is 1𝔽, return +∞𝔽.
        // 5. If n is -1𝔽, return -∞𝔽.
        // 6. Return an implementation-approximated Number value representing the inverse hyperbolic tangent of ℝ(n).
        n.atanh()
    }

    /// # 21.3.2.8 Math.atan2 ( y, x )
    /// https://262.ecma-international.org/#sec-math.atan2
    ///
    /// This function returns the inverse tangent of the quotient y / x of the arguments y and x, where the signs of y and x are used to determine the quadrant of the result. Note that it is intentional and traditional for the two-argument inverse tangent function that the argument named y be first and the argument named x be second. The result is expressed in radians and is in the inclusive interval from -π to +π.
    fn atan2(ny: f64, nx: f64) -> f64 {

        // 1. Let ny be ? ToNumber(y).
        // 2. Let nx be ? ToNumber(x).


        // If ny is NaN or nx is NaN, return NaN.
        // 4. If ny is +∞𝔽, then
        //
        //     a. If nx is +∞𝔽, return an implementation-approximated Number value representing π / 4.
        //     b. If nx is -∞𝔽, return an implementation-approximated Number value representing 3π / 4.
        //     c. Return an implementation-approximated Number value representing π / 2.
        //
        // 5. If ny is -∞𝔽, then
        //
        //     a. If nx is +∞𝔽, return an implementation-approximated Number value representing -π / 4.
        //     b. If nx is -∞𝔽, return an implementation-approximated Number value representing -3π / 4.
        //     c. Return an implementation-approximated Number value representing -π / 2.
        //
        // 6. If ny is +0𝔽, then
        //
        //     a. If nx > +0𝔽 or nx is +0𝔽, return +0𝔽.
        //     b. Return an implementation-approximated Number value representing π.
        //
        // 7. If ny is -0𝔽, then
        //
        //     a. If nx > +0𝔽 or nx is +0𝔽, return -0𝔽.
        //     b. Return an implementation-approximated Number value representing -π.
        //
        // 8. Assert: ny is finite and is neither +0𝔽 nor -0𝔽.
        // 9. If ny > +0𝔽, then
        //
        //    a. If nx is +∞𝔽, return +0𝔽.
        //    b. If nx is -∞𝔽, return an implementation-approximated Number value representing π.
        //    c. If nx is either +0𝔽 or -0𝔽, return an implementation-approximated Number value representing π / 2.
        //
        // 10. If ny < -0𝔽, then
        //
        //    a. If nx is +∞𝔽, return -0𝔽.
        //    b. If nx is -∞𝔽, return an implementation-approximated Number value representing -π.
        //    c. If nx is either +0𝔽 or -0𝔽, return an implementation-approximated Number value representing -π / 2.
        //
        // 11. Assert: nx is finite and is neither +0𝔽 nor -0𝔽.
        // 12. Let r be the inverse tangent of abs(ℝ(ny) / ℝ(nx)).
        // 13. If nx < -0𝔽, then
        //
        //    a. If ny > +0𝔽, set r to π - r.
        //    b. Else, set r to -π + r.
        //
        // 14. Else,
        //
        //    a. If ny < -0𝔽, set r to -r.
        //
        // 15. Return an implementation-approximated Number value representing r.
        ny.atan2(nx)
    }


    /// # 21.3.2.9 Math.cbrt ( x )
    /// https://262.ecma-international.org/#sec-math.cbrt
    ///
    /// This function returns the cube root of x.
    fn cbrt(n: f64) -> f64 {
        // 1. Let n be ? ToNumber(x).

        // 2. If n is not finite or n is either +0𝔽 or -0𝔽, return n.
        // 3. Return an implementation-approximated Number value representing the cube root of ℝ(n).
        n.cbrt()
    }

    /// # 21.3.2.10 Math.ceil ( x )
    /// https://262.ecma-international.org/#sec-math.ceil
    ///
    /// This function returns the smallest (closest to -∞) integral Number value that is not less than x. If x is already an integral Number, the result is x.
    fn ceil(n: f64) -> f64 {
        // 1. Let n be ? ToNumber(x).

        // 2. If n is not finite or n is either +0𝔽 or -0𝔽, return n.
        // 3. If n < -0𝔽 and n > -1𝔽, return -0𝔽.
        // 4. If n is an integral Number, return n.
        // 5. Return the smallest (closest to -∞) integral Number value that is not less than n.
        n.ceil()
    }

    /// # 21.3.2.11 Math.clz32 ( x )
    /// https://262.ecma-international.org/#sec-math.clz32
    const fn clz32(value: f64) -> f64 {
        // Let n be ? ToUint32(x). - TODO: this should be implemented after the spec
        // 2. 2. Let p be the number of leading zero bits in the unsigned 32-bit binary representation of n.
        // 3. 3. Return 𝔽(p).

        if value.is_infinite() {
            return 32.0;
        }

        (value as i64 as u32).leading_zeros() as f64
    }

    /// # 21.3.2.12 Math.cos ( x )
    /// https://262.ecma-international.org/#sec-math.cos
    ///
    /// This function returns the cosine of x. The argument is expressed in radians.
    fn cos(n: f64) -> f64 {
        // 1. Let n be ? ToNumber(x).

        // 2. If n is not finite, return NaN.
        // 3. If n is either +0𝔽 or -0𝔽, return 1𝔽.
        // 4. Return an implementation-approximated Number value representing the cosine of ℝ(n).
        n.cos()
    }

    /// # 21.3.2.13 Math.cosh ( x )
    /// https://262.ecma-international.org/#sec-math.cosh
    ///
    /// This function returns the hyperbolic cosine of x.
    fn cosh(n: f64) -> f64 {
        // 1. Let n be ? ToNumber(x).

        // 2. If n is NaN, return NaN.
        // 3. If n is either +∞𝔽 or -∞𝔽, return +∞𝔽.
        // 4. If n is either +0𝔽 or -0𝔽, return 1𝔽.
        // 5. Return an implementation-approximated Number value representing the hyperbolic cosine of ℝ(n).
        n.cosh()
    }

    /// # 21.3.2.14 Math.exp ( x )
    /// https://262.ecma-international.org/#sec-math.exp
    ///
    /// This function returns the exponential function of x (e raised to the power of x, where e is the base of the natural logarithms).
    fn exp(n: f64) -> f64 {
        // 1. Let n be ? ToNumber(x).

        // 2. If n is either NaN or +∞𝔽, return n.
        // 3. If n is either +0𝔽 or -0𝔽, return 1𝔽.
        // 4. If n is -∞𝔽, return +0𝔽.
        // 5. Return an implementation-approximated Number value representing the exponential function of ℝ(n).
        n.exp()
    }

    /// # 21.3.2.15 Math.expm1 ( x )
    /// https://262.ecma-international.org/#sec-math.expm1
    ///
    /// This function returns the result of subtracting 1 from the exponential function of x (e raised to the power of x, where e is the base of the natural logarithms). The result is computed in a way that is accurate even when the value of x is close to 0.
    fn expm1(n: f64) -> f64 {
        // 1. Let n be ? ToNumber(x).

        // 2. If n is one of NaN, +0𝔽, -0𝔽, or +∞𝔽, return n.
        // 3. If n is -∞𝔽, return -1𝔽.
        // 4. Let exp be the exponential function of ℝ(n).
        // 5. Return an implementation-approximated Number value representing exp - 1.
        n.exp_m1()
    }

    /// # 21.3.2.16 Math.floor ( x )
    /// https://262.ecma-international.org/#sec-math.floor
    ///
    /// This function returns the greatest (closest to +∞) integral Number value that is not greater than x. If x is already an integral Number, the result is x.
    fn floor(n: f64) -> f64 {
        // 1. Let n be ? ToNumber(x).

        // 2. If n is not finite or n is either +0𝔽 or -0𝔽, return n.
        // 3. If n < 1𝔽 and n > +0𝔽, return +0𝔽.
        // 4. If n is an integral Number, return n.
        // 5. Return the greatest (closest to +∞) integral Number value that is not greater than n.
        n.floor()
    }

    /// # 21.3.2.17 Math.fround ( x )
    /// https://262.ecma-international.org/#sec-math.fround
    const fn fround(n: f64) -> f64 {
        // 1. Let n be ? ToNumber(x).
        // 2. 2. If n is NaN, return NaN.
        // 3. 3. If n is one of +0𝔽, -0𝔽, +∞𝔽, or -∞𝔽, return n.
        // 4. 4. Let n32 be the result of converting n to IEEE 754-2019 binary32 format using roundTiesToEven mode.
        // 5. 5. Let n64 be the result of converting n32 to IEEE 754-2019 binary64 format.
        // 6. 6. Return the ECMAScript Number value corresponding to n64.
        n as f32 as f64
    }

    /// # 21.3.2.18 Math.f16round ( x )
    /// https://262.ecma-international.org/#sec-math.f16round
    fn f16round(n: f64) -> f64 {
        // 1. Let n be ? ToNumber(x).

        // 2. If n is NaN, return NaN.
        // 3. If n is one of +0𝔽, -0𝔽, +∞𝔽, or -∞𝔽, return n.
        // 4. Let n16 be the result of converting n to IEEE 754-2019 binary16 format using roundTiesToEven mode.
        // 5. Let n64 be the result of converting n16 to IEEE 754-2019 binary64 format.
        // 6. Return the ECMAScript Number value corresponding to n64.
        half::f16::from_f64(n).to_f64()
    }


    /// # 21.3.2.19 Math.hypot ( ...args )
    /// https://262.ecma-international.org/#sec-math.hypot
    ///
    /// Given zero or more arguments, this function returns the square root of the sum of squares of its arguments.
    #[length(2)]
    fn hypot(coerced: &[f64]) -> f64 {
        // 1. Let coerced be a new empty List.
        // 2. For each element arg of args, do
        //     a. Let n be ? ToNumber(arg).
        //     b. Append n to coerced.


        // 3. For each element number of coerced, do
        //     a. a. If number is either +∞𝔽 or -∞𝔽, return +∞𝔽.
        if coerced.iter().copied().any(f64::is_infinite) {
            return f64::INFINITY;
        }

        // 4. Let onlyZero be true.
        let mut only_zero = true;

        // 5. For each element number of coerced, do
        for &number in coerced {
            // a. If number is NaN, return NaN.
            if number.is_nan() {
                return f64::NAN;
            }
            // b. If number is neither +0𝔽 nor -0𝔽, set onlyZero to false.
            if number != 0.0 && number != -0.0 {
                only_zero = false;
            }
        }

        // 6. If onlyZero is true, return +0𝔽.
        if only_zero {
            return 0.0;
        }

        // Return an implementation-approximated Number value representing the square root of the sum of squares of the mathematical values of the elements of coerced.
        coerced.iter().map(|&n| n * n).sum::<f64>().sqrt()
    }

    /// # 21.3.2.20 Math.imul ( x, y )
    /// https://262.ecma-international.org/#sec-math.imul
    ///
    /// This function performs the following steps when called:
    const fn imul(x: f64, y: f64) -> i32 {
        // 1. Let a be ℝ(? ToUint32(x)).
        let a = x as i64 as u32;
        // 2. Let b be ℝ(? ToUint32(y)).
        let b = y as i64 as u32;

        // 3. Let product be (a × b) modulo 2****32.
        // 4. 4. If product ≥ 2****31, return 𝔽(product - 2****32); otherwise return 𝔽(product).
        a.wrapping_mul(b) as i32
    }

    /// # 21.3.2.21 Math.log ( x )
    /// https://262.ecma-international.org/#sec-math.log
    ///
    /// This function returns the natural logarithm of x.
    fn log(n: f64) -> f64 {
        // 1. Let n be ? ToNumber(x).

        // 2. If n is either NaN or +∞𝔽, return n.
        // 3. If n is 1𝔽, return +0𝔽.
        // 4. If n is either +0𝔽 or -0𝔽, return -∞𝔽.
        // 5. If n < -0𝔽, return NaN.
        // 6. Return an implementation-approximated Number value representing the natural logarithm of ℝ(n).
        n.ln()
    }

    /// # 21.3.2.22 Math.log1p ( x )
    /// https://262.ecma-international.org/#sec-math.log1p
    ///
    /// This function returns the natural logarithm of 1 + x. The result is computed in a way that is accurate even when the value of x is close to zero.
    fn log1p(n: f64) -> f64 {
        // 1. Let n be ? ToNumber(x).

        // 2. If n is one of NaN, +0𝔽, -0𝔽, or +∞𝔽, return n.
        // 3. If n is -1𝔽, return -∞𝔽.
        // 4. If n < -1𝔽, return NaN.
        // 5. Return an implementation-approximated Number value representing the natural logarithm of 1 + ℝ(n).
        n.ln_1p()
    }

    /// # 21.3.2.23 Math.log10 ( x )
    /// https://262.ecma-international.org/#sec-math.log10
    ///
    /// This function returns the base 10 logarithm of x.
    fn log10(n: f64) -> f64 {
        // 1. Let n be ? ToNumber(x).

        // 2. If n is either NaN or +∞𝔽, return n.
        // 3. If n is 1𝔽, return +0𝔽.
        // 4. If n is either +0𝔽 or -0𝔽, return -∞𝔽.
        // 5. If n < -0𝔽, return NaN.
        // 6. Return an implementation-approximated Number value representing the base 10 logarithm of ℝ(n).
        n.log10()
    }


    /// # 21.3.2.24 Math.log2 ( x )
    /// https://262.ecma-international.org/#sec-math.log2
    ///
    /// This function returns the base 2 logarithm of x.
    fn log2(n: f64) -> f64 {
        // 1. Let n be ? ToNumber(x).

        // 2. If n is either NaN or +∞𝔽, return n.
        // 3. If n is 1𝔽, return +0𝔽.
        // 4. If n is either +0𝔽 or -0𝔽, return -∞𝔽.
        // 5. If n < -0𝔽, return NaN.
        // 6. Return an implementation-approximated Number value representing the base 2 logarithm of ℝ(n).
        n.log2()
    }

    /// # 21.3.2.25 Math.max ( ...args )
    /// https://262.ecma-international.org/#sec-math.max
    ///
    /// Given zero or more arguments, this function calls ToNumber on each of the arguments and returns the largest of the resulting values.
    #[length(2)]
    fn max(coerced: &[f64]) -> f64 {
        // 1. Let coerced be a new empty List.
        // 2. For each element arg of args, do
        //     a. Let n be ? ToNumber(arg).
        //     b. Append n to coerced.

        // 3. Let highest be -∞𝔽.
        let mut highest = f64::NEG_INFINITY;

        // 4. For each element number of coerced, do
        for &number in coerced {
            // a. If number is NaN, return NaN.
            if number.is_nan() {
                return f64::NAN;
            }

            //b. If number is +0𝔽 and highest is -0𝔽, set highest to +0𝔽.
            if number == 0.0 && highest == -0.0 {
                highest = 0.0;
            }

            // c. If number > highest, set highest to number.
            if number > highest {
                highest = number;
            }
        }


        // 5. Return highest.
        highest
    }

    #[length(2)]
    fn min(coerced: &[f64]) -> f64 {
        // 1. Let coerced be a new empty List.
        // 2. For each element arg of args, do
        //     a. Let n be ? ToNumber(arg).
        //     b. Append n to coerced.

        // 3. Let lowest be +∞𝔽.
        let mut lowest = f64::INFINITY;

        // 4. For each element number of coerced, do
        for &number in coerced {
            // a. If number is NaN, return NaN.
            if number.is_nan() {
                return f64::NAN;
            }

            //b. If number is -0𝔽 and lowest is +0𝔽, set lowest to -0𝔽.
            if number == -0.0 && lowest == 0.0 {
                lowest = -0.0;
            }

            // c. If number < lowest, set lowest to number.
            if number < lowest {
                lowest = number;
            }
        }

        // 5. Return lowest.
        lowest

    }

    /// # 21.3.2.27 Math.pow ( base, exponent )
    /// https://262.ecma-international.org/#sec-math.pow
    fn pow(base: f64, exponent: f64) -> f64 {
        // 1. Set base to ? ToNumber(base).
        // 2. Set exponent to ? ToNumber(exponent).

        // 3. Return Number::exponentiate(base, exponent). TODO: impl this
        if exponent.is_zero() {
            return 1.0;
        }

        if base.is_nan() || exponent.is_nan() {
            return f64::NAN;
        }

        if base.abs().is_one() && exponent.is_infinite() {
            return f64::NAN;
        }

        base.powf(exponent)
    }

    /// # 21.3.2.28 Math.random ( )
    /// https://262.ecma-international.org/#sec-math.random
    ///
    /// This function returns a Number value with positive sign, greater than or equal to +0𝔽 but strictly less than 1𝔽, chosen randomly or pseudo randomly with approximately uniform distribution over that range, using an implementation-defined algorithm or strategy.
    fn random() -> f64 {
        // Each Math.random function created for distinct realms must produce a distinct sequence of values from successive calls.
        rand::random()
    }

    /// # 21.3.2.29 Math.round ( x )
    /// https://262.ecma-international.org/#sec-math.round
    ///
    /// This function returns the Number value that is closest to x and is integral. If two integral Numbers are equally close to x, then the result is the Number value that is closer to +∞. If x is already integral, the result is x.
    fn round(n: f64) -> f64 {
        // 1. Let n be ? ToNumber(x).

        // 2. If n is not finite or n is an integral Number, return n.
        // 3. If n < 0.5𝔽 and n > +0𝔽, return +0𝔽.
        // 4. If n < -0𝔽 and n ≥ -0.5𝔽, return -0𝔽.
        // 5. Return the integral Number closest to n, preferring the Number closer to +∞ in the case of a tie.
        if (n.fract() + 0.5).abs() < f64::EPSILON {
            return n.ceil();
        }

        n.round()
    }

    /// # 21.3.2.30 Math.sign ( x )
    /// https://262.ecma-international.org/#sec-math.sign
    ///
    /// This function returns the sign of x, indicating whether x is positive, negative, or zero.
    const fn sign(n: f64) -> f64 {
        // 1. Let n be ? ToNumber(x).

        // 2. If n is one of NaN, +0𝔽, or -0𝔽, return n
        if n.is_nan() || n == 0.0 || n == -0.0 {
            return n;
        }

        // 3. If n < -0𝔽, return -1𝔽.
        // 4. Return 1𝔽.
        n.signum()
    }

    /// # 21.3.2.31 Math.sin ( x )
    /// https://262.ecma-international.org/#sec-math.sin
    ///
    /// This function returns the sine of x. The argument is expressed in radians.
    fn sin(n: f64) -> f64 {
        // 1. Let n be ? ToNumber(x).

        // 2. If n is one of NaN, +0𝔽, or -0𝔽, return n.
        // 3. If n is either +∞𝔽 or -∞𝔽, return NaN.
        // 4. Return an implementation-approximated Number value representing the sine of ℝ(n).
        n.sin()
    }

    /// # 21.3.2.32 Math.sinh ( x )
    /// https://262.ecma-international.org/#sec-math.sinh
    ///
    /// This function returns the hyperbolic sine of x.
    fn sinh(n: f64) -> f64 {
        // 1. Let n be ? ToNumber(x).

        // 2. If n is not finite or n is either +0𝔽 or -0𝔽, return n.
        // 3. Return an implementation-approximated Number value representing the hyperbolic sine of ℝ(n).
        n.sinh()
    }

    /// # 21.3.2.33 Math.sqrt ( x )
    /// https://262.ecma-international.org/#sec-math.sqrt
    ///
    /// This function returns the square root of x.
    fn sqrt(n: f64) -> f64 {
        // 1. Let n be ? ToNumber(x).

        // 2. If n is one of NaN, +0𝔽, -0𝔽, or +∞𝔽, return n.
        // 3. If n < -0𝔽, return NaN.
        // 4. Return 𝔽(the square root of ℝ(n)).
        n.sqrt()
    }


    /// # 21.3.2.34 Math.tan ( x )
    /// https://262.ecma-international.org/#sec-math.tan
    ///
    /// This function returns the tangent of x. The argument is expressed in radians.
    fn tan(n: f64) -> f64 {
        // 1. Let n be ? ToNumber(x).

        // 2. If n is one of NaN, +0𝔽, or -0𝔽, return n.
        // 3. If n is either +∞𝔽 or -∞𝔽, return NaN.
        // 4. Return an implementation-approximated Number value representing the tangent of ℝ(n).
        n.tan()
    }

    /// # 21.3.2.35 Math.tanh ( x )
    /// https://262.ecma-international.org/#sec-math.tanh
    ///
    /// This function returns the hyperbolic tangent of x.
    fn tanh(n: f64) -> f64 {
        // 1. Let n be ? ToNumber(x).

        // 2. If n is one of NaN, +0𝔽, or -0𝔽, return n.
        // 3. If n is +∞𝔽, return 1𝔽.
        // 4. If n is -∞𝔽, return -1𝔽.
        // 5. Return an implementation-approximated Number value representing the hyperbolic tangent of ℝ(n).
        n.tanh()
    }

    /// # 21.3.2.36 Math.trunc ( x )
    /// https://262.ecma-international.org/#sec-math.trunc
    ///
    /// This function returns the integral part of the number x, removing any fractional digits. If x is already integral, the result is x.
    fn trunc(n: f64) -> f64 {
        // 1. Let n be ? ToNumber(x).

        // 2. If n is not finite or n is either +0𝔽 or -0𝔽, return n.
        // 3. If n < 1𝔽 and n > +0𝔽, return +0𝔽.
        // 4. If n < -0𝔽 and n > -1𝔽, return -0𝔽.
        // 5. Return the integral Number nearest n in the direction of +0𝔽.
        n.trunc()
    }


    #[prop("sumPrecise")]
    fn sum_precise(iter: &ObjectHandle, #[realm] realm: &mut Realm) -> Res<f64> {
        if !iter.contains_key(Symbol::ITERATOR.into(), realm)? {
            return Err(Error::ty(
                "Value is not iterable: missing @@iterator method",
            ));
        }

        let iter = ValueIterator::new_obj(iter, realm)?;
        let mut sum = xsum::XsumAuto::new();

        while let Some(value) = iter.next(realm)? {
            if let Value::Number(num) = value {
                sum.add(num);
            } else {
                iter.close(realm)?;
                return Err(Error::ty("Iterator value is not a number"));
            }
        }

        iter.close(realm)?;

        Ok(sum.sum())
    }
}

fn float_max(left: f64, right: f64) -> f64 {
    #[allow(clippy::float_cmp)]
    if left > right {
        left
    } else if right > left {
        right
    } else if left == right {
        if left.is_sign_positive() && right.is_sign_negative() {
            left
        } else {
            right
        }
    } else {
        left + right
    }
}

fn float_min(left: f64, right: f64) -> f64 {
    #[allow(clippy::float_cmp)]
    if left < right {
        left
    } else if right < left {
        right
    } else if left == right {
        if left.is_sign_negative() && right.is_sign_positive() {
            left
        } else {
            right
        }
    } else {
        left + right
    }
}

impl Initializer<ObjectHandle> for Math {
    fn initialize(realm: &mut Realm) -> Res<ObjectHandle> {
        Math::new(realm)
    }
}
