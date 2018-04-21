use linenoise;
use std;
use cpu::{Cpu, Register8};

pub fn debug(cpu: &mut Cpu) {
    println!("Entering debugger");
    loop {
        if let Some(input) = linenoise::input("> ") {
            match input.as_str() {
                "exit" => std::process::exit(0),
                "r" => dump_regs(cpu),
                "c" => return,
                "s" => {
                    let _ret = cpu.run_cycle();
                }
                _ => println!("Unrecognized command: {}", input),
            }
        }
    }
}

fn dump_regs(cpu: &Cpu) {
    println!("A: {:#X}   F: {:#X}", cpu[Register8::A], cpu[Register8::F]);
    println!("B: {:#X}   C: {:#X}", cpu[Register8::B], cpu[Register8::C]);
    println!("D: {:#X}   E: {:#X}", cpu[Register8::D], cpu[Register8::E]);
    println!("H: {:#X}   L: {:#X}", cpu[Register8::H], cpu[Register8::L]);
    println!("SP: {:#X}   PC: {:#X}", cpu.sp.0, cpu.pc.0);
}
