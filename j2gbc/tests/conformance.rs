use std::fs::File;
use std::time::Duration;

use j2gbc::{debug::Address, NullSink, System};

macro_rules! conformance_test {
    {
        name: $test_name:ident,
        path: $path:expr,
        max_runtime: $max_runtime:expr,
        expected_value: ($addr:expr, $value:expr),
    } => {
        #[test]
        fn $test_name() {
            run_conformance_test($path, $max_runtime, $value, Address($addr));
        }
    }
}

include!(concat!(env!("OUT_DIR"), "/conformance_test_list.rs"));

fn run_conformance_test(path: &str, sec_to_run: u64, expected: &[u8], expected_addr: Address) {
    let cart_file = File::open(path).unwrap();
    let mut system = System::new(cart_file, Box::new(NullSink), false).unwrap();

    system.run_for_duration(&Duration::from_secs(sec_to_run));

    for (i, e) in expected.iter().enumerate() {
        let addr = expected_addr + Address(i as u16);
        assert_eq!(system.debugger().read_mem(addr).unwrap(), *e);
    }
}
