use anyhow::{Context, Result, anyhow};
use sim8086::cpu::Cpu;
use sim8086::decoder::Decoder;
use sim8086::disassembler::Disassembler;
use sim8086::memory::{Memory, MemoryAccess};
use std::env;

// Instructions encoding table is around page 256 of the 8086 user's manual.
// 8086 nice tutorial: https://yassinebridi.github.io/asm-docs/
// 8086 cool simulator: https://yjdoc2.github.io/8086-emulator-web/compile

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    // First arg is the path of the executable, we want the others.
    if args.len() <= 1 {
        return Err(anyhow!("binary file is required"));
    }

    let path = args
        .get(1)
        .ok_or_else(|| anyhow!("program binary file is required"))?;

    let simulate_flag = args.get(2);
    let should_simulate = match simulate_flag {
        Some(sim_flag_value) => {
            if sim_flag_value == "--simulate" {
                true
            } else {
                false
            }
        }
        None => false,
    };

    let memory = Memory::load_program_binary(path)?;
    let mut memory_access = MemoryAccess::new();
    let mut disassembler = Disassembler::new();

    disassembler.add_bits_16_header();
    loop {
        let (instruction, new_memory_access) = {
            // We borrow Memory instance to the Decoder instance...
            // The decoder instance just lives during this scope, and then
            // is droped, releasing the borrowed memory so it can be mutable borrowed after.
            Decoder::new(&memory)
                .decode_machine_code(memory_access)
                .with_context(|| "failed decoding current instruction")?
        };
        disassembler.add_instruction(&instruction);

        memory_access = new_memory_access;

        // If next address don't have program bytes, break
        if memory_access.absolute_address() + 1 > memory.program_size() {
            break;
        }
    }

    disassembler.save_to_file("./result.asm")?;

    if !should_simulate {
        return Ok(());
    }

    let mut cpu = Cpu::new();
    memory_access = MemoryAccess::new(); // Start from the beggining.
    loop {
        let (instruction, new_memory_access) = {
            // We borrow Memory instance to the Decoder instance...
            // The decoder instance just lives during this scope, and then
            // is droped, releasing the borrowed memory so it can be mutable borrowed after.
            Decoder::new(&memory)
                .decode_machine_code(memory_access)
                .with_context(|| "failed decoding current instruction")?
        };

        memory_access = cpu.execute_instruction(instruction, new_memory_access)?;

        // If we reached the end of the program, exit.
        if memory_access.absolute_address() + 1 >= memory.program_size() {
            break;
        }
    }

    println!("{}", cpu.to_string());

    Ok(())
}
