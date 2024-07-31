//! The representation of numbers in Reti

use std::{
    fmt::Display,
    num::ParseFloatError,
    ops::{Add, Div, Mul, MulAssign, Sub},
    str::FromStr,
};

///The number representation
#[derive(Debug, Clone, Hash, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "wasm",
    derive(tsify_next::Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
pub struct NumberLiteral(
    ///the raw string without being parsed as a number
    pub String,
);
impl NumberLiteral {
    pub fn is_zero(&self) -> bool {
        self.parse_or_panic("is zero check").abs() < f64::EPSILON
    }

    pub fn is_one(&self) -> bool {
        (self.parse_or_panic("is one check") - 1.0).abs() < f64::EPSILON
    }

    pub fn equals(&self, other: &Self) -> bool {
        (self.parse_or_panic("equals") - other.parse_or_panic("equals")).abs()
            < f64::EPSILON
    }
    pub fn abs(&self) -> NumberLiteral {
        self.parse_or_panic("abs").abs().into()
    }
    pub fn sqrt(&self) -> NumberLiteral {
        self.parse_or_panic("sqrt").sqrt().into()
    }
    pub fn parse_or_panic(&self, msg: &'static str) -> f64 {
        self.0.parse().expect(msg)
    }

    pub(crate) fn pow(&self, exponent: &NumberLiteral) -> NumberLiteral {
        (self
            .parse_or_panic("sub")
            .powf(exponent.parse_or_panic("sub")))
        .into()
    }
}
impl PartialEq for NumberLiteral {
    fn eq(&self, other: &Self) -> bool {
        match (&*self.0, &*other.0) {
            ("-0", "0") => true,
            ("0", "-0") => true,
            (l, r) => l.eq(r),
        }
    }
}
impl Sub for &NumberLiteral {
    type Output = NumberLiteral;

    fn sub(self, rhs: Self) -> Self::Output {
        (self.parse_or_panic("sub") - rhs.parse_or_panic("sub")).into()
    }
}
impl Add for &NumberLiteral {
    type Output = NumberLiteral;

    fn add(self, rhs: Self) -> Self::Output {
        (self.parse_or_panic("add") + rhs.parse_or_panic("add")).into()
    }
}
impl Mul for &NumberLiteral {
    type Output = NumberLiteral;

    fn mul(self, rhs: Self) -> Self::Output {
        (self.parse_or_panic("mul") * rhs.parse_or_panic("mul")).into()
    }
}
impl Div for &NumberLiteral {
    type Output = NumberLiteral;

    fn div(self, rhs: Self) -> Self::Output {
        (self.parse_or_panic("div") / rhs.parse_or_panic("div")).into()
    }
}
impl Display for NumberLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl MulAssign<&Self> for NumberLiteral {
    fn mul_assign(&mut self, rhs: &Self) {
        println!("input self:{}, rhs:{}", self, rhs);
        let mut awns = &*self * rhs;
        println!("awns:{}", awns);
        std::mem::swap(self, &mut awns);
    }
}
impl From<f64> for NumberLiteral {
    fn from(value: f64) -> Self {
        NumberLiteral(value.to_string())
    }
}
impl FromStr for NumberLiteral {
    type Err = ParseFloatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_owned()))
    }
}
impl Sub for NumberLiteral {
    type Output = NumberLiteral;

    fn sub(self, rhs: Self) -> Self::Output {
        (self.parse_or_panic("sub") - rhs.parse_or_panic("sub")).into()
    }
}
impl Add for NumberLiteral {
    type Output = NumberLiteral;

    fn add(self, rhs: Self) -> Self::Output {
        (self.parse_or_panic("add") + rhs.parse_or_panic("add")).into()
    }
}
impl Mul for NumberLiteral {
    type Output = NumberLiteral;

    fn mul(self, rhs: Self) -> Self::Output {
        (self.parse_or_panic("mul") * rhs.parse_or_panic("mul")).into()
    }
}
impl Div for NumberLiteral {
    type Output = NumberLiteral;

    fn div(self, rhs: Self) -> Self::Output {
        (self.parse_or_panic("div") / rhs.parse_or_panic("div")).into()
    }
}

impl From<&str> for NumberLiteral {
    fn from(value: &str) -> Self {
        value.parse::<f64>().expect("failed to parse number").into()
    }
}
impl From<String> for NumberLiteral {
    fn from(value: String) -> Self {
        value.parse::<f64>().expect("failed to parse number").into()
    }
}
impl From<usize> for NumberLiteral {
    fn from(value: usize) -> Self {
        value.to_string().into()
    }
}
#[cfg(test)]
mod test {
    use crate::number_literal::NumberLiteral;
    use pretty_assertions::assert_eq;
    fn num(f: impl Into<NumberLiteral>) -> NumberLiteral {
        f.into()
    }
    #[test]
    fn from_f64_is_zero() {
        assert!(num(0.0).is_zero())
    }
    #[test]
    fn addition() {
        assert_eq!(num(2.0) + num(3.0), num(5.0))
    }
    #[test]
    fn subtraction() {
        assert_eq!(num(2.0) - num(3.0), num(-1.0))
    }
    #[test]
    fn mul() {
        assert_eq!(num(2.0) * num(3.0), num(6.0))
    }
    #[test]
    fn div() {
        assert_eq!(num(2.0) / num(3.0), num(2.0 / 3.0))
    }
    #[test]
    fn abs() {
        assert_eq!(num(-2.0).abs(), num(2.0))
    }
    #[test]
    fn pow() {
        assert_eq!(num(2.0).pow(&num(3.0)), num(8.0))
    }
    #[test]
    fn sqrt() {
        assert_eq!(num(16.0).sqrt(), num(4.0))
    }
}
