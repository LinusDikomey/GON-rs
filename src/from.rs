use std::collections::HashMap;

use arrayvec::ArrayVec;

use crate::{Gon, GonGetError, GonError};

#[derive(Debug)]
pub enum FromGonError {
    Gon(GonError),
    ParseInt(std::num::ParseIntError),
    ParseFloat(std::num::ParseFloatError),
    Parse(Box<dyn std::error::Error>),
    Missing(&'static &'static str),
    ExpectedValue,
    ExpectedArray,
    ExpectedObject,
    InvalidVariant(String),
    InvalidLength { expected: usize, found: usize },
    IndexOutOfBounds(usize),
    UnexpectedValue,
    UnexpectedArray,
    UnexpectedObject,
    UnexpectedVariant(String),
    Other(Box<dyn std::error::Error>),
    Unknown
}
impl std::fmt::Display for FromGonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for FromGonError { }

impl From<GonError> for FromGonError {
    fn from(err: GonError) -> Self {
        Self::Gon(err)
    }
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
impl<E: std::error::Error + 'static> From<GonGetError<E>> for FromGonError {
    fn from(err: GonGetError<E>) -> Self {
        match err {
            GonGetError::UnexpectedValue => FromGonError::UnexpectedValue,
            GonGetError::UnexpectedArray => FromGonError::UnexpectedArray,
            GonGetError::UnexpectedObject => FromGonError::UnexpectedObject,
            GonGetError::IndexOutOfBounds(index) => FromGonError::IndexOutOfBounds(index),
            GonGetError::ConversionFailed(err) => FromGonError::Parse(Box::new(err))
        }
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

#[cfg(test)]
mod tests {
    use super::FromGonError;


    #[test]
    fn err_conversions() -> Result<(), FromGonError> {
        let gon = crate::Gon::parse(r#"
            list [1 2 3]
            map {
                a 1
                b hello
                c 12.5
            }
        "#)?;

        let list_gon: [i32; 3] = gon.try_value("list")?
            .map(|arr| Result::<[i32; 3], FromGonError>::Ok([arr[0].try_get()?, arr[1].try_get()?, arr[2].try_get()?]))
            .unwrap_or(Ok([0, 0, 0]))?;

        assert_eq!(list_gon, [1, 2, 3]);
        
        Ok(())
    }
}