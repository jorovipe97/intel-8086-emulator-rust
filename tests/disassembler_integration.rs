use anyhow::{Context, Result};
use indoc::indoc;
use sim8086::decoder::Decoder;
use sim8086::disassembler::Disassembler;
use sim8086::memory::{Memory, MemoryAccess};

// Dissemble register to register
#[test]
fn disassemble_register_to_register() -> Result<()> {
    let memory = Memory::load_program_binary("listings_asm/listing_0038_many_register_mov")?;
    let mut memory_access = MemoryAccess::new();

    let decoder = Decoder::new(&memory);
    let mut disassembler = Disassembler::new();

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

    let result = disassembler.build();

    // We use indoc to remove tabs
    let expected = indoc!(
        "bits 16
        mov cx, bx
        mov ch, ah
        mov dx, bx
        mov si, bx
        mov bx, di
        mov al, cl
        mov ch, ch
        mov bx, ax
        mov bx, si
        mov sp, di
        mov bp, ax\n"
    );
    assert_eq!(result, expected);

    Ok(())
}

// Dissemble all mov modes
#[test]
fn disassemble_memory_to_register() -> Result<()> {
    let memory = Memory::load_program_binary("listings_asm/listing_0040_challenge_movs")?;
    let mut memory_access = MemoryAccess::new();

    let decoder = Decoder::new(&memory);
    let mut disassembler = Disassembler::new();

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

    let result = disassembler.build();

    // We use indoc to remove tabs
    let expected = indoc!(
        "bits 16
        mov ax, [bx+di+-37]
        mov [si+-300], cx
        mov dx, [bx+-32]
        mov byte [bp+di], 7
        mov word [di+901], 347
        mov bp, [+5]
        mov bx, [+3458]
        mov ax, [+2555]
        mov ax, [+16]
        mov [+2554], ax
        mov [+15], ax\n"
    );
    assert_eq!(result, expected);

    Ok(())
}
