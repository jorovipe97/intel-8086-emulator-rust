use std::fmt::Display;

use anyhow::{Result, anyhow};

use crate::{
    instructions::{
        decoded_instruction::DecodedInstruction,
        encodings::{OperationType, RegisterName},
        operands::Operand,
    },
    memory::MemoryAccess,
};

pub struct Cpu {
    /// General purpose registers
    registers: [u16; 8],
    /// Instruction pointer register
    instruction_pointer: usize,
    /// Extra Segment (ES), Code Segment (CS), Stack Segment (SS), Data Segment (DS)
    segment_registers: [u16; 4],
    flags: u16,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            registers: [0; 8],
            instruction_pointer: 0,
            segment_registers: [0; 4],
            flags: 0,
        }
    }

    pub fn execute_instruction(
        &mut self,
        instruction: DecodedInstruction,
        memory_access: MemoryAccess,
    ) -> Result<MemoryAccess> {
        let destination_value = self.get_operand_value(instruction.operands.destination)?;
        let source_value = self.get_operand_value(instruction.operands.source)?;

        let final_value: u16 = match instruction.operation {
            OperationType::None => 0,
            OperationType::Mov => source_value,
            OperationType::Add => destination_value + source_value,
            OperationType::Cmp | OperationType::Sub => destination_value - source_value,
            // All jump operations operate on the destination value.
            OperationType::Jb
            | OperationType::Jbe
            | OperationType::Jcxz
            | OperationType::Je
            | OperationType::Jl
            | OperationType::Jle
            | OperationType::Jnb
            | OperationType::Jnbe
            | OperationType::Jne
            | OperationType::Jnl
            | OperationType::Jnle
            | OperationType::Jno
            | OperationType::Jnp
            | OperationType::Jns
            | OperationType::Jnz
            | OperationType::Jo
            | OperationType::Jp
            | OperationType::Js
            | OperationType::Loop
            | OperationType::LoopNz
            | OperationType::LoopZ => destination_value,
        };

        // Computes flags

        // Check if instruction is a cmp, This instructions does not writes to destination
        // operand, just affects flags, this instruction is usually used to control the program
        // execution flow.
        if let OperationType::Cmp = instruction.operation {
            return Ok(memory_access);
        }

        // Updates simulated memory. Destination can be a register or memory.
        match instruction.operands.destination {
            Operand::None => (),
            Operand::Immediate(_) => {
                return Err(anyhow!(
                    "you cannot have an immediate as destination operand"
                ));
            }
            Operand::Memory(_) => {
                return Err(anyhow!("destination memory operand is not supported"));
            }
            Operand::Register(reg) => {
                // TODO: Move to RegisterName function.
                let reg_index = reg.register_name as usize;
                if reg_index >= 8 {
                    return Err(anyhow!("register name is invalid"));
                }

                // If is a byte operand, eg: al, bl, cl, dl, ah, bh, ch, dh
                // then we need to write the appropiate part of the register
                if reg.count == 1 {
                    // Shift value left based on the offset, lower register have a 0 offset
                    // while higher register have an 1 offset.

                    // Ensures corresponding parth of the original register is cleared
                    // Where we have 1 are the places where we want to write
                    let mask: u16 = 0b00000000_11111111;
                    let left_shift: u16 = (reg.offset as u16) * 8;
                    // Resets the part of the register that will be written
                    self.registers[reg_index] = self.registers[reg_index] & !(mask << left_shift);
                    // Write new value there.
                    self.registers[reg_index] =
                        self.registers[reg_index] | ((final_value & mask) << left_shift);
                } else {
                    self.registers[reg_index] = final_value;
                }
            }
            Operand::InstructionPointerIncrement(_) => {
                return Err(anyhow!("instruction pointer not supported yet"));
            }
        }

        Ok(memory_access)
    }

    fn get_operand_value(&self, operand: Operand) -> Result<u16> {
        match operand {
            Operand::None => Ok(0), // Zero value, this is a no-op
            Operand::Immediate(v) => Ok(v as u16),
            Operand::Register(reg) => {
                // Get the index of the register using register name enum's value.
                let reg_index = reg.register_name as usize;
                if reg_index > 8 {
                    return Err(anyhow!("register name in insntruction is invalid"));
                }

                // If is a byte operand, eg: al, bl, cl, dl, ah, bh, ch, dh
                // then we need to write the appropiate part of the register
                if reg.count == 1 {
                    // This is used to remove the part of the register we are not interested in.
                    let mask: u16 = 0b00000000_11111111;
                    let right_shift: u16 = (reg.offset as u16) * 8;
                    return Ok((self.registers[reg_index] >> right_shift) & mask);
                }

                // If reaches here we are in a word operand, eg: ax, bx, cx, dx
                Ok(self.registers[reg_index])
            }
            Operand::InstructionPointerIncrement(ip_inc) => Ok(ip_inc as u16),
            // TODO: How are we going to do an immutable borrow to the memory?
            Operand::Memory(_) => Err(anyhow!("simulator still not supports memory operands")),
        }
    }
}

impl Display for Cpu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "General Purposes Registers:\n")?;
        write!(
            f,
            "\t - AX: {} ({:08b})\n",
            self.registers[RegisterName::A as usize],
            self.registers[RegisterName::A as usize]
        )?;
        write!(
            f,
            "\t - BX: {} ({:08b})\n",
            self.registers[RegisterName::B as usize],
            self.registers[RegisterName::B as usize],
        )?;
        write!(
            f,
            "\t - CX: {} ({:08b})\n",
            self.registers[RegisterName::C as usize],
            self.registers[RegisterName::C as usize]
        )?;
        write!(
            f,
            "\t - DX: {} ({:08b})\n",
            self.registers[RegisterName::D as usize],
            self.registers[RegisterName::D as usize],
        )?;
        write!(
            f,
            "\t - SP: {} ({:08b})\n",
            self.registers[RegisterName::SP as usize],
            self.registers[RegisterName::SP as usize],
        )?;
        write!(
            f,
            "\t - BP: {} ({:08b})\n",
            self.registers[RegisterName::BP as usize],
            self.registers[RegisterName::BP as usize]
        )?;
        write!(
            f,
            "\t - SI: {} ({:08b})\n",
            self.registers[RegisterName::SI as usize],
            self.registers[RegisterName::SI as usize]
        )?;
        write!(
            f,
            "\t - DI: {} ({:08b})\n",
            self.registers[RegisterName::DI as usize],
            self.registers[RegisterName::DI as usize]
        )
    }
}
