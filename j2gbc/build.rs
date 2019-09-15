#[macro_use]
extern crate serde_derive;

use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Write},
    path::Path,
    process::Command,
};

#[derive(Deserialize)]
struct ConformanceTestData {
    name: String,
    path: String,
    max_runtime: u64,

    pass_value: String,
    pass_addr: u16,
}

fn main() {
    build_conformance_roms();

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let destination = Path::new(&out_dir).join("conformance_test_list.rs");
    let mut output = File::create(&destination).unwrap();

    let mut input_d = Vec::new();
    let mut input_f = File::open("conformance_tests.toml").unwrap();
    input_f.read_to_end(&mut input_d).unwrap();
    let tests: HashMap<String, Vec<ConformanceTestData>> =
        toml::from_slice(input_d.as_slice()).unwrap();

    for test in &tests["test"] {
        write!(
            output,
            r#"
                conformance_test! {{
                    name: {},
                    path: "{}",
                    max_runtime: {},
                    expected_value: ({}, b"{}"),
                }}
            "#,
            test.name, test.path, test.max_runtime, test.pass_addr, test.pass_value
        )
        .unwrap();
    }
}

fn build_conformance_roms() {
    Command::new("make")
        .current_dir("gb-conformance")
        .spawn()
        .expect("Failed to build ROMs");
}
