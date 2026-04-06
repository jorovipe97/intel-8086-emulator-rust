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
    pub registers: [u16; 8],
    /// Instruction pointer register
    pub instruction_pointer: usize,
    /// Extra Segment (ES), Code Segment (CS), Stack Segment (SS), Data Segment (DS)
    pub segment_registers: [u16; 4],
    pub flags: u16,
}

// TODO: To make CPU flow look realistic, maybe create a memory access method that returns the initial
// memory access that starts at instruction pointer 0, etc
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
        // Update internal instruction pointer to the value where decoded left it after
        // decoding the instruction.
        self.instruction_pointer = memory_access.instruction_pointer;
        let destination_value = self.get_operand_value(instruction.operands.destination)?;
        let source_value = self.get_operand_value(instruction.operands.source)?;

        let final_value: u16 = match instruction.operation {
            OperationType::None => 0,
            OperationType::Mov => source_value,
            OperationType::Add => destination_value.wrapping_add(source_value), // Rust overflows panics in debug mode.
            OperationType::Cmp | OperationType::Sub => destination_value.wrapping_sub(source_value),
            OperationType::Push => source_value,
            OperationType::Pop => destination_value, // TODO: This needs to change, right now this is the value of the destination, however what we actually need is the value at the top of the stack.
            OperationType::Xchg => todo!(),          // TODO: Implement the operation.
            OperationType::In => todo!(),            // TOOD: Implement IO device transfers
            OperationType::Out => todo!(),           // TOOD: Implement IO device transfers
            OperationType::Xlat => todo!(),          // TOOD: Implement IO device transfers
            OperationType::Lea => todo!(),           // TOOD: Implement IO device transfers
            OperationType::Lds => todo!(),           // TOOD: Implement IO device transfers
            OperationType::Les => todo!(),           // TOOD: Implement IO device transfers
            OperationType::Lahf => todo!(),          // TOOD: Implement IO device transfers
            OperationType::Sahf => todo!(),          // TOOD: Implement IO device transfers
            OperationType::Pushf => todo!(),         // TOOD: Implement IO device transfers
            OperationType::Popf => todo!(),          // TOOD: Implement IO device transfers
            OperationType::Adc => todo!(),           // TOOD: Implement IO device transfers
            OperationType::Inc => destination_value.wrapping_add(1),
            OperationType::Aaa => todo!(), // TOOD: Implement IO device transfers
            OperationType::Daa => todo!(),
            OperationType::Sbb => todo!(),
            OperationType::Dec => destination_value.wrapping_sub(1),
            OperationType::Neg => (!destination_value).wrapping_add(1), // Two's complement negation
            OperationType::Aas => todo!(),
            OperationType::Das => todo!(),
            OperationType::Mul => todo!(),
            OperationType::Imul => todo!(),
            OperationType::Aam => todo!(),
            OperationType::Div => todo!(),
            OperationType::Idiv => todo!(),
            OperationType::Aad => todo!(),
            OperationType::Cbw => todo!(),
            OperationType::Cwd => todo!(),
            OperationType::Not => !destination_value,
            OperationType::Shl => todo!(), // TODO: OF=0 if first operand keeps original sign.
            OperationType::Shr => todo!(), // TODO: OF=0 if first operand keeps original sign.
            OperationType::Sar => todo!(), // TODO: OF=0 if first operand keeps original sign.
            OperationType::Rol => todo!(), // TODO: OF=0 if first operand keeps original sign.
            OperationType::Ror => todo!(), // TODO: OF=0 if first operand keeps original sign.
            OperationType::Rcl => todo!(), // TODO: OF=0 if first operand keeps original sign.
            OperationType::Rcr => todo!(), // TODO: OF=0 if first operand keeps original sign.
            OperationType::And => destination_value & source_value,
            OperationType::Test => destination_value & source_value,
            OperationType::Or => destination_value | source_value,
            OperationType::Xor => destination_value ^ source_value,
            OperationType::Rep => todo!(),
            OperationType::Repe => todo!(),
            OperationType::Repne => todo!(),
            OperationType::Movsb => todo!(),
            OperationType::Movsw => todo!(),
            OperationType::Lodsb => todo!(),
            OperationType::Lodsw => todo!(),
            OperationType::Stosb => todo!(),
            OperationType::Stosw => todo!(),
            OperationType::Cmpsb => todo!(),
            OperationType::Cmpsw => todo!(),
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
        self.compute_pf(&instruction, final_value);
        self.compute_cf(&instruction, destination_value, source_value, final_value);
        self.compute_af(&instruction, destination_value, source_value, final_value);
        self.compute_of(&instruction, destination_value, source_value, final_value);

        // Check if instruction is a cmp, This instructions does not writes to destination
        // operand, just affects flags, this instruction is usually used to control the program
        // execution flow.
        if let OperationType::Cmp = instruction.operation {
            return Ok(memory_access);
        }
        if let OperationType::Test = instruction.operation {
            return Ok(memory_access);
        }

        // Updates simulated memory. Destination can be a register or memory.
        match instruction.operands.destination {
            Operand::None => (),
            Operand::Immediate(_) => {
                // TODO: How to handle instructions that support immediate in the destination
                // eg: the OUT, the number represents an I/O device port
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
            Operand::InstructionPointerIncrement(increment) => {
                self.compute_instruction_pointer(&instruction, increment);
            }
        }

        Ok(MemoryAccess {
            instruction_pointer: self.instruction_pointer,
            code_segment: self.segment_registers[SegmentRegisterName::CS as usize] as usize,
        })
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

    fn compute_instruction_pointer(&mut self, instruction: &DecodedInstruction, increment: i32) {
        let should_jump: bool = match instruction.operation {
            OperationType::Jnz => {
                if !self.is_flag_set(CpuFlags::ZF) {
                    true
                } else {
                    false
                }
            }
            OperationType::Je => {
                if self.is_flag_set(CpuFlags::ZF) {
                    true
                } else {
                    false
                }
            }
            OperationType::Jp => {
                if self.is_flag_set(CpuFlags::PF) {
                    true
                } else {
                    false
                }
            }
            OperationType::Jb => {
                if self.is_flag_set(CpuFlags::CF) {
                    true
                } else {
                    false
                }
            }
            OperationType::Jbe => {
                if self.is_flag_set(CpuFlags::CF) || self.is_flag_set(CpuFlags::ZF) {
                    true
                } else {
                    false
                }
            }
            OperationType::Jcxz => {
                if self.registers[RegisterName::C as usize] == 0 {
                    true
                } else {
                    false
                }
            }
            OperationType::Jl => {
                if self.is_flag_set(CpuFlags::SF) != self.is_flag_set(CpuFlags::OF) {
                    true
                } else {
                    false
                }
            }
            OperationType::Jle => {
                if self.is_flag_set(CpuFlags::ZF)
                    || (self.is_flag_set(CpuFlags::SF) != self.is_flag_set(CpuFlags::OF))
                {
                    true
                } else {
                    false
                }
            }
            OperationType::Jnb => {
                if self.is_flag_set(CpuFlags::CF) {
                    true
                } else {
                    false
                }
            }
            OperationType::Jnbe => {
                if self.is_flag_set(CpuFlags::CF) || self.is_flag_set(CpuFlags::ZF) {
                    true
                } else {
                    false
                }
            }
            OperationType::Jne => {
                if self.is_flag_set(CpuFlags::ZF) {
                    true
                } else {
                    false
                }
            }
            OperationType::Jnl => {
                if self.is_flag_set(CpuFlags::SF) == self.is_flag_set(CpuFlags::OF) {
                    true
                } else {
                    false
                }
            }
            OperationType::Jnle => {
                if !self.is_flag_set(CpuFlags::ZF)
                    && self.is_flag_set(CpuFlags::SF) == self.is_flag_set(CpuFlags::OF)
                {
                    true
                } else {
                    false
                }
            }
            OperationType::Jno => {
                if !self.is_flag_set(CpuFlags::OF) {
                    true
                } else {
                    false
                }
            }
            OperationType::Jnp => {
                if !self.is_flag_set(CpuFlags::PF) {
                    true
                } else {
                    false
                }
            }
            OperationType::Jns => {
                if !self.is_flag_set(CpuFlags::SF) {
                    true
                } else {
                    false
                }
            }
            OperationType::Jo => {
                if self.is_flag_set(CpuFlags::OF) {
                    true
                } else {
                    false
                }
            }
            OperationType::Js => {
                if self.is_flag_set(CpuFlags::SF) {
                    true
                } else {
                    false
                }
            }
            OperationType::LoopNz => {
                // Decremtn CX register
                self.registers[RegisterName::C as usize] =
                    self.registers[RegisterName::C as usize].wrapping_sub_signed(1);
                if self.registers[RegisterName::C as usize] != 0 && !self.is_flag_set(CpuFlags::ZF)
                {
                    true
                } else {
                    false
                }
            }
            OperationType::LoopZ => {
                // Decremtn CX register
                self.registers[RegisterName::C as usize] =
                    self.registers[RegisterName::C as usize].wrapping_sub_signed(1);
                if self.registers[RegisterName::C as usize] != 0 && self.is_flag_set(CpuFlags::ZF) {
                    true
                } else {
                    false
                }
            }
            OperationType::Loop => {
                // Decremtn CX register
                self.registers[RegisterName::C as usize] =
                    self.registers[RegisterName::C as usize].wrapping_sub_signed(1);
                if self.registers[RegisterName::C as usize] != 0 {
                    true
                } else {
                    false
                }
            }
            // No jump instructions.
            _ => false,
            // OperationType::Cmp
            // | OperationType::Sub
            // | OperationType::Add
            // | OperationType::None
            // | OperationType::Mov
            // | OperationType::Push
            // | OperationType::Pop
            // | OperationType::Xchg
            // | OperationType::In
            // | OperationType::Out
            // | OperationType::Xlat
            // | OperationType::Lea
            // | OperationType::Lds
            // | OperationType::Les
            // | OperationType::Lahf
            // | OperationType::Sahf
            // | OperationType::Pushf
            // | OperationType::Popf
            // | OperationType::Adc
            // | OperationType::Inc
            // | OperationType::Aaa => false,
        };

        if should_jump {
            self.instruction_pointer = self
                .instruction_pointer
                .wrapping_add_signed(increment as isize);
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
                self.clear_flag(CpuFlags::SF);
            }
        }
    }

    fn compute_pf(&mut self, instruction: &DecodedInstruction, final_value: u16) {
        if !instruction.affected_cpu_flags.contains(CpuFlags::PF) {
            return;
        }

        if ((final_value as u8).count_ones() & 1) == 0 {
            // If final values is even number of 1 in the lowest byte
            self.set_flag(CpuFlags::PF);
        } else {
            self.clear_flag(CpuFlags::PF);
        }
    }

    fn compute_cf(
        &mut self,
        instruction: &DecodedInstruction,
        destination_value: u16,
        source_value: u16,
        final_value: u16,
    ) {
        // https://www.youtube.com/watch?v=F20rPdjGI8k
        if !instruction.affected_cpu_flags.contains(CpuFlags::CF) {
            return;
        }

        // Overflow calculation, depends on the operation
        if instruction.operation == OperationType::Add {
            // If the final value is lower than one of the operands
            // then is because there happened an overflow.
            if final_value < destination_value {
                self.set_flag(CpuFlags::CF);
            } else {
                self.clear_flag(CpuFlags::CF);
            }
        } else if instruction.operation == OperationType::Cmp
            || instruction.operation == OperationType::Sub
        {
            // if first operand is lower than second one, then it will result in a negative
            // which is an overflow
            if destination_value < source_value {
                self.set_flag(CpuFlags::CF);
            } else {
                self.clear_flag(CpuFlags::CF);
            }
        }
    }

    fn compute_af(
        &mut self,
        instruction: &DecodedInstruction,
        destination_value: u16,
        source_value: u16,
        final_value: u16,
    ) {
        // https://www.youtube.com/watch?v=F20rPdjGI8k
        if !instruction.affected_cpu_flags.contains(CpuFlags::AF) {
            return;
        }

        // Overflow calculation, depends on the operation
        if instruction.operation == OperationType::Add {
            // If the final value is lower than one of the operands
            // then is because there happened an overflow.
            if (final_value & 0x0f) < (destination_value & 0x0f) {
                self.set_flag(CpuFlags::AF);
            } else {
                self.clear_flag(CpuFlags::AF);
            }
        } else if instruction.operation == OperationType::Cmp
            || instruction.operation == OperationType::Sub
        {
            // if first operand is lower than second one, then it will result in a negative
            // which is an overflow
            if (destination_value & 0x0f) < (source_value & 0x0f) {
                self.set_flag(CpuFlags::AF);
            } else {
                self.clear_flag(CpuFlags::AF);
            }
        }
    }

    fn compute_of(
        &mut self,
        instruction: &DecodedInstruction,
        destination_value: u16,
        source_value: u16,
        final_value: u16,
    ) {
        if !instruction.affected_cpu_flags.contains(CpuFlags::OF) {
            return;
        }

        if instruction.operation == OperationType::Add {
            if instruction.is_w_field_set {
                // TODO: Can check sign without casting, just using most significative bit flag.
                // If operands are positive but result is negative, an overflow happened.
                let positive_overflow = (destination_value as i16) >= 0
                    && (source_value as i16) >= 0
                    && (final_value as i16) < 0;

                // If operands are negative but result is positive or 0, an overflow happened
                let negative_overflow = (destination_value as i16) <= 0
                    && (source_value as i16) <= 0
                    && (final_value as i16) >= 0;

                if positive_overflow || negative_overflow {
                    self.set_flag(CpuFlags::OF);
                } else {
                    self.clear_flag(CpuFlags::OF);
                }
            } else {
                // If operands are positive but result is negative, an overflow happened.
                let positive_overflow = (destination_value as i8) >= 0
                    && (source_value as i8) >= 0
                    && (final_value as i8) < 0;

                // If operands are negative but result is positive or 0, an overflow happened
                let negative_overflow = (destination_value as i8) <= 0
                    && (source_value as i8) <= 0
                    && (final_value as i8) >= 0;

                if positive_overflow || negative_overflow {
                    self.set_flag(CpuFlags::OF);
                } else {
                    self.clear_flag(CpuFlags::OF);
                }
            }
        } else if instruction.operation == OperationType::Cmp
            || instruction.operation == OperationType::Sub
        {
            // For sub, an overflow happens if operands have different sign
            // and the result have a sign different to the first operand
            // then an overflow happened
            let most_significate_bit: u16 = if instruction.is_w_field_set {
                1 << 15
            } else {
                1 << 7
            };

            // For example if:
            // 1000 0010 ^
            // 0000 1010
            // ---------
            // 1000 1000 (Note the most significate bit is 1, then a negative result indicates signs are different)
            // We cast to int16, so the result give us a negative number when msb is 1.
            //
            // Note this, is similar to the Add case, however since operator is a sub the conditions in which
            // can happen a possitive or negative overflow change a bit.
            //
            // If most significative bit is 1, then operand signs are different.
            let are_operands_signs_different =
                ((destination_value ^ source_value) & most_significate_bit) == most_significate_bit;

            // Did result sign changed from sign of first operand?
            let did_result_changed_sign =
                ((destination_value ^ final_value) & most_significate_bit) == most_significate_bit;

            if are_operands_signs_different && did_result_changed_sign {
                self.set_flag(CpuFlags::OF);
            } else {
                self.clear_flag(CpuFlags::OF);
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
        write!(f, "===================================\n")?;
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
            "\t - DS: {:04x} ({})\n\n",
            self.segment_registers[SegmentRegisterName::DS.to_index()],
            self.segment_registers[SegmentRegisterName::DS.to_index()]
        )?;

        write!(
            f,
            "Instruction Pointer: {:04x} ({})\n\n",
            self.instruction_pointer, self.instruction_pointer
        )?;

        write!(f, "Flags:\n")?;
        if self.is_flag_set(CpuFlags::CF) {
            write!(f, "\t - CF\n")?;
        }
        if self.is_flag_set(CpuFlags::PF) {
            write!(f, "\t - PF\n")?;
        }
        if self.is_flag_set(CpuFlags::AF) {
            write!(f, "\t - AF\n")?;
        }
        if self.is_flag_set(CpuFlags::ZF) {
            write!(f, "\t - ZF\n")?;
        }
        if self.is_flag_set(CpuFlags::SF) {
            write!(f, "\t - SF\n")?;
        }
        if self.is_flag_set(CpuFlags::OF) {
            write!(f, "\t - OF\n")?;
        }
        write!(f, "===================================\n")
    }
}
