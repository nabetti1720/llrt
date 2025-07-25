use llrt_utils::object::ObjectExt;
// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
use rquickjs::{
    atom::PredefinedAtom,
    function::{Opt, This},
    prelude::Func,
    Ctx, Exception, FromJs, Function, Object, Result, Value,
};

const DIGITS: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";
const BUF_SIZE: usize = 80;
const BIN_MAX_DIGITS: usize = 64;
const OCT_MAX_DIGITS: usize = 21;

pub fn redefine_prototype(ctx: &Ctx<'_>) -> Result<()> {
    let globals = ctx.globals();
    let number: Function = globals.get(PredefinedAtom::Number)?;
    let number_proto: Object = number.get(PredefinedAtom::Prototype)?;
    number_proto.set(PredefinedAtom::ToString, Func::from(number_to_string))?;
    Ok(())
}

#[inline(always)]
pub fn to_dec(number: i64) -> String {
    itoa::Buffer::new().format(number).into()
}

#[inline(always)]
pub fn to_base_less_than_10(buf: &mut [u8], num: i64, base: i64) -> String {
    let max = buf.len();

    if num == 0 {
        return "0".into();
    }
    let mut index = max;
    let mut n = num;

    let mut string = String::with_capacity(max + 1);

    if n < 0 {
        n = !n + 1;
        string.push('-');
    }

    while n > 0 {
        index -= 1;
        buf[index] = (n % base) as u8 + b'0';
        n /= base;
    }

    string.push_str(unsafe { std::str::from_utf8_unchecked(&buf[index..max]) });
    string
}

pub fn i64_to_base_n(number: i64, radix: u8) -> String {
    match radix {
        10 => {
            return to_dec(number);
        },
        2 => {
            let mut buf = [0u8; BIN_MAX_DIGITS];
            return to_base_less_than_10(&mut buf, number, 2);
        },
        8 => {
            let mut buf = [0u8; OCT_MAX_DIGITS];
            return to_base_less_than_10(&mut buf, number, 8);
        },
        _ => {},
    }

    let mut abs_number = number;

    let mut buf = [0u8; BUF_SIZE];
    let mut string = String::with_capacity(BUF_SIZE);
    if number < 0 {
        abs_number = -number;
        string.push('-');
    }

    let index = internal_i64_to_base_n(&mut buf, abs_number, radix);

    string.push_str(unsafe { std::str::from_utf8_unchecked(&buf[index..BUF_SIZE]) });
    string
}

#[inline(always)]
fn internal_i64_to_base_n(buf: &mut [u8], number: i64, radix: u8) -> usize {
    if number == 0 {
        buf[BUF_SIZE - 1] = DIGITS[0];
        return BUF_SIZE - 1;
    }

    let mut n = number;
    let mut index = BUF_SIZE;

    while n > 0 {
        index -= 1;
        let digit = n % radix as i64;
        buf[index] = DIGITS[digit as usize];
        n /= radix as i64;
    }

    index
}

#[inline(always)]
fn next_up(num: f64) -> f64 {
    const TINY_BITS: u64 = 0x1;
    const CLEAR_SIGN_MASK: u64 = 0x7fff_ffff_ffff_ffff;

    let bits = num.to_bits();
    if num.is_nan() || bits == f64::INFINITY.to_bits() {
        return num;
    }

    let abs = bits & CLEAR_SIGN_MASK;
    let next_bits = if abs == 0 {
        TINY_BITS
    } else if bits == abs {
        bits + 1
    } else {
        bits - 1
    };
    f64::from_bits(next_bits)
}

#[inline(always)]
fn fractional_to_base(buf: &mut [u8], mut index: usize, mut number: f64, radix: u8) -> usize {
    let mut is_odd = number <= 0x1fffffffffffffi64 as f64 && (number as i64) & 1 != 0;
    let mut digit;

    //let mut needs_rounding_up = false;

    let next_number = next_up(number);
    let mut delta_next_double = next_number - number;

    loop {
        let ntmp = number * radix as f64;
        let rtmp = delta_next_double * radix as f64;
        digit = ntmp as usize;
        let ritmp = rtmp as usize;

        if digit & 1 != 0 {
            is_odd = !is_odd;
        }

        number = ntmp - digit as f64;
        delta_next_double = rtmp - ritmp as f64;

        if number > 0.5f64 || number == 0.5f64 && if radix & 1 > 0 { is_odd } else { digit & 1 > 0 }
        {
            if number + delta_next_double > 1.0 {
                //TODO impl round up
                break;
            }
        } else if number < delta_next_double * 2.0 {
            break;
        }
        buf[index] = DIGITS[digit];

        index += 1;
    }

    // let last_index = index;
    // while number > 0.0 {
    //     let tmp = number * radix as f64;
    //     let itmp = tmp as usize;
    //     buf[index] = DIGITS[itmp];
    //     number = tmp - itmp as f64;
    //     index += 1;
    //     if index - last_index > BUF_SIZE - last_index - 1 {
    //         break;
    //     }
    // }
    index
}

#[inline(always)]
fn f64_to_base_n(number: f64, radix: u8) -> String {
    const EXP_MASK: u64 = 0x7ff0000000000000;
    let bits = number.to_bits();
    if bits & EXP_MASK == EXP_MASK {
        return get_nonfinite(bits).to_string();
    }

    if radix == 10 {
        let mut result = ryu::Buffer::new().format_finite(number).to_string();
        if result.ends_with(".0") {
            result.truncate(result.len() - 2);
        }
        return result;
    }

    let mut abs_num = number;
    let mut string = String::with_capacity(BUF_SIZE);
    let mut buf = [0u8; BUF_SIZE];

    if number < 0.0 {
        abs_num = -number;
        string.push('-');
    }

    let integer_part = abs_num.trunc();
    let fractional_part = abs_num - integer_part;
    let integer_part = abs_num as i64;

    let index = internal_i64_to_base_n(&mut buf, integer_part, radix);
    string.push_str(unsafe { std::str::from_utf8_unchecked(&buf[index..BUF_SIZE]) });

    if fractional_part > 0.0 {
        buf.fill(0);
        let frac_end = fractional_to_base(&mut buf, 0, fractional_part, radix);
        if frac_end > 0 {
            string.push('.');
            string.push_str(unsafe { std::str::from_utf8_unchecked(&buf[0..frac_end]) });
        }
    }
    string
}

pub fn float_to_string(float: f64) -> String {
    f64_to_base_n(float, 10)
}

#[inline(always)]
#[cold]
fn get_nonfinite<'a>(bits: u64) -> &'a str {
    const MANTISSA_MASK: u64 = 0x000fffffffffffff;
    const SIGN_MASK: u64 = 0x8000000000000000;
    if bits & MANTISSA_MASK != 0 {
        "NaN"
    } else if bits & SIGN_MASK != 0 {
        "-Infinity"
    } else {
        "Infinity"
    }
}

#[inline(always)]
#[cold]
fn check_radix(ctx: &Ctx, radix: u8) -> Result<()> {
    if !(2..=36).contains(&radix) {
        return Err(Exception::throw_type(ctx, "radix must be between 2 and 36"));
    }
    Ok(())
}

fn number_to_string<'js>(
    ctx: Ctx<'js>,
    this: This<Value<'js>>,
    radix: Opt<Value>,
) -> Result<String> {
    let radix = radix
        .0
        .and_then(|v| v.as_int())
        .and_then(|r| u8::try_from(r).ok())
        .unwrap_or(10);
    check_radix(&ctx, radix)?;

    // handle primitive number values
    if let Some(float) = this.as_float() {
        return Ok(f64_to_base_n(float, radix));
    }
    if let Ok(int) = i64::from_js(&ctx, this.0.clone()) {
        return Ok(i64_to_base_n(int, radix));
    }

    // handle boxed Number objects
    if let Some(obj) = this.as_object() {
        if let Some(value_of) = obj.get_optional::<_, Function>(PredefinedAtom::ValueOf)? {
            let primitive: Value = value_of.call((this,))?;

            if let Some(float) = primitive.as_float() {
                return Ok(f64_to_base_n(float, radix));
            }
            if let Ok(int) = i64::from_js(&ctx, primitive) {
                return Ok(i64_to_base_n(int, radix));
            }
        }
    }

    Ok("".into())
}

#[cfg(test)]
mod test {

    use rand::Rng;

    use crate::{float_to_string, i64_to_base_n};

    #[test]
    fn test_base_conversions() {
        let mut rng = rand::rng();

        for _ in 0..1_000_000 {
            // Generate random i64 and radix values
            let num: i64 = rng.random_range(i64::MIN + 1..i64::MAX - 1);

            let minus_str = if num < 0 { "-" } else { "" };

            //test bin
            let expected_bin = format!("{}{:b}", minus_str, num.abs());
            let actual_bin = i64_to_base_n(num, 2);
            assert_eq!(expected_bin, actual_bin);

            //test octal
            let expected_octal = format!("{}{:o}", minus_str, num.abs());
            let actual_octal = i64_to_base_n(num, 8);
            assert_eq!(expected_octal, actual_octal);

            //test hex
            let expected_hex = format!("{}{:x}", minus_str, num.abs());
            let actual_hex = i64_to_base_n(num, 16);
            assert_eq!(expected_hex, actual_hex);
        }

        // Test i64_to_base_n
        let base_36 = i64_to_base_n(0, 36);
        assert_eq!("0", base_36);

        let base_36 = i64_to_base_n(123456789, 36);
        assert_eq!("21i3v9", base_36);

        let base_36 = i64_to_base_n(-123456789, 36);
        assert_eq!("-21i3v9", base_36);

        let float = float_to_string(123.456);
        assert_eq!("123.456", float);

        let float = float_to_string(123.);
        assert_eq!("123", float);

        let float = float_to_string(0.0);
        assert_eq!("0", float);
    }
}
