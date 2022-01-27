use gon_rs::{FromGon, from::FromGon};



#[test]
fn derive_test() {
    #[derive(FromGon, PartialEq, Debug)]
    enum AnEnum {
        ValueA,
        ValueB,
        ValueC
    }
    #[derive(FromGon, PartialEq, Debug)]
    struct Example {
        a: i32,
        b: AnEnum
    }
    let gon_str = r#"
    a 5
    b ValueB
    "#;

    let gon = gon_rs::Gon::parse(gon_str).unwrap();
    assert_eq!(Example::from_gon(&gon).unwrap(), Example { a: 5, b: AnEnum::ValueB })
}