use linenoise;
use std;
use std::u16;

use super::cpu::{Cpu, Register8};
use super::mem::Address;

pub fn debug(cpu: &mut Cpu) {
    println!("Entering debugger");
    for &(a, i) in &cpu.last_instructions {
        println!("    {}: {}", a, i);
    }
    print_next_instruction(cpu);
    let mut prev_cmd = String::new();
    let mut prev_args = Vec::new();

    loop {
        if let Some(input) = linenoise::input("> ") {
            let mut pieces: Vec<String> = input.as_str().split(' ').map(String::from).collect();
            let cmd = pieces.remove(0);
            let result = if input.as_str() == "" {
                execute_command(&prev_cmd, &prev_args, cpu)
            } else {
                prev_cmd = cmd.clone();
                prev_args = pieces.clone();
                execute_command(&cmd, &pieces, cpu)
            };
            if !result {
                return;
            }
        }
    }
}

fn print_next_instruction(cpu: &mut Cpu) {
    match cpu.fetch_instruction() {
        Result::Ok((i, _)) => println!(" => {}: {}", cpu.pc, i),
        Result::Err(()) => println!("    FAILED TO READ NEXT INSTRUCTION"),
    }
}

fn execute_command(cmd: &str, args: &[String], cpu: &mut Cpu) -> bool {
    match cmd {
        "exit" => std::process::exit(0),
        "r" => dump_regs(cpu),
        "c" => return false,
        "s" => {
            let _ret = cpu.run_cycle();
            print_next_instruction(cpu);
        }
        "w" => {
            let address = Address(u16::from_str_radix(args[0].as_str(), 16).unwrap());
            cpu.mmu.watchpoints.insert(address);
        }
        "uw" => {
            let address = Address(u16::from_str_radix(args[0].as_str(), 16).unwrap());
            cpu.mmu.watchpoints.remove(&address);
        }
        "b" => {
            let address = Address(u16::from_str_radix(args[0].as_str(), 16).unwrap());
            cpu.breakpoints.insert(address);
        }
        _ => println!("Unrecognized command: {}", cmd),
    }

    true
}

fn dump_regs(cpu: &Cpu) {
    println!(
        " A: 0x{:02x}   F: 0x{:02x}    SP: {}",
        cpu[Register8::A],
        cpu[Register8::F],
        cpu.sp
    );
    println!(
        " B: 0x{:02x}   C: 0x{:02x}    PC: {}",
        cpu[Register8::B],
        cpu[Register8::C],
        cpu.pc
    );
    println!(
        " D: 0x{:02x}   E: 0x{:02x}   IME: {}",
        cpu[Register8::D],
        cpu[Register8::E],
        cpu.interrupt_master_enable
    );
    println!(
        " H: 0x{:02x}   L: 0x{:02x}",
        cpu[Register8::H],
        cpu[Register8::L]
    );
}
