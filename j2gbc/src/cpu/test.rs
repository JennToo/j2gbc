use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Cursor;
use std::time::Duration;

use super::{Arith, Cpu, Instruction, Load, Operand, Register16, Register8};
use crate::alu::Flags;
use crate::audio::NullSink;
use crate::cart::Cart;
use crate::mem::{Address, MemDevice};
use crate::system::System;

const INTIAL_PC: Address = Address(0x0150);
const INITAL_SP: Address = Address(0xFFFE);

// --------------- General Instructions ------------------
#[test]
fn test_nop() {
    let mut cpu = make_test_cpu();

    let i = Instruction::Nop;
    cpu.execute(i).unwrap();
    assert_reg_vals(&cpu, &[]);
    assert_eq!(cpu.pc, INTIAL_PC);
    assert_eq!(cpu.sp, INITAL_SP);
}

#[test]
fn test_ei() {
    let mut cpu = make_test_cpu();
    cpu.interrupt_master_enable = false;

    let i = Instruction::Ei;
    cpu.execute(i).unwrap();
    assert_eq!(cpu.interrupt_master_enable, true);

    assert_reg_vals(&cpu, &[]);
    assert_eq!(cpu.pc, INTIAL_PC);
    assert_eq!(cpu.sp, INITAL_SP);
}

#[test]
fn test_di() {
    let mut cpu = make_test_cpu();
    cpu.interrupt_master_enable = true;

    let i = Instruction::Di;
    cpu.execute(i).unwrap();
    assert_eq!(cpu.interrupt_master_enable, false);

    assert_reg_vals(&cpu, &[]);
    assert_eq!(cpu.pc, INTIAL_PC);
    assert_eq!(cpu.sp, INITAL_SP);
}

#[test]
fn test_halt() {
    let mut cpu = make_test_cpu();
    cpu.halted = false;

    let i = Instruction::Halt;
    cpu.execute(i).unwrap();
    assert_eq!(cpu.halted, true);

    assert_reg_vals(&cpu, &[]);
    assert_eq!(cpu.pc, INTIAL_PC);
    assert_eq!(cpu.sp, INITAL_SP);
}

#[test]
fn test_scf() {
    let mut cpu = make_test_cpu();
    cpu[Register8::F] = Flags(0).zero().0;

    let i = Instruction::Scf;
    cpu.execute(i).unwrap();

    assert_reg_vals(&cpu, &[(Register8::F, Flags(0).carry().zero().0)]);
    assert_eq!(cpu.pc, INTIAL_PC);
    assert_eq!(cpu.sp, INITAL_SP);
}

#[test]
fn test_cpi() {
    let mut cpu = make_test_cpu();
    cpu[Register8::A] = 0x3C;

    let i = Instruction::Cp(Operand::Immediate(0x3C));
    cpu.execute(i).unwrap();

    assert_reg_vals(
        &cpu,
        &[
            (Register8::A, 0x3C),
            (Register8::F, Flags(0).subtract().zero().0),
        ],
    );
    assert_eq!(cpu.pc, INTIAL_PC);
    assert_eq!(cpu.sp, INITAL_SP);
}

#[test]
fn test_cpr() {
    let mut cpu = make_test_cpu();
    cpu[Register8::A] = 0x3C;
    cpu[Register8::B] = 0x2F;

    let i = Instruction::Cp(Operand::Register(Register8::B));
    cpu.execute(i).unwrap();

    assert_reg_vals(
        &cpu,
        &[
            (Register8::A, 0x3C),
            (Register8::B, 0x2F),
            (Register8::F, Flags(0).subtract().halfcarry().0),
        ],
    );
    assert_eq!(cpu.pc, INTIAL_PC);
    assert_eq!(cpu.sp, INITAL_SP);
}

// --------------- Arith Instructions ------------------

#[test]
fn test_addn() {
    let mut cpu = make_test_cpu();
    cpu[Register8::A] = 0x3C;
    cpu[Register8::H] = 0xFF;
    cpu[Register8::L] = 0x80;
    cpu.mmu.write(Address(0xFF80), 0x12).unwrap();

    let i = Instruction::Arith(Arith::Add(Operand::IndirectRegister(Register16::HL)));
    cpu.execute(i).unwrap();

    assert_reg_vals(
        &cpu,
        &[
            (Register8::A, 0x4E),
            (Register8::F, Flags(0).0),
            (Register8::H, 0xFF),
            (Register8::L, 0x80),
        ],
    );
    assert_eq!(cpu.pc, INTIAL_PC);
    assert_eq!(cpu.sp, INITAL_SP);
}

#[test]
fn test_addr() {
    let mut cpu = make_test_cpu();
    cpu[Register8::A] = 0x3A;
    cpu[Register8::B] = 0xC6;

    let i = Instruction::Arith(Arith::Add(Operand::Register(Register8::B)));
    cpu.execute(i).unwrap();

    assert_reg_vals(
        &cpu,
        &[
            (Register8::A, 0x00),
            (Register8::B, 0xC6),
            (Register8::F, Flags(0).zero().halfcarry().carry().0),
        ],
    );
    assert_eq!(cpu.pc, INTIAL_PC);
    assert_eq!(cpu.sp, INITAL_SP);
}

// --------------- Load Instructions ------------------

#[test]
fn test_load_indirect_increment() {
    let mut cpu = make_test_cpu();
    cpu[Register8::A] = 0x3C;
    cpu[Register8::H] = 0xFF;
    cpu[Register8::L] = 0x80;
    cpu.mmu.write(Address(0xFF80), 0x12).unwrap();

    let i = Instruction::Load(Load::LdNA(1));
    cpu.execute(i).unwrap();

    assert_eq!(cpu.mmu.read(Address(0xFF80)).unwrap(), 0x3C);
    assert_reg_vals(
        &cpu,
        &[
            (Register8::A, 0x3C),
            (Register8::F, Flags(0).0),
            (Register8::H, 0xFF),
            (Register8::L, 0x81),
        ],
    );
    assert_eq!(cpu.pc, INTIAL_PC);
    assert_eq!(cpu.sp, INITAL_SP);
}

#[test]
fn test_load_indirect_decrement() {
    let mut cpu = make_test_cpu();
    cpu[Register8::A] = 0x3C;
    cpu[Register8::H] = 0xFF;
    cpu[Register8::L] = 0x80;
    cpu.mmu.write(Address(0xFF80), 0x12).unwrap();

    let i = Instruction::Load(Load::LdNA(-1));
    cpu.execute(i).unwrap();

    assert_eq!(cpu.mmu.read(Address(0xFF80)).unwrap(), 0x3C);
    assert_reg_vals(
        &cpu,
        &[
            (Register8::A, 0x3C),
            (Register8::F, Flags(0).0),
            (Register8::H, 0xFF),
            (Register8::L, 0x7F),
        ],
    );
    assert_eq!(cpu.pc, INTIAL_PC);
    assert_eq!(cpu.sp, INITAL_SP);
}

// --------------- Test helpers ------------------

fn make_test_cpu() -> Cpu {
    let mut v = Vec::new();
    v.resize(1024, 0);
    let mock_cart = Cart::load(Cursor::new(v)).expect("Failed to create mock cart");
    let mut cpu = Cpu::new(mock_cart, Box::new(NullSink), false);
    cpu.pc = INTIAL_PC;
    for (r, v) in reg_defaults().iter() {
        cpu[*r] = *v;
    }

    cpu
}

fn reg_set() -> HashSet<Register8> {
    let mut s = HashSet::new();
    s.insert(Register8::A);
    s.insert(Register8::B);
    s.insert(Register8::C);
    s.insert(Register8::D);
    s.insert(Register8::E);
    s.insert(Register8::F);
    s.insert(Register8::H);
    s.insert(Register8::L);
    s
}

fn reg_defaults() -> HashMap<Register8, u8> {
    let mut m = HashMap::new();
    m.insert(Register8::A, 1);
    m.insert(Register8::B, 2);
    m.insert(Register8::C, 3);
    m.insert(Register8::D, 4);
    m.insert(Register8::E, 5);
    m.insert(Register8::F, 0);
    m.insert(Register8::H, 6);
    m.insert(Register8::L, 7);
    m
}

fn assert_reg_vals(cpu: &Cpu, vals: &[(Register8, u8)]) {
    let mut regs = reg_set();
    let defaults = reg_defaults();

    for &(r, v) in vals.iter() {
        println!("Checking register {}", r);
        assert_eq!(cpu[r], v);
        regs.remove(&r);
    }

    for r in regs.iter() {
        println!("Checking register (default) {}", r);
        assert_eq!(cpu[*r], *defaults.get(&r).unwrap());
    }
}

// --------------- Blarg tests ---------------------

#[test]
fn test_blarg_01_special() {
    run_blarg_test(
        "test_resources/cpu_instrs/individual/01-special.gb",
        3,
        b"Passed",
        Address(0x9860),
    );
}

#[test]
fn test_blarg_02_interrupts() {
    run_blarg_test(
        "test_resources/cpu_instrs/individual/02-interrupts.gb",
        1,
        b"Passed",
        Address(0x9860),
    );
}

#[test]
fn test_blarg_03_op_sp_hl() {
    run_blarg_test(
        "test_resources/cpu_instrs/individual/03-op sp,hl.gb",
        3,
        b"Passed",
        Address(0x9860),
    );
}

#[test]
fn test_blarg_04_op_r_imm() {
    run_blarg_test(
        "test_resources/cpu_instrs/individual/04-op r,imm.gb",
        3,
        b"Passed",
        Address(0x9860),
    );
}

#[test]
fn test_blarg_05_op_rp() {
    run_blarg_test(
        "test_resources/cpu_instrs/individual/05-op rp.gb",
        4,
        b"Passed",
        Address(0x9860),
    );
}

#[test]
fn test_blarg_06_ld_r_r() {
    run_blarg_test(
        "test_resources/cpu_instrs/individual/06-ld r,r.gb",
        1,
        b"Passed",
        Address(0x9860),
    );
}

#[test]
fn test_blarg_07_jmp_etc() {
    run_blarg_test(
        "test_resources/cpu_instrs/individual/07-jr,jp,call,ret,rst.gb",
        1,
        b"Passed",
        Address(0x9880),
    );
}

#[test]
fn test_blarg_08_misc() {
    run_blarg_test(
        "test_resources/cpu_instrs/individual/08-misc instrs.gb",
        1,
        b"Passed",
        Address(0x9860),
    );
}

#[test]
fn test_blarg_09_op_r_r() {
    run_blarg_test(
        "test_resources/cpu_instrs/individual/09-op r,r.gb",
        10,
        b"Passed",
        Address(0x9860),
    );
}

#[test]
fn test_blarg_10_bit_ops() {
    run_blarg_test(
        "test_resources/cpu_instrs/individual/10-bit ops.gb",
        14,
        b"Passed",
        Address(0x9860),
    );
}

#[test]
fn test_blarg_11_op_a_hl() {
    run_blarg_test(
        "test_resources/cpu_instrs/individual/11-op a,(hl).gb",
        18,
        b"Passed",
        Address(0x9860),
    );
}

fn run_blarg_test(path: &str, sec_to_run: u64, expected: &[u8], expected_addr: Address) {
    let cart_file = File::open(path).unwrap();
    let cart = Cart::load(cart_file).unwrap();
    let cpu = Cpu::new(cart, Box::new(NullSink), false);
    let mut system = System::new(cpu);

    system.run_for_duration(&Duration::from_secs(sec_to_run));

    for (i, e) in expected.iter().enumerate() {
        let addr = expected_addr + Address(i as u16);
        assert_eq!(system.cpu.mmu.read(addr).unwrap(), *e);
    }
}
