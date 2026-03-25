use crate::instructions::INSTRUCTION_ENCODINGS_TABLE;
use crate::instructions::MAX_INSTRUCTION_BYTE_COUNT;
use crate::instructions::decoded_instruction::DecodedInstruction;
use crate::instructions::encodings::{
    InstructionBitsUsage, InstructionEncoding, OperationType, RegisterName,
};
use crate::instructions::operands::{MemoryDisplacementInfo, Operand, RegisterInfo};
use crate::memory::{Memory, MemoryAccess};
use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;

pub struct Decoder<'a> {
    memory: &'a Memory,
}

impl<'a> Decoder<'a> {
    pub fn new(memory: &'a Memory) -> Decoder<'a> {
        Decoder { memory }
    }

    pub fn has_more_instructions(&self, memory_access: MemoryAccess) -> bool {
        return (memory_access.absolute_address() + 1) < self.memory.program_size();
    }

    // TODO: Make this method indicate action, eg_ decode_instruction.
    // Returns the decoded instruction and a new memory_access instance with the beginning of the enxt instruction.
    pub fn current_instruction(
        &self,
        memory_access: MemoryAccess,
    ) -> Result<(DecodedInstruction, MemoryAccess)> {
        let mut result = DecodedInstruction::DEFAULT;
        let mut next_memory_address = memory_access;

        // Compares current byte location with all the instruction encodings in INSTRUCTIONS_TABLE constant.
        for candidate_encoding in INSTRUCTION_ENCODINGS_TABLE {
            (result, next_memory_address) = self.try_decode(candidate_encoding, memory_access)?;

            if result.operation != OperationType::None {
                // We were able to decode the instruction, no need to keep looking for candidates.
                break;
            }
        }

        // If could not decode using any of the candidate encodings in INSTRUCTION_ENCODINGS_TABLE
        // return an error.
        if result.operation == OperationType::None {
            return Err(anyhow!(
                "instruction is unknown, at position {}",
                next_memory_address.absolute_address()
            ));
        }

        Ok((result, next_memory_address))
    }

    fn try_decode(
        &self,
        candidate_encoding: &InstructionEncoding,
        memory_access: MemoryAccess,
    ) -> Result<(DecodedInstruction, MemoryAccess)> {
        // This is an optimized map, we use this to track the values of
        // each bit field in the instructino
        let mut bits_parts = [0; InstructionBitsUsage::Count as usize];

        // Tracks if the instruction has an specific bit field.
        let mut has = [false; InstructionBitsUsage::Count as usize];

        // Tracks if the instruction readed from memory can be decoded using candidate_encoding.
        let mut valid = true;

        // The resulting decoded instruction.
        let mut result = DecodedInstruction::DEFAULT;

        // The actual bits values pending for processing in current byte.
        let mut bits_pending: u8 = 0;

        // The number of bits pending for processing in current byte.
        let mut bits_pending_count = 0;

        let mut memory_access_internal = memory_access;

        for test_bits in candidate_encoding.bits {
            // If candidate encoding result not valid, stop
            // checking more parts of the encoding.
            if !valid {
                break;
            }

            let mut read_bits = test_bits.value;
            if test_bits.bit_count != 0 {
                if bits_pending_count == 0 {
                    // We finished reading a byte, reset pending count
                    // and read a new byte
                    bits_pending_count = 8;
                    // We copy the value from memory array to bits_pending local variable.
                    bits_pending = *self
                        .memory
                        .read(memory_access_internal)
                        .with_context(|| "error when trying to decode")?;
                    memory_access_internal.instruction_pointer += 1;
                }

                // NOTE(casey): If this assert fires, it means we have an error in our table,
                // since there are no 8086 instructions that have bit values straddling a
                // byte boundary.
                if test_bits.bit_count > MAX_INSTRUCTION_BYTE_COUNT {
                    return Err(anyhow!("instruction table has an error"));
                }

                bits_pending_count -= test_bits.bit_count;
                read_bits = bits_pending;

                // Let's say bits_pending = 0b10001101 and we want to extract 6 bits, then 2 bits are pending:
                // First extraction (6 bits):
                //
                // bits_pending       = 10001101
                // bits_pending_count = 8 - 6 = 2
                //
                // read_bits = 10001101 >> 2 = 00100011
                // mask = ~(0xff << 6) = ~(11000000) = 00111111
                //
                // read_bits & mask = 00100011 & 00111111 = 00100011 ✓ (extracted top 6 bits)
                read_bits = read_bits >> bits_pending_count;
                let mask = match (0xff as u8).checked_shl(test_bits.bit_count as u32) {
                    Some(m) => m,
                    None => 0, // Overflow, just return 0. The bitwise not will turn all bits to 1.
                };
                read_bits = read_bits & !mask
            }

            // Either check if literal bits is equal, or save the fields.
            if test_bits.usage == InstructionBitsUsage::Literal {
                // All instructions start here, if read_bits is not the opcode of the candidate_encoding
                // then is invalid, this byte does not represents the candidate instruction encoding.

                // Note literals can appear multiple times in different instructions, not only at the begining
                // for that we do the valid && ...
                valid = valid && (read_bits == test_bits.value)
            } else {
                bits_parts[test_bits.usage as usize] = read_bits as i32;
                has[test_bits.usage as usize] = true;
            }
        }

        // If literals are not valid, then return DecodedInstruction::DEFAULT which is a none.
        if !valid {
            return Ok((result, memory_access));
        }

        let d = bits_parts[InstructionBitsUsage::D as usize];
        let w = bits_parts[InstructionBitsUsage::W as usize];
        let mod_field = bits_parts[InstructionBitsUsage::Mod as usize];
        let rm = bits_parts[InstructionBitsUsage::Rm as usize];
        let s = bits_parts[InstructionBitsUsage::S as usize];

        let has_direct_address = mod_field == 0b00 && rm == 0b110;
        let disp_is_w = mod_field == 0b10 || has_direct_address;
        let data_is_w = bits_parts[InstructionBitsUsage::WMakesDataWide as usize] == 0b1
            && s == 0b0
            && w == 0b1;
        let has_data = has[InstructionBitsUsage::Data as usize];

        // Warning, the order calling of parseDispValue and parseDataValue
        // is important, because those function update the internalPosition.
        bits_parts[InstructionBitsUsage::Disp as usize] =
            self.parse_disp_value(&mut memory_access_internal, mod_field, disp_is_w)?;
        bits_parts[InstructionBitsUsage::Data as usize] =
            self.parse_data_value(&mut memory_access_internal, has_data, data_is_w)?;
        // TODO: internal pointer is increasde by 4 after parse_disp_value and parse_data_value

        result.operation = candidate_encoding.op;
        result.affected_cpu_flags = candidate_encoding.affected_cpu_flags;

        let has_w = has[InstructionBitsUsage::W as usize];
        if has_w && w == 1 {
            result.is_w_field_set = true
        }

        let mut reg_operand = Operand::None;
        let mut mod_operand = Operand::None;

        let has_reg = has[InstructionBitsUsage::Reg as usize];
        let has_mod = has[InstructionBitsUsage::Mod as usize];
        let has_d = has[InstructionBitsUsage::D as usize];
        let has_ip_inc = has[InstructionBitsUsage::IpInc as usize];

        if has_reg {
            reg_operand = self.get_reg_operand(bits_parts[InstructionBitsUsage::Reg as usize], w)?
        }

        if has_mod {
            if mod_field == 0b11 {
                // If MOD==0b11 (register-to-register mode), then
                // R/M identifies the second register operand.
                mod_operand = self.get_reg_operand(rm, w)?
            } else {
                let displacement = bits_parts[InstructionBitsUsage::Disp as usize];
                mod_operand = self.get_memory_operand(mod_field, rm, displacement)?;
            }
        }

        if has_ip_inc {
            let ip_increment = self
                .parse_data_value(&mut memory_access_internal, has_ip_inc, data_is_w)
                .with_context(|| "could not extract instruction pointer increment")?;
            mod_operand = Operand::InstructionPointerIncrement(ip_increment)
        }

        // How many bytes did move the internal cursor to decode the full instruction.
        result.size = memory_access_internal.absolute_address() - memory_access.absolute_address();

        if has_d {
            if d == 0 {
                // Instruction source is specified in REG field.
                result.operands.destination = mod_operand;
                result.operands.source = reg_operand;
            } else {
                // Instruction destination is specified in REG field.
                result.operands.destination = reg_operand;
                result.operands.source = mod_operand;
            }
        }

        // NOTE(casey): Because there are some strange opcodes that do things like have an immediate as
        // a _destination_ ("out", for example), I define immediates and other "additional operands" to
        // go in "whatever slot was not used by the reg and mod fields".
        if has_data && !has_ip_inc {
            let data = bits_parts[InstructionBitsUsage::Data as usize];
            if let Operand::None = result.operands.source {
                result.operands.source = Operand::Immediate(data)
            } else if let Operand::None = result.operands.destination {
                result.operands.destination = Operand::Immediate(data)
            }
        }

        Ok((result, memory_access_internal))
    }

    fn parse_disp_value(
        &self,
        memory_access: &mut MemoryAccess,
        mod_field: i32,
        disp_is_w: bool,
    ) -> Result<i32> {
        let mut displacement = 0;
        if disp_is_w {
            // Memory mode, 16-bit displacement follows
            // Or mod == was 0b00 and rm == 0b110.
            let memory_access_0 = *memory_access;
            let disp_0 = *self
                .memory
                .read(memory_access_0)
                .with_context(|| "failed reading first byte for displacement value")?
                as i16;

            let mut memory_access_1 = *memory_access;
            memory_access_1.instruction_pointer += 1;
            let disp_1 = *self
                .memory
                .read(memory_access_1)
                .with_context(|| "failed reading second byte for displacement value")?
                as i16;

            // Perform cast to sign 16 so we get symbol correctly.
            // Then cast to int so go compiler performs a sign extension.
            displacement = ((disp_1 << 8) | disp_0) as i32;
            // Updated memory access passed as mutable reference.
            memory_access.instruction_pointer += 2
        } else if mod_field == 0b01 {
            // Memory mode, 8-bit displacement follows
            // Perform cast to sign 8 so we get symbol correctly.
            // Then cast to int so go compiler performs a sign extension.
            displacement = *self
                .memory
                .read(*memory_access)
                .with_context(|| "failed reading displacement value")?
                as i8 as i32; // TODO: Do we need both casts? Maybe yes to ensure correct sign, then signn extension.
            // Updated memory access passed as mutable reference.
            memory_access.instruction_pointer += 1
        }

        return Ok(displacement);
    }

    fn parse_data_value(
        &self,
        memory_access: &mut MemoryAccess,
        has_data: bool,
        data_is_w: bool,
    ) -> Result<i32> {
        let mut data = 0;

        if !has_data {
            // Simply return 0, this won't be used so is ok to return 0
            // otherwise you may think we return data that dont exists.
            return Ok(data);
        }

        if data_is_w {
            let memory_access_0 = *memory_access;
            let data_0 = *self
                .memory
                .read(memory_access_0)
                .with_context(|| "failed reading first byte for displacement value")?
                as i16;

            let mut memory_access_1 = *memory_access;
            memory_access_1.instruction_pointer += 1;
            let data_1 = *self
                .memory
                .read(memory_access_1)
                .with_context(|| "failed reading second byte for displacement value")?
                as i16;

            // Perform cast to sign 16 so we get symbol correctly.
            // Then cast to int so go compiler performs a sign extension.
            data = ((data_1 << 8) | data_0) as i32;
            memory_access.instruction_pointer += 2
        } else {
            // Perform cast to sign 8 so we get symbol correctly.
            // Then cast to int so go compiler performs a sign extension.
            data = *self
                .memory
                .read(*memory_access)
                .with_context(|| "failed reading displacement value")? as i8
                as i32;
            memory_access.instruction_pointer += 1;
        }

        Ok(data)
    }

    const REG_TABLE: [[RegisterInfo; 2]; 8] = [
        // In each row, first item is for w=0 and second item is for w=1
        [
            RegisterInfo {
                register_name: RegisterName::A,
                offset: 0,
                count: 1,
            }, // AL
            RegisterInfo {
                register_name: RegisterName::A,
                offset: 0,
                count: 2,
            }, // AX
        ],
        [
            RegisterInfo {
                register_name: RegisterName::C,
                offset: 0,
                count: 1,
            }, // CL
            RegisterInfo {
                register_name: RegisterName::C,
                offset: 0,
                count: 2,
            }, // CX
        ],
        [
            RegisterInfo {
                register_name: RegisterName::D,
                offset: 0,
                count: 1,
            }, // DL
            RegisterInfo {
                register_name: RegisterName::D,
                offset: 0,
                count: 2,
            }, // DX
        ],
        [
            RegisterInfo {
                register_name: RegisterName::B,
                offset: 0,
                count: 1,
            }, // BL
            RegisterInfo {
                register_name: RegisterName::B,
                offset: 0,
                count: 2,
            }, // BX
        ],
        [
            RegisterInfo {
                register_name: RegisterName::A,
                offset: 1,
                count: 1,
            }, // AH
            RegisterInfo {
                register_name: RegisterName::SP,
                offset: 0,
                count: 2,
            }, // SP
        ],
        [
            RegisterInfo {
                register_name: RegisterName::C,
                offset: 1,
                count: 1,
            }, // CH
            RegisterInfo {
                register_name: RegisterName::BP,
                offset: 0,
                count: 2,
            }, // BP
        ],
        [
            RegisterInfo {
                register_name: RegisterName::D,
                offset: 1,
                count: 1,
            }, // DH
            RegisterInfo {
                register_name: RegisterName::SI,
                offset: 0,
                count: 2,
            }, // SI
        ],
        [
            RegisterInfo {
                register_name: RegisterName::B,
                offset: 1,
                count: 1,
            }, // BH
            RegisterInfo {
                register_name: RegisterName::DI,
                offset: 0,
                count: 2,
            }, // DI
        ],
    ];

    fn get_reg_operand(&self, reg_flag: i32, w: i32) -> Result<Operand> {
        Decoder::REG_TABLE
            .get(reg_flag as usize)
            .ok_or_else(|| anyhow!("no register found with specified flag"))?
            .get(w as usize)
            .ok_or_else(|| anyhow!("w field must be either 1 or 0"))
            .map(|register_info| Operand::Register(*register_info))
    }

    const MEMORY_EFFECTIVE_ADDRESS_TERMS_0_TABLE: [RegisterName; 8] = [
        RegisterName::B,
        RegisterName::B,
        RegisterName::BP,
        RegisterName::BP,
        RegisterName::SI,
        RegisterName::DI,
        RegisterName::BP,
        RegisterName::B,
    ];

    const MEMORY_EFFECTIVE_ADDRESS_TERMS_1_TABLE: [RegisterName; 8] = [
        RegisterName::SI,
        RegisterName::DI,
        RegisterName::SI,
        RegisterName::DI,
        RegisterName::None,
        RegisterName::None,
        RegisterName::None,
        RegisterName::None,
    ];

    fn get_memory_operand(&self, mod_field: i32, rm: i32, displacement: i32) -> Result<Operand> {
        if mod_field == 0b00 && rm == 0b110 {
            // On this case effective address calculation simpy uses
            // a direct address.
            return Ok(Operand::Memory(MemoryDisplacementInfo {
                terms: [RegisterInfo::NONE, RegisterInfo::NONE],
                displacement,
            }));
        }

        let term0 = Self::MEMORY_EFFECTIVE_ADDRESS_TERMS_0_TABLE
            .get(rm as usize)
            .ok_or_else(|| anyhow!("rm field is invalid"))
            .map(|register_name| RegisterInfo {
                register_name: *register_name,
                offset: 0,
                count: 2,
            })?;

        let term1 = Self::MEMORY_EFFECTIVE_ADDRESS_TERMS_1_TABLE
            .get(rm as usize)
            .ok_or_else(|| anyhow!("rm field invalid"))
            .map(|register_name| RegisterInfo {
                register_name: *register_name,
                count: 2,
                offset: 0,
            })?;

        Ok(Operand::Memory(MemoryDisplacementInfo {
            terms: [term0, term1],
            displacement,
        }))
    }
}
