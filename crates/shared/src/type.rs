use crate::error::{Error, Result};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::{ops::Deref, slice, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NonEmpty<T>(T);

impl TryFrom<String> for NonEmpty<String> {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        value.parse()
    }
}

impl FromStr for NonEmpty<String> {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.trim() {
            "" => Err(Error::NonEmpty),
            value => Ok(Self(value.to_string())),
        }
    }
}

impl<S> TryFrom<Vec<S>> for NonEmpty<Vec<NonEmpty<String>>>
where
    S: Into<String>,
{
    type Error = Error;

    fn try_from(value: Vec<S>) -> Result<Self> {
        match value
            .into_iter()
            .map(|s| s.into().try_into())
            .collect::<Result<Vec<NonEmpty<String>>>>()?
        {
            vec if vec.is_empty() => Err(Error::NonEmpty),
            vec => Ok(Self(vec)),
        }
    }
}

impl<T> TryFrom<Vec<T>> for NonEmpty<Vec<T>> {
    type Error = Error;

    fn try_from(value: Vec<T>) -> Result<Self> {
        if value.is_empty() {
            Err(Error::NonEmpty)
        } else {
            Ok(Self(value))
        }
    }
}

impl<I, T> IntoIterator for NonEmpty<I>
where
    I: IntoIterator<Item = T>,
{
    type Item = T;
    type IntoIter = I::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T> NonEmpty<Vec<T>> {
    pub fn iter(&self) -> slice::Iter<'_, T> {
        self.0.iter()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NonNegative<T>(T);

impl TryFrom<Decimal> for NonNegative<Decimal> {
    type Error = Error;

    fn try_from(value: Decimal) -> Result<Self> {
        if value.lt(&Decimal::ZERO) {
            Err(Error::NonNegativeValue(value.to_string()))
        } else {
            Ok(Self(value))
        }
    }
}

impl Deref for NonNegative<Decimal> {
    type Target = Decimal;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
