use anyhow::{Context, Result, anyhow};
use sim8086::cpu::{Cpu, ExecutionResult};
use sim8086::decoder::Decoder;
use sim8086::disassembler::Disassembler;
use sim8086::instructions::encodings::OperationType;
use sim8086::memory::{Memory, MemoryAccess};
use sim8086::reporter::{CpuVersion, Reporter};

use std::env;

// 8086 user's manual table of content:
// - Memory segmentation is described in page 31
// - Instructions clocks in page 71
// - Instructions encoding table is around page 256
// 8086 nice tutorial: https://yassinebridi.github.io/asm-docs/
// 8086 cool simulator: https://yjdoc2.github.io/8086-emulator-web/compile
//
// An Engineer did reverse enginerring of the 8086
// https://www.righto.com/2023/05/8086-processor-group-decode-rom.html
// https://news.ycombinator.com/item?id=35939168
//
// How 8086 microcode works:
// https://www.righto.com/2022/11/how-8086-processors-microcode-engine.html

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    // First arg is the path of the executable, we want the others.
    if args.len() <= 1 {
        return Err(anyhow!("binary file is required"));
    }

    let path = args
        .get(1)
        .ok_or_else(|| anyhow!("program binary file is required"))?;

    // Iterates flags and extract them.
    let mut should_simulate = false;
    let mut should_dump_memory = false;
    let mut should_count_clocks = false;
    for flag in args.iter().map(|f| f.as_str()) {
        match flag {
            "--simulate" => should_simulate = true,
            "--dump-memory" => should_dump_memory = true,
            "--count-clocks" => should_count_clocks = true,
            _ => (),
        }
    }

    if should_count_clocks && !should_simulate {
        return Err(anyhow!("to count clocks you must pass --simulate flag"));
    }

    let mut memory = Memory::load_program_binary(path)?;
    let mut ip_memory_access = MemoryAccess::new(); // The memory access to get the instructions.
    let mut disassembler = Disassembler::new();

    disassembler.add_bits_16_header();
    loop {
        let (instruction, new_ip_memory_access) = {
            // We borrow Memory instance to the Decoder instance...
            // The decoder instance just lives during this scope, and then
            // is droped, releasing the borrowed memory so it can be mutable borrowed after.
            Decoder::new(&memory)
                .decode_machine_code(ip_memory_access)
                .with_context(|| "failed decoding current instruction")?
        };
        disassembler.add_instruction(&instruction);
        disassembler.add_new_line();

        ip_memory_access = new_ip_memory_access;

        // If next address don't have program bytes, break
        if ip_memory_access.absolute_address() + 1 > memory.program_size() {
            break;
        }
    }

    disassembler.save_to_file("./result.asm")?;

    if !should_simulate {
        return Ok(());
    }

    let mut cpu = Cpu::new();
    let mut reporter = Reporter::new(CpuVersion::Intel8086);
    let mut ip_memory_access = MemoryAccess::new(); // Start from the beggining.
    let mut prev_execution_result = ExecutionResult {
        new_ip_memory_access: ip_memory_access,
        flags: cpu.flags,
        condition_branch_taken: false,
    };
    loop {
        let (instruction, new_ip_memory_access) = {
            // We borrow Memory instance to the Decoder instance...
            // The decoder instance just lives during this scope, and then
            // is droped, releasing the borrowed memory so it can be mutable borrowed after.
            Decoder::new(&memory)
                .decode_machine_code(ip_memory_access)
                .with_context(|| "failed decoding current instruction")?
        };

        // If detected a ret instruction, just stop, we do not implement
        // ret, and call instructions.
        if instruction.operation == OperationType::Ret {
            break;
        }

        // We pass a mutable borrow of the memory to the cpu.execute_instruction
        // to support load and store operations.
        let new_execution_result =
            cpu.execute_instruction(instruction, &mut memory, new_ip_memory_access)?;

        // This may change because of jump instructions.
        ip_memory_access = new_execution_result.new_ip_memory_access;

        // Report instruction execution
        reporter.analyze_instruction_execution(
            instruction,
            prev_execution_result,
            new_execution_result,
        );
        prev_execution_result = new_execution_result;

        // If we reached the end of the program, exit.
        if ip_memory_access.absolute_address() + 1 > memory.program_size() {
            break;
        }
    }

    println!("{}", cpu.to_string());
    reporter.save_to_file("./result_simulation.txt")?;

    if should_dump_memory {
        memory.save_to_file("./result.data")?;
    }

    Ok(())
}
