# GON-rs (Glaiel Object Notation)
A Rust implementation of the Glaiel Object Notation, a json-inspired structured data format "without the crap".
The implementation is inspired by the original [C++ implementation](https://github.com/TylerGlaiel/GON/) by Tyler Glaiel

Some edge cases might still be present

# Features
- Parsing Data
- Error Handling
- Simple data access making use of type inference
- Full JSON compatibility

I might implement more features including writing data, but that isn't the format's primary purpose

# Example

```rust
let gon = gon_rs::Gon::parse(r#"
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
let twirly_widgets: i32 = gon["little_factory"]["twirly_widgets"].get();
assert_eq!(twirly_widgets, 15);
println!("GON: {:#?}", gon);
```

# Use
To use this project as dependency, add this to the dependencies section of your *Cargo.toml*
```toml
gon_rs = { git = "https://github.com/LinusDikomey/GON-rs" }
```