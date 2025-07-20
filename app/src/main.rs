use serialize_macro::{DeserializeNumberStruct, SerializeNumberStruct};
use serialize_macro_traits::{Deserialize, Serialize};
use std::fmt::Error;

#[derive(SerializeNumberStruct, DeserializeNumberStruct)]
struct Swap {
    name: String,
    qty_2: usize,
    qty_3: i8,
}

fn main() {
    println!("Hello, world!");
    let s = Swap {
        name: "Test".to_string(),
        qty_2: 100,
        qty_3: -5,
    };
    let bytes = s.serialize();
    println!("{:?}", bytes);

    let deserialized = Swap::deserialize(&bytes).unwrap();
    println!(
        "name={}, qty_2={}, qty_3={}",
        deserialized.name, deserialized.qty_2, deserialized.qty_3
    );
}
