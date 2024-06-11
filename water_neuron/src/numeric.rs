use crate::{DisplayAmount, E8S};
use candid::CandidType;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::marker::PhantomData;
use std::ops::AddAssign;

#[derive(PartialEq, Eq, Ord, PartialOrd, Clone, Copy)]
pub struct AmountOf<Unit>(pub u64, PhantomData<Unit>);

impl<Unit> Serialize for AmountOf<Unit> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

impl<'de, Unit> Deserialize<'de> for AmountOf<Unit> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        u64::deserialize(deserializer).map(Self::from_e8s)
    }
}

impl<Unit> CandidType for AmountOf<Unit> {
    fn _ty() -> candid::types::Type {
        u64::_ty()
    }

    fn idl_serialize<S>(&self, serializer: S) -> Result<(), S::Error>
    where
        S: candid::types::Serializer,
    {
        self.0.idl_serialize(serializer)
    }
}

impl<C, Unit> minicbor::Encode<C> for AmountOf<Unit> {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        _ctx: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.u64(self.0)?;
        Ok(())
    }
}

impl<'b, C, Unit> minicbor::Decode<'b, C> for AmountOf<Unit> {
    fn decode(
        d: &mut minicbor::Decoder<'b>,
        _ctx: &mut C,
    ) -> Result<Self, minicbor::decode::Error> {
        match d.u64() {
            Ok(n) => Ok(Self::from_e8s(n)),
            Err(e) => Err(e),
        }
    }
}

impl<Unit> AmountOf<Unit> {
    pub const ZERO: Self = Self(0, PhantomData);
    pub const ONE: Self = Self(E8S, PhantomData);
    pub const TWO: Self = Self(2 * E8S, PhantomData);
    pub const MAX: Self = Self(u64::MAX, PhantomData);

    #[inline]
    pub const fn from_e8s(value: u64) -> Self {
        Self(value, PhantomData)
    }

    pub const fn from_unscaled(value: u64) -> Self {
        Self(value * E8S, PhantomData)
    }

    pub fn checked_sub(self, other: Self) -> Option<Self> {
        self.0.checked_sub(other.0).map(Self::from_e8s)
    }
}

#[derive(PartialEq, Eq, Debug, Ord, PartialOrd, Clone, Copy)]
pub enum WTNEnum {}
#[derive(PartialEq, Eq, Debug, Ord, PartialOrd, Clone, Copy)]
pub enum NICPEnum {}
#[derive(PartialEq, Eq, Debug, Ord, PartialOrd, Clone, Copy)]
pub enum ICPEnum {}

pub type ICP = AmountOf<ICPEnum>;
#[allow(non_camel_case_types)]
pub type nICP = AmountOf<NICPEnum>;
pub type WTN = AmountOf<WTNEnum>;

impl<Unit> fmt::Debug for AmountOf<Unit> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AmountOf({})", self.0)
    }
}

impl<Unit> AddAssign for AmountOf<Unit> {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
    }
}

impl<Unit> fmt::Display for AmountOf<Unit> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", DisplayAmount(self.0))
    }
}