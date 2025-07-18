pub mod pkg;

use anyhow::Error;



fn main() -> Result<(), Error>{
    println!("hello");

    Ok(())
}

// use serde::{de::Serialize, de::Deserialize};
// use std::collections::HashMap;
//
// use ::serde::{Deserialize, Serialize};
// // Import our custom format functions and types from `src/lib.rs`
// use serde::{to_string, from_str, Error};
//
// // Define a struct that we want to serialize and deserialize
// // It must derive Serialize and Deserialize from Serde
// #[derive(Debug, PartialEq, Serialize, Deserialize)]
// struct Product {
//     id: u32,
//     name: String,
//     price: f64,
//     available: bool,
//     tags: Vec<String>,
//     properties: HashMap<String, String>,
// }
//
// fn main() -> Result<(), Error> {
//     // --- 1. Serialization Example ---
//     println!("--- Serialization Example ---");
//
//     let my_product = Product {
//         id: 101,
//         name: "Wireless Headphones".to_string(),
//         price: 99.99,
//         available: true,
//         tags: vec!["audio".to_string(), "electronics".to_string(), "bluetooth".to_string()],
//         properties: {
//             let mut map = HashMap::new();
//             map.insert("color".to_string(), "black".to_string());
//             map.insert("brand".to_string(), "AudioPro".to_string());
//             map.insert("model".to_string(), "X10".to_string());
//             map
//         },
//     };
//
//     // Serialize the Rust struct into our custom format string
//     let serialized_string = to_string(&my_product)?;
//     println!("Original struct: {:?}", my_product);
//     println!("Serialized string:\n{}", serialized_string);
//
//     // Expected Output (map order might vary):
//     // T,101,"Wireless Headphones",99.99,["audio" "electronics" "bluetooth"],{"color":"black" "brand":"AudioPro" "model":"X10"}
//
//
//     // --- 2. Deserialization Example ---
//     println!("\n--- Deserialization Example ---");
//
//     // We need to provide an input string that matches our custom format's rules.
//     // Given our simple parser, it needs to be *exactly* as the serializer outputs it,
//     // and whitespace handling is basic.
//     let input_string = r#"T,101,"Wireless Headphones",99.99,["audio" "electronics" "bluetooth"],{"color":"black" "brand":"AudioPro" "model":"X10"}"#;
//     // Note: The order of key-value pairs in the HashMap portion {"color":"black" "brand":"AudioPro" "model":"X10"}
//     // matters for exact string matching in deserialization with our current simplistic `deserialize_any` and parsing logic.
//     // A robust deserializer would handle arbitrary order and flexible whitespace.
//
//     println!("Input string for deserialization:\n{}", input_string);
//
//     // Deserialize the string back into a Rust struct
//     let deserialized_product: Product = from_str(input_string)?;
//     println!("Deserialized struct: {:?}", deserialized_product);
//
//     // --- 3. Verification ---
//     println!("\n--- Verification ---");
//     if my_product == deserialized_product {
//         println!("Serialization and Deserialization successful: Original and deserialized structs match!");
//     } else {
//         println!("Error: Original and deserialized structs DO NOT match!");
//         println!("Original: {:?}", my_product);
//         println!("Deserialized: {:?}", deserialized_product);
//     }
//
//     Ok(())
// }
