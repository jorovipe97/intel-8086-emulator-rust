use std::fmt::Display;

use anyhow::{Result, anyhow};

use crate::{
    instructions::{
        decoded_instruction::DecodedInstruction,
        encodings::{CpuFlags, OperationType, RegisterName},
        operands::{Operand, SegmentRegisterName},
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
        self.compute_zf(&instruction, final_value);
        self.compute_sf(&instruction, final_value);

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
                return Err(anyhow!("destination memory operand is not supported yet"));
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
            Operand::SegmentRegister(segment_register) => {
                self.segment_registers[segment_register.to_index()] = final_value;
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
            Operand::SegmentRegister(segment_register) => {
                Ok(self.segment_registers[segment_register.to_index()])
            }
            Operand::Immediate(v) => Ok(v as u16),
            Operand::Register(reg) => {
                // Get the index of the register using register name enum's value.
                // TODO: Create to index method on the enum to get a usize.
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

    fn compute_zf(&mut self, instruction: &DecodedInstruction, final_value: u16) {
        if !instruction.affected_cpu_flags.contains(CpuFlags::ZF) {
            return;
        }

        if final_value == 0 {
            self.set_flag(CpuFlags::ZF);
        } else {
            self.clear_flag(CpuFlags::ZF);
        }
    }

    fn compute_sf(&mut self, instruction: &DecodedInstruction, final_value: u16) {
        if !instruction.affected_cpu_flags.contains(CpuFlags::SF) {
            return;
        }

        if instruction.is_w_field_set {
            if (final_value & (1 << 15)) != 0 {
                self.set_flag(CpuFlags::SF);
            } else {
                self.clear_flag(CpuFlags::SF);
            }
        } else {
            if (final_value & (1 << 7)) != 0 {
                self.set_flag(CpuFlags::SF);
            } else {
                self.set_flag(CpuFlags::SF);
            }
        }
    }

    /// Set a flag
    fn set_flag(&mut self, flag: CpuFlags) {
        // sets new value into flag position
        self.flags = self.flags | flag.bits();
    }

    fn clear_flag(&mut self, flag: CpuFlags) {
        // Resets flag position
        self.flags = self.flags & !flag.bits();
    }

    /// Check if the CpuFlag is set.
    fn is_flag_set(&self, flag: CpuFlags) -> bool {
        (self.flags & flag.bits()) > 0
    }
}

impl Display for Cpu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "General Purposes Registers:\n")?;
        write!(
            f,
            "\t - AX: {:04x} ({})\n",
            self.registers[RegisterName::A as usize],
            self.registers[RegisterName::A as usize]
        )?;
        write!(
            f,
            "\t - BX: {:04x} ({})\n",
            self.registers[RegisterName::B as usize],
            self.registers[RegisterName::B as usize],
        )?;
        write!(
            f,
            "\t - CX: {:04x} ({})\n",
            self.registers[RegisterName::C as usize],
            self.registers[RegisterName::C as usize]
        )?;
        write!(
            f,
            "\t - DX: {:04x} ({})\n",
            self.registers[RegisterName::D as usize],
            self.registers[RegisterName::D as usize],
        )?;
        write!(
            f,
            "\t - SP: {:04x} ({})\n",
            self.registers[RegisterName::SP as usize],
            self.registers[RegisterName::SP as usize],
        )?;
        write!(
            f,
            "\t - BP: {:04x} ({})\n",
            self.registers[RegisterName::BP as usize],
            self.registers[RegisterName::BP as usize]
        )?;
        write!(
            f,
            "\t - SI: {:04x} ({})\n",
            self.registers[RegisterName::SI as usize],
            self.registers[RegisterName::SI as usize]
        )?;
        write!(
            f,
            "\t - DI: {:04x} ({})\n\n",
            self.registers[RegisterName::DI as usize],
            self.registers[RegisterName::DI as usize]
        )?;
        write!(f, "Segment Registers:\n")?;
        write!(
            f,
            "\t - ES: {:04x} ({})\n",
            self.segment_registers[SegmentRegisterName::ES.to_index()],
            self.segment_registers[SegmentRegisterName::ES.to_index()]
        )?;
        write!(
            f,
            "\t - CS: {:04x} ({})\n",
            self.segment_registers[SegmentRegisterName::CS.to_index()],
            self.segment_registers[SegmentRegisterName::CS.to_index()]
        )?;
        write!(
            f,
            "\t - SS: {:04x} ({})\n",
            self.segment_registers[SegmentRegisterName::SS.to_index()],
            self.segment_registers[SegmentRegisterName::SS.to_index()]
        )?;
        write!(
            f,
            "\t - DS: {:04x} ({})\n",
            self.segment_registers[SegmentRegisterName::DS.to_index()],
            self.segment_registers[SegmentRegisterName::DS.to_index()]
        )
    }
}
