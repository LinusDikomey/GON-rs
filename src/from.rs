use std::collections::HashMap;

use arrayvec::ArrayVec;

use crate::Gon;

#[derive(Debug)]
pub enum FromGonError {
    ParseInt(std::num::ParseIntError),
    ParseFloat(std::num::ParseFloatError),
    Missing(&'static &'static str),
    ExpectedValue,
    ExpectedArray,
    ExpectedObject,
    InvalidVariant(Box<String>),
    InvalidLength { expected: usize, found: usize },
    UnexpectedValue(String)
}
impl From<std::num::ParseIntError> for FromGonError {
    fn from(e: std::num::ParseIntError) -> Self {
        Self::ParseInt(e)
    }
}
impl From<std::num::ParseFloatError> for FromGonError {
    fn from(e: std::num::ParseFloatError) -> Self {
        Self::ParseFloat(e)
    }
}
pub trait FromGon {
    fn from_gon(gon: &Gon) -> Result<Self, FromGonError> where Self: Sized;
}

macro_rules! parse_impls {
    ($($t: ty)*) => {
        $(
            impl FromGon for $t {
                fn from_gon(gon: &Gon) -> Result<Self, FromGonError> {
                    match gon {
                        Gon::Value(val) => Ok(val.parse::<$t>()?),
                        Gon::Object(_) | Gon::Array(_) => Err(FromGonError::ExpectedValue)
                    }
                }
            }
        )*
    };
}

parse_impls!(u8 u16 u32 u64 u128 i8 i16 i32 i64 i128);

impl FromGon for String {
    fn from_gon(gon: &Gon) -> Result<Self, FromGonError> {
        match gon {
            Gon::Value(val) => Ok(val.clone()),
            Gon::Object(_) | Gon::Array(_) => Err(FromGonError::ExpectedValue)
        }
    }
}

impl<T: FromGon, const N: usize> FromGon for [T; N] {
    fn from_gon(gon: &Gon) -> Result<Self, FromGonError>
    where Self: Sized {
        match gon {
            Gon::Object(_) | Gon::Value(_) => Err(FromGonError::ExpectedArray),
            Gon::Array(arr) => {
                if arr.len() != N {
                    return Err(FromGonError::InvalidLength {
                        expected: N,
                        found: arr.len()
                    })
                }
                let array_vec = arr.into_iter().map(|entry| T::from_gon(entry)).collect::<Result<ArrayVec<T, N>, _>>()?;
                // SAFETY: the length is checked to be equal in the if check above. The map also doesn't filter any values.
                Ok(unsafe { array_vec.into_inner_unchecked() })
            }
        }
    }
}

impl<T: FromGon> FromGon for Vec<T> {
    fn from_gon(gon: &Gon) -> Result<Self, FromGonError>
    where Self: Sized {
        match gon {
            Gon::Object(_) | Gon::Value(_) => Err(FromGonError::ExpectedArray),
            Gon::Array(arr) => {
                arr.into_iter().map(|entry| T::from_gon(entry)).collect::<Result<Vec<T>, _>>()
            }
        }
    }
}

impl FromGon for Gon {
    fn from_gon(gon: &Gon) -> Result<Self, FromGonError>
    where Self: Sized {
        Ok(gon.clone())
    }
}

impl<T: FromGon> FromGon for HashMap<String, T> {
    fn from_gon(gon: &Gon) -> Result<Self, FromGonError>
    where Self: Sized {
        match gon {
            Gon::Array(_) | Gon::Value(_) => Err(FromGonError::ExpectedObject),
            Gon::Object(map) => {
                map.iter().map(|(key, val)| Ok((key.clone(), T::from_gon(val)?))).collect::<Result<HashMap<String, T>, _>>()
            }
        }
    }
}