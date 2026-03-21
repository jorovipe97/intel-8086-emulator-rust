use anyhow::{Context, Result, anyhow};
use sim8086::decoder::Decoder;
use sim8086::disassembler::Disassembler;
use sim8086::memory::{Memory, MemoryAccess};
use std::env;

// Instructions encoding table is around page 256 of the 8086 user's manual.

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    // First arg is the path of the executable, we want the others.
    if args.len() <= 1 {
        return Err(anyhow!("binary file is required"));
    }

    let path = &args[1];
    let memory = Memory::load_program_binary(path)?;
    let mut memory_access = MemoryAccess::new();
    // We borrow Memory instance to the Decoder instance..
    let decoder = Decoder::new(&memory);
    let mut disassembler = Disassembler::new();
    // let cpu = Cpu::new();

    disassembler.add_bits_16_header();
    loop {
        if !decoder.has_more_instructions(memory_access) {
            break;
        }

        let (instruction, new_memory_access) = decoder
            .current_instruction(memory_access)
            .with_context(|| "failed decoding current instruction")?;

        disassembler.add_instruction(&instruction);

        // Update memory_access, so on next loop we get next instruction.
        memory_access = new_memory_access;
    }

    disassembler.save_to_file("./result.asm")?;

    Ok(())
}
