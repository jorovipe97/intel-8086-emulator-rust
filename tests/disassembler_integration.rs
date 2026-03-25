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
            .decode_machine_code(memory_access)
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
fn disassemble_mov_modes() -> Result<()> {
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
            .decode_machine_code(memory_access)
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

// Dissemble add, sub, cmp and conditional jumps
#[test]
fn disassemble_add_sub_cmp_jumps() -> Result<()> {
    let memory = Memory::load_program_binary("listings_asm/listing_0041_add_sub_cmp_jnz")?;
    let mut memory_access = MemoryAccess::new();

    let decoder = Decoder::new(&memory);
    let mut disassembler = Disassembler::new();

    disassembler.add_bits_16_header();
    loop {
        if !decoder.has_more_instructions(memory_access) {
            break;
        }

        let (instruction, new_memory_access) = decoder
            .decode_machine_code(memory_access)
            .with_context(|| "failed decoding current instruction")?;

        disassembler.add_instruction(&instruction);

        // Update memory_access, so on next loop we get next instruction.
        memory_access = new_memory_access;
    }

    let result = disassembler.build();

    // We use indoc to remove tabs
    let expected = indoc!(
        "bits 16
        add bx, [bx+si]
        add bx, [bp]
        add si, 2
        add bp, 2
        add cx, 8
        add bx, [bp]
        add cx, [bx+2]
        add bh, [bp+si+4]
        add di, [bp+di+6]
        add [bx+si], bx
        add [bp], bx
        add [bp], bx
        add [bx+2], cx
        add [bp+si+4], bh
        add [bp+di+6], di
        add byte [bx], 34
        add word [bp+si+1000], 29
        add ax, [bp]
        add al, [bx+si]
        add ax, bx
        add al, ah
        add ax, 1000
        add al, -30
        add al, 9
        sub bx, [bx+si]
        sub bx, [bp]
        sub si, 2
        sub bp, 2
        sub cx, 8
        sub bx, [bp]
        sub cx, [bx+2]
        sub bh, [bp+si+4]
        sub di, [bp+di+6]
        sub [bx+si], bx
        sub [bp], bx
        sub [bp], bx
        sub [bx+2], cx
        sub [bp+si+4], bh
        sub [bp+di+6], di
        sub byte [bx], 34
        sub word [bx+di], 29
        sub ax, [bp]
        sub al, [bx+si]
        sub ax, bx
        sub al, ah
        sub ax, 1000
        sub al, -30
        sub al, 9
        cmp bx, [bx+si]
        cmp bx, [bp]
        cmp si, 2
        cmp bp, 2
        cmp cx, 8
        cmp bx, [bp]
        cmp cx, [bx+2]
        cmp bh, [bp+si+4]
        cmp di, [bp+di+6]
        cmp [bx+si], bx
        cmp [bp], bx
        cmp [bp], bx
        cmp [bx+2], cx
        cmp [bp+si+4], bh
        cmp [bp+di+6], di
        cmp byte [bx], 34
        cmp word [+4834], 29
        cmp ax, [bp]
        cmp al, [bx+si]
        cmp ax, bx
        cmp al, ah
        cmp ax, 1000
        cmp al, -30
        cmp al, 9
        jnz byte $+2+2
        jnz byte $+2+-4
        jnz byte $+2+-6
        jnz byte $+2+-4
        je byte $+2+-2
        jl byte $+2+-4
        jle byte $+2+-6
        jb byte $+2+-8
        jbe byte $+2+-10
        jp byte $+2+-12
        jo byte $+2+-14
        js byte $+2+-16
        jnz byte $+2+-18
        jnl byte $+2+-20
        jnle byte $+2+-22
        jnb byte $+2+-24
        jnbe byte $+2+-26
        jnp byte $+2+-28
        jno byte $+2+-30
        jns byte $+2+-32
        loop byte $+2+-34
        loopz byte $+2+-36
        loopnz byte $+2+-38
        jcxz byte $+2+-40\n"
    );
    assert_eq!(result, expected);

    Ok(())
}
