use std::fs::File;
use std::time::Duration;

use j2gbc::{System, NullSink, debug::Address};

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

conformance_test!{
    name: blarg_01_special,
    path: "gb-test-roms/cpu_instrs/individual/01-special.gb",
    max_runtime: 3,
    expected_value: (0x9860, b"Passed"),
}

conformance_test!{
    name: blarg_02_interrupts,
    path: "gb-test-roms/cpu_instrs/individual/02-interrupts.gb",
    max_runtime: 1,
    expected_value: (0x9860, b"Passed"),
}

conformance_test!{
    name: blarg_03_op_sp_hl,
    path: "gb-test-roms/cpu_instrs/individual/03-op sp,hl.gb",
    max_runtime: 3,
    expected_value: (0x9860, b"Passed"),
}

conformance_test!{
    name: blarg_04_op_r_imm,
    path: "gb-test-roms/cpu_instrs/individual/04-op r,imm.gb",
    max_runtime: 3,
    expected_value: (0x9860, b"Passed"),
}

conformance_test!{
    name: blarg_05_op_rp,
    path: "gb-test-roms/cpu_instrs/individual/05-op rp.gb",
    max_runtime: 4,
    expected_value: (0x9860, b"Passed"),
}

conformance_test!{
    name: blarg_06_ld_r_r,
    path: "gb-test-roms/cpu_instrs/individual/06-ld r,r.gb",
    max_runtime: 1,
    expected_value: (0x9860, b"Passed"),
}

conformance_test!{
    name: blarg_07_jmp_etc,
    path: "gb-test-roms/cpu_instrs/individual/07-jr,jp,call,ret,rst.gb",
    max_runtime: 1,
    expected_value: (0x9880, b"Passed"),
}

conformance_test!{
    name: blarg_08_misc,
    path: "gb-test-roms/cpu_instrs/individual/08-misc instrs.gb",
    max_runtime: 1,
    expected_value: (0x9860, b"Passed"),
}

conformance_test!{
    name: blarg_09_op_r_r,
    path: "gb-test-roms/cpu_instrs/individual/09-op r,r.gb",
    max_runtime: 10,
    expected_value: (0x9860, b"Passed"),
}

conformance_test!{
    name: blarg_10_bit_ops,
    path: "gb-test-roms/cpu_instrs/individual/10-bit ops.gb",
    max_runtime: 14,
    expected_value: (0x9860, b"Passed"),
}

conformance_test!{
    name: blarg_11_op_a_hl,
    path: "gb-test-roms/cpu_instrs/individual/11-op a,(hl).gb",
    max_runtime: 18,
    expected_value: (0x9860, b"Passed"),
}

fn run_conformance_test(path: &str, sec_to_run: u64, expected: &[u8], expected_addr: Address) {
    let cart_file = File::open(path).unwrap();
    let mut system = System::new(cart_file, Box::new(NullSink), false).unwrap();

    system.run_for_duration(&Duration::from_secs(sec_to_run));

    for (i, e) in expected.iter().enumerate() {
        let addr = expected_addr + Address(i as u16);
        assert_eq!(system.debugger().read_mem(addr).unwrap(), *e);
    }
}
