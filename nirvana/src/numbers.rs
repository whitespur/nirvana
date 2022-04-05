use anchor_lang::prelude::*;
use std::{
    convert::TryInto,
    ops::{Add, AddAssign, Div, Mul, Sub, SubAssign},
};

use rust_decimal::Decimal;
pub trait Decimalable {
    fn to_decimal(&self) -> Decimal;
}

#[derive(Default, Clone, Copy, Debug, AnchorDeserialize, AnchorSerialize)]
pub struct ArbitraryNumber {
    pub val: u64,
    pub scale: u8,
}

#[derive(Default, Clone, Copy, Debug, AnchorDeserialize, AnchorSerialize)]
pub struct PreciseNumber {
    pub val: u64,
}

#[derive(Default, Clone, Copy, Debug, AnchorDeserialize, AnchorSerialize)]
pub struct CoarseNumber {
    pub val: u64,
}

#[derive(Default, Debug, Clone, Copy, AnchorDeserialize, AnchorSerialize, PartialEq)]
pub struct ANA {
    pub val: u64,
}

#[derive(Default, Debug, Clone, Copy, AnchorDeserialize, AnchorSerialize, PartialEq)]
pub struct NIRV {
    pub val: u64,
}

#[derive(Default, Debug, Clone, Copy, AnchorDeserialize, AnchorSerialize, PartialEq)]
pub struct ALMS {
    pub val: u64,
}

impl ArbitraryNumber {
    pub fn from_decimal(d: Decimal, scale: u8) -> ArbitraryNumber {
        let mut d = d.to_owned();
        d.rescale(scale.into());

        ArbitraryNumber {
            val: d.mantissa().try_into().unwrap(),
            scale,
        }
    }
    pub fn new(val: u64, scale: u8) -> ArbitraryNumber {
        ArbitraryNumber { val, scale }
    }
}

impl ANA {
    pub const SCALE: u32 = 6;
    pub const DENOM: u64 = 10u64.pow(ANA::SCALE);
    pub const ZERO: ANA = ANA { val: 0 };
    pub const ONE: ANA = ANA { val: 1_000_000 };

    pub fn from_u64(val: u64) -> ANA {
        ANA { val }
    }

    pub fn from_decimal(d: Decimal) -> ANA {
        let mut d = d.to_owned();
        d.rescale(ANA::SCALE);
        ANA {
            val: d.mantissa().try_into().unwrap(),
        }
    }

    pub fn new(n: u64) -> ANA {
        ANA {
            val: n.checked_mul(ANA::DENOM).unwrap(),
        }
    }
}

impl NIRV {
    pub const SCALE: u32 = 6;
    pub const DENOM: u64 = 10u64.pow(NIRV::SCALE);
    pub const ZERO: NIRV = NIRV { val: 0 };
    pub const ONE: NIRV = NIRV { val: 1_000_000 };

    pub fn from_u64(val: u64) -> NIRV {
        NIRV { val }
    }

    pub fn from_decimal(d: Decimal) -> NIRV {
        let mut d = d.to_owned();
        d.rescale(NIRV::SCALE);
        NIRV {
            val: d.mantissa().try_into().unwrap(),
        }
    }

    pub fn new(n: u64) -> NIRV {
        NIRV {
            val: n.checked_mul(NIRV::DENOM).unwrap(),
        }
    }
}

impl ALMS {
    pub const SCALE: u32 = 6;
    pub const DENOM: u64 = 10u64.pow(ALMS::SCALE);
    pub const ZERO: ALMS = ALMS { val: 0 };
    pub const ONE: ALMS = ALMS { val: 1_000_000 };

    pub fn from_u64(val: u64) -> ALMS {
        ALMS { val }
    }

    pub fn from_decimal(d: Decimal) -> ALMS {
        let mut d = d.to_owned();
        d.rescale(ALMS::SCALE);
        ALMS {
            val: d.mantissa().try_into().unwrap(),
        }
    }

    pub fn new(n: u64) -> ALMS {
        ALMS {
            val: n.checked_mul(ALMS::DENOM).unwrap(),
        }
    }
}

impl PreciseNumber {
    pub const SCALE: u32 = 12;
    pub const ZERO: PreciseNumber = PreciseNumber { val: 0 };
    pub const DENOMINATOR: u64 = 10u64.pow(PreciseNumber::SCALE);

    pub fn from_decimal(d: Decimal) -> PreciseNumber {
        let mut d = d.to_owned();
        d.rescale(PreciseNumber::SCALE);
        PreciseNumber {
            val: d.mantissa().try_into().unwrap(),
        }
    }
    pub fn new(n: u64) -> PreciseNumber {
        PreciseNumber {
            val: n.checked_mul(PreciseNumber::DENOMINATOR).unwrap(),
        }
    }
}

impl CoarseNumber {
    pub const SCALE: u32 = 6;
    pub const DENOMINATOR: u64 = 10u64.pow(CoarseNumber::SCALE);

    pub fn from_u64(val: u64) -> CoarseNumber {
        CoarseNumber { val }
    }
}

impl Decimalable for ArbitraryNumber {
    fn to_decimal(&self) -> Decimal {
        Decimal::new(self.val.try_into().unwrap(), self.scale.into())
    }
}

impl Decimalable for PreciseNumber {
    fn to_decimal(&self) -> Decimal {
        Decimal::new(self.val.try_into().unwrap(), PreciseNumber::SCALE)
    }
}

impl Decimalable for CoarseNumber {
    fn to_decimal(&self) -> Decimal {
        Decimal::new(self.val.try_into().unwrap(), CoarseNumber::SCALE)
    }
}

impl Decimalable for ANA {
    fn to_decimal(&self) -> Decimal {
        Decimal::new(self.val.try_into().unwrap(), ANA::SCALE)
    }
}

impl Decimalable for NIRV {
    fn to_decimal(&self) -> Decimal {
        Decimal::new(self.val.try_into().unwrap(), NIRV::SCALE)
    }
}

impl Decimalable for ALMS {
    fn to_decimal(&self) -> Decimal {
        Decimal::new(self.val.try_into().unwrap(), ALMS::SCALE)
    }
}

impl Into<Decimal> for PreciseNumber {
    fn into(self) -> Decimal {
        self.to_decimal()
    }
}

impl Into<Decimal> for CoarseNumber {
    fn into(self) -> Decimal {
        self.to_decimal()
    }
}

impl Into<Decimal> for ANA {
    fn into(self) -> Decimal {
        self.to_decimal()
    }
}

impl Into<Decimal> for NIRV {
    fn into(self) -> Decimal {
        self.to_decimal()
    }
}

impl Into<Decimal> for ALMS {
    fn into(self) -> Decimal {
        self.to_decimal()
    }
}

impl Into<Decimal> for ArbitraryNumber {
    fn into(self) -> Decimal {
        self.to_decimal()
    }
}

impl Into<u64> for ANA {
    fn into(self) -> u64 {
        self.val
    }
}

impl Into<ANA> for u64 {
    fn into(self) -> ANA {
        ANA { val: self }
    }
}

impl Into<u64> for NIRV {
    fn into(self) -> u64 {
        self.val
    }
}

impl Into<NIRV> for u64 {
    fn into(self) -> NIRV {
        NIRV { val: self }
    }
}

impl Into<u64> for ALMS {
    fn into(self) -> u64 {
        self.val
    }
}

impl Into<ALMS> for u64 {
    fn into(self) -> ALMS {
        ALMS { val: self }
    }
}

impl Into<u64> for PreciseNumber {
    fn into(self) -> u64 {
        self.val
    }
}

impl Mul<u64> for CoarseNumber {
    type Output = Self;

    fn mul(self, rhs: u64) -> Self {
        let numerator = self.val * rhs;
        let denominator = CoarseNumber::DENOMINATOR;

        Self {
            val: numerator.checked_div(denominator).unwrap(),
        }
    }
}

impl Mul<CoarseNumber> for ANA {
    type Output = Self;

    fn mul(self, rhs: CoarseNumber) -> Self {
        let num = self.val.checked_mul(rhs.val).unwrap();
        let denom = CoarseNumber::DENOMINATOR;

        Self {
            val: num.checked_div(denom).unwrap(),
        }
    }
}

impl Mul<PreciseNumber> for ANA {
    type Output = Self;

    fn mul(self, rhs: PreciseNumber) -> Self {
        let num = self.val.checked_mul(rhs.val).unwrap();
        let denom = PreciseNumber::DENOMINATOR;

        Self {
            val: num.checked_div(denom).unwrap(),
        }
    }
}

impl Div<PreciseNumber> for ANA {
    type Output = ANA;

    fn div(self, rhs: PreciseNumber) -> ANA {
        Self::new(self.val.mul(PreciseNumber::DENOMINATOR).div(rhs.val))
    }
}

impl Sub<ANA> for ANA {
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output {
        Self {
            val: self.val.checked_sub(other.val).unwrap(),
        }
    }
}

impl Sub<PreciseNumber> for PreciseNumber {
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output {
        Self {
            val: self.val.checked_sub(other.val).unwrap(),
        }
    }
}

impl Add<ANA> for ANA {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        Self {
            val: self.val.checked_add(other.val).unwrap(),
        }
    }
}

impl Add<PreciseNumber> for PreciseNumber {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        Self {
            val: self.val.checked_add(other.val).unwrap(),
        }
    }
}

impl AddAssign<ANA> for ANA {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            val: self.val.checked_add(other.val).unwrap(),
        }
    }
}

impl AddAssign<NIRV> for NIRV {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            val: self.val.checked_add(other.val).unwrap(),
        }
    }
}

impl AddAssign<ALMS> for ALMS {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            val: self.val.checked_add(other.val).unwrap(),
        }
    }
}

impl AddAssign<CoarseNumber> for CoarseNumber {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            val: self.val.checked_add(other.val).unwrap(),
        }
    }
}

impl AddAssign<PreciseNumber> for PreciseNumber {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            val: self.val.checked_add(other.val).unwrap(),
        }
    }
}

impl SubAssign<ANA> for ANA {
    fn sub_assign(&mut self, other: Self) {
        *self = Self {
            val: self.val.checked_sub(other.val).unwrap(),
        }
    }
}

impl SubAssign<NIRV> for NIRV {
    fn sub_assign(&mut self, other: Self) {
        *self = Self {
            val: self.val.checked_sub(other.val).unwrap(),
        }
    }
}

impl SubAssign<ALMS> for ALMS {
    fn sub_assign(&mut self, other: Self) {
        *self = Self {
            val: self.val.checked_sub(other.val).unwrap(),
        }
    }
}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn ana_mul_coarse() {
        let a = ANA { val: 1_000_000 };
        let c = CoarseNumber { val: 500_000 };
        let p = a * c;
        assert_eq!(p.val, 500_000);

        let a = ANA { val: 9_000_000 };
        let c = CoarseNumber { val: 500_000 };
        let p = a * c;
        assert_eq!(p.val, 4_500_000);
    }

    #[test]
    fn ana_sub_assign() {
        let mut a = ANA { val: 50 };

        a -= ANA { val: 25 };
        assert_eq!(a.val, 25);

        a -= ANA { val: 25 };
        assert_eq!(a.val, 0);

        let mut a = ANA { val: 100 };
        a -= ANA { val: 100 };
        assert_eq!(a.val, 0);
    }
}
