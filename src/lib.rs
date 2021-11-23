use std::{collections::HashMap, fmt::Debug, ops::Index, str::FromStr};

use parser::{Parser, StrParser};

mod parser;

#[derive(Debug)]
pub enum GonError {
    InvalidGon,
    StringExpected,
    EndOfFileExpected,
    QuoteExpected,
    ClosingBraceExpected,
    ClosingBracketExpected,
    ValueExpected,
    DuplicateKey(String),
    UnexpectedEscapeCharacter(char),
    EscapeCharacterExpected,
    InvalidHexEscape,
    InvalidUtf8,
    HexEscapesNotSupported
}

#[derive(Debug)]
pub enum GonGetError<E> {
    UnexpectedObject,
    UnexpectedArray,
    UnexpectedValue,
    ConversionFailed(E)
}

#[derive(Debug)]
pub enum Gon {
    Object(HashMap<String, Gon>),
    Array(Vec<Gon>),
    Value(String)
}

impl Index<&str> for Gon {
    type Output = Gon;
    fn index(&self, index: &str) -> &Self::Output {
        match self {
            Self::Object(map) => &map[index],
            Self::Array(_) => panic!("Tried to string-index into GON array!"),
            Self::Value(_) => panic!("Tried to index into GON value!")
        }
    }
}
impl Index<usize> for Gon {
    type Output = Gon;
    fn index(&self, index: usize) -> &Self::Output {
        match self {
            Self::Array(arr) => &arr[index],
            Self::Value(_) => panic!("Tried to int-index into GON value!"),
            Self::Object(_) => panic!("Tried to int-index into GON object!")
        }
    }
}
impl Gon {
    /// Tries to get the GON as a value of a specific type that can be converted from a string.
    /// Will panic on invalid type of object or a conversion fail. Use `try_get`
    pub fn get<T: FromStr>(&self) -> T {
        match self {
            Self::Object(_) => panic!("Tried to get GON object as value!"),
            Self::Array(_) => panic!("Tried to get GON array as value!"),
            Self::Value(val) => {
                match val.parse() {
                    Ok(val) => val,
                    Err(_) => panic!("Failed to parse GON value: {}", val)
                }
            }
        }
    }

    /// Tries to get the GON as a value. In contrast to the `Gon::get` method, this won't panic and will instead return
    /// a `Result`.
    pub fn try_get<T: FromStr>(&self) -> Result<T, GonGetError<<T as FromStr>::Err>> {
        match self {
            Self::Object(_) => Err(GonGetError::UnexpectedObject),
            Self::Array(_) => Err(GonGetError::UnexpectedArray),
            Self::Value(val) => match val.parse() {
                Ok(val) => Ok(val),
                Err(err) => Err(GonGetError::ConversionFailed(err))
            }
        }
    }

    /// Gives a reference to the string if the GON is a string and panics otherwise.
    /// This doesn't copy the string in contrast to the `Gon::get::<String>` method.
    pub fn str(&self) -> &str {
        match self {
            Self::Object(_) => panic!("Tried to get GON object as str!"),
            Self::Array(_) => panic!("Tried to get GON array as str!"),
            Self::Value(val) => val
        }
    }

    /// Returns the size if the GON is an array and panics otherwise.
    pub fn len(&self) -> usize {
        match self {
            Self::Array(arr) => arr.len(),
            Self::Value(_) => panic!("Tried to int-index into GON value!"),
            Self::Object(_) => panic!("Tried to int-index into GON object!")
        }
    }

    pub fn parse(s: &str) -> Result<Self, GonError> {
        let mut p = StrParser::new(s);
        p.skip_whitespace();
        
        // This has some ugly edge cases to make parsing of single values work
        let gon = match p.peek() {
            // Check for object/array
            Some('{') | Some('[') => p.parse_val()?,
            // try to parse as object otherwise because the outermost braces are optional
            _ => match p.parse_object() {
                // if that fails with a 'ValueExpected' error, it might be a single value
                Err(GonError::ValueExpected) => {
                    println!("Falling back to parsing value: {}", s);
                    let mut p = StrParser::new(s);
                    p.skip_whitespace();
                    if let Ok(gon) = p.parse_val() {
                        gon
                    } else {
                        // not an object and not a value, maybe improve the error message
                        return Err(GonError::InvalidGon);
                    }
                },
                res => res?
            }
        };
        p.skip_whitespace();
    
        if p.peek().is_some() {
            Err(GonError::EndOfFileExpected)
        } else {
            Ok(gon)
        }
    }
}

#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn parse_gon() {
        // Examples from https://github.com/TylerGlaiel/GON/blob/master/README.md
        let gon1 = Gon::parse("    
            whirly_widgets 10
            twirly_widgets 15
            girly_widgets 4
            burly_widgets 1
        ").unwrap();
        println!("{:#?}", gon1);
        assert_eq!(gon1["girly_widgets"].get::<i32>(), 4);

        let gon2 = Gon::parse(r#"    
            big_factory {
                location "New York City"
            
                whirly_widgets 8346
                twirly_widgets 854687
                girly_widgets 44336
                burly_widgets 2673
            }
            
            little_factory {
                location "My Basement"
            
                whirly_widgets 10
                twirly_widgets 15
                girly_widgets 4
                burly_widgets 1
            }
        "#).unwrap();
        println!("{:#?}", gon2);
        assert_eq!(gon2["little_factory"]["twirly_widgets"].get::<i32>(), 15);
        
        let gon3 = Gon::parse("    
            weekdays [Monday Tuesday Wednesday Thursday Friday Saturday Sunday]
        ").unwrap();
        println!("{:#?}", gon3);
        assert_eq!(gon3["weekdays"][2].str(), "Wednesday")
    }

    #[test]
    fn json_gon() {
        // Some more tests with valid json which should also parse
        let gon1 = Gon::parse(r#"
        {
            "Accept-Language": "en-US,en;q=0.8",
            "Host": "headers.jsontest.com",
            "Accept-Charset": "ISO-8859-1,utf-8;q=0.7,*;q=0.3",
            "Accept": "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"
         }
        "#).unwrap();
        println!("{:#?}", gon1);
        assert_eq!(gon1["Accept-Charset"].str(), "ISO-8859-1,utf-8;q=0.7,*;q=0.3");

        let gon2 = Gon::parse(r#"
        [
            {
                "_id": "5973782bdb9a930533b05cb2",
                "isActive": true,
                "balance": "$1,446.35",
                "age": 32,
                "eyeColor": "green",
                "name": "Logan Keller",
                "gender": "male",
                "company": "ARTIQ",
                "email": "logankeller@artiq.com",
                "phone": "+1 (952) 533-2258",
                "friends": [
                    {
                        "id": 0,
                        "name": "Colon Salazar"
                    },
                    {
                        "id": 1,
                        "name": "French Mcneil"
                    },
                    {
                        "id": 2,
                        "name": "Carol Martin"
                    }
                ],
                "favoriteFruit": "banana"
            }
        ]
        "#).unwrap();
        println!("Json Gon 2: {:#?}", gon2);
        assert_eq!(gon2[0]["phone"].str(), "+1 (952) 533-2258");
    }

    #[test]
    fn single_values() {
        assert_eq!(Gon::parse("123.456").unwrap().get::<f32>(), 123.456);
        assert_eq!(Gon::parse(r#"
            "Hello World"
        "#).unwrap().str(), "Hello World");

        // This should be recognized as a map, not as a single value:
        let obj = Gon::parse(r#"
            Hello World
        "#).unwrap();
        assert_eq!(obj["Hello"].str(), "World");
    }

    #[test]
    fn escape_codes() {
        assert_eq!(
            Gon::parse(r#""\b \f \n \r \t \" \\ \/""#).unwrap().str(),
            "\x08 \x0C \n \r \t \" \\ /"
        );
    }
}