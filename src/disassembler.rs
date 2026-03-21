use crate::instructions::{DecodedInstruction, Operand};
use anyhow::{Context, Result};
use std::{self, fs};

pub struct Disassembler {
    string_builder: String,
}

impl Disassembler {
    pub fn new() -> Self {
        return Self {
            string_builder: String::with_capacity(512),
        };
    }

    /// Tells NASM assembler we intent to run assembly for old 8086 architecture.
    pub fn add_bits_16_header(&mut self) {
        self.string_builder.push_str("bits 16\n");
    }

    /// Add a single line comment.
    // pub fn add_line_comment(&mut self, comment: &str) {
    //     self.string_builder.push_str(comment);
    //     self.string_builder.push('\n');
    // }

    /// Receives a DecodedInstruction and prints its ASM representation.
    pub fn add_instruction(&mut self, instruction: &DecodedInstruction) {
        // Print the mnemonic name.
        self.string_builder
            .push_str(&instruction.operation.to_string());
        self.string_builder.push(' ');

        // When the destination is a register, the assembler infers the size from the register name:
        // MOV AX, [BX]      ; AX is 16-bit → word operation (no specifier needed)
        // MOV AL, [BX]      ; AL is 8-bit → byte operation (no specifier needed)
        // But when the destination is memory, there's no way to know the size:
        // MOV [BX], 5       ; Is this 8-bit or 16-bit? Assembler can't tell!
        // So you MUST specify:
        // MOV byte [BX], 5   ; Store 5 as 8-bit
        // MOV word [BX], 5   ; Store 5 as 16-bit
        //
        // If destination is not register or a insutrction pointer increment do not add size of destination.
        // if !matches!(instruction.operands.destination, Operand::Register(_)) {
        //     // Add word, byte depending on if the instruction is wide or not.
        // }

        self.print_operand(&instruction.operands.destination);

        // Print operand separator if both operands are not Operand::None.
        if !matches!(instruction.operands.destination, Operand::None)
            && !matches!(instruction.operands.source, Operand::None)
        {
            self.string_builder.push_str(", ");
        }

        self.print_operand(&instruction.operands.source);
        self.string_builder.push('\n');
    }

    fn print_operand(&mut self, operand: &Operand) {
        match operand {
            Operand::None => (), // no-op
            Operand::Register(reg) => {
                self.string_builder.push_str(&reg.to_string());
            }
        }
    }

    /// Saves disassembled code into a file in disk.
    pub fn save_to_file(&self, path: &str) -> Result<()> {
        fs::write(path, &self.string_builder)
            .with_context(|| "could not write disassembled file")?;
        Ok(())
    }

    /// If you call this function, you will no longer be able to mutate the internal String.
    /// as if there as an immutable borrow, we cannot longer do mutable borrow.
    ///
    /// This function is intended for integration tests.
    pub fn build(&self) -> &str {
        return &self.string_builder;
    }
}
