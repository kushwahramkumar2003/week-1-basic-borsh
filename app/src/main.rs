use serialize_macro::{DeserializeNumberStruct, SerializeNumberStruct};
use serialize_macro_traits::{Deserialize, Serialize};
use std::fmt::Error;

#[derive(SerializeNumberStruct, DeserializeNumberStruct)]
struct Swap {
    qty_1: i32,
    qty_2: usize,
    qty_3: i8,
}

fn main() {
    println!("Hello, world!");
    let s = Swap {
        qty_1: 1,
        qty_2: 100,
        qty_3: -5,
    };
    let bytes = s.serialize();
    println!("{:?}", bytes);

    let deserialized = Swap::deserialize(&bytes).unwrap();
    println!(
        "qty_1={}, qty_2={}, qty_3={}",
        deserialized.qty_1, deserialized.qty_2, deserialized.qty_3
    );
}
