use std::{collections::HashMap, fmt::Debug, iter::Peekable, ops::Index, str::{Chars, FromStr}};

#[derive(Debug)]
pub enum GonError {
    InvalidGon,
    StringExpected,
    EndOfFileExpected,
    WhitespaceExpected,
    QuoteExpected,
    ClosingBraceExpected,
    ClosingBracketExpected,
    ValueExpected,
    DuplicateKey(String),
    IO(std::io::Error)
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

    pub fn str(&self) -> &str {
        match self {
            Self::Object(_) => panic!("Tried to get GON object as str!"),
            Self::Array(_) => panic!("Tried to get GON array as str!"),
            Self::Value(val) => val
        }
    }

    pub fn parse(s: &str) -> Result<Self, GonError> {
        let p = &mut s.chars().peekable();
        // the outermost braces are optional
        let gon = if skip_whitespace_and_token('{', p) {
            let gon = parse_object(p)?;
            if !skip_whitespace_and_token('}', p) {
                return Err(GonError::ClosingBraceExpected);
            }
            gon
        } else {
            // This has some ugly edge cases to make parsing of single values work
            let gon = match p.peek() {
                Some('[') => parse_val(p)?,
                _ => match parse_object(p) {
                    Err(GonError::ValueExpected) => {
                        println!("Falling back to parsing value: {}", s);
                        let p = &mut s.chars().peekable();
                        skip_whitespace(p);
                        if let Ok(gon) = parse_val(p) {
                            gon
                        } else {
                            return Err(GonError::InvalidGon);
                        }
                    },
                    res@_ => res?
                }
            };
            skip_whitespace(p);
            gon
        };
        if p.peek().is_some() {
            Err(GonError::EndOfFileExpected)
        } else {
            Ok(gon)
        }
    }
}

type P<'a> = Peekable<Chars<'a>>;

fn parse_object(p: &mut P) -> Result<Gon, GonError> {
    let mut map = HashMap::new();
    while !matches!(p.peek(), Some('}') | None) {
        let key= parse_string(p)?;
        skip_whitespace_and_token(':', p);
        let val = parse_val(p)?;
        if map.get(&key).is_some() {
            return Err(GonError::DuplicateKey(key));
        }
        map.insert(key, val);
        skip_whitespace_and_token(',', p);
    }
    Ok(Gon::Object(map))
}

fn parse_val<'a>(p: &mut P) -> Result<Gon, GonError> {
    match p.peek() {
        Some('{') => {
            p.next();
            skip_whitespace(p);
            let val = parse_object(p)?;
            if !matches!(p.next(), Some('}')) {
                return Err(GonError::ClosingBraceExpected);
            }
            Ok(val)
        },
        Some('[') => {
            p.next();
            let mut arr = Vec::new();
            skip_whitespace(p);
            loop {
                match p.peek() {
                    Some(']') => {
                        p.next();
                        break;
                    },
                    None => return Err(GonError::ClosingBracketExpected),
                    _ => {
                        arr.push(parse_val(p)?);
                        skip_whitespace_and_token(',', p);
                    }
                }
            }
            Ok(Gon::Array(arr))
        }
        Some(_) => parse_string(p).map(|val| Gon::Value(val)),
        None => Err(GonError::ValueExpected)
    }
}

fn parse_string(p: &mut P) -> Result<String, GonError> {
    match p.peek() {
        Some('\"') => {
            p.next();
            let mut res = String::new();
            loop {
                match p.next() {
                    Some('\"') => break,
                    Some(c) => res.push(c),
                    None => return Err(GonError::QuoteExpected)
                }
            }
            Ok(res)
        },
        Some(_) => {
            let mut res = String::new();
            loop {
                match p.peek() {
                    Some(c) if is_whitespace(*c) | is_token(*c) => break,
                    None => break,
                    Some(_) =>res.push(p.next().unwrap()),
                }
            }
            Ok(res)
        },
        None => Err(GonError::StringExpected)
    }
}

fn is_whitespace(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\r')
}

fn is_token(c: char) -> bool {
    matches!(c, '\"' | '{' | '}' |  '[' | ']' | ':' | ',')
}

fn skip_whitespace(p: &mut P<'_>) {
    while p.peek().map_or(false, |c| is_whitespace(*c)) {
        p.next();
    }
}

/// Skips whitespace and and a single optional provided token. Returns if that token was skipped
fn skip_whitespace_and_token(c: char, p: &mut P<'_>) -> bool {
    skip_whitespace(p);
    let skip = p.peek() == Some(&c);
    if skip {
        p.next();
    }
    skip_whitespace(p);
    skip
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
}