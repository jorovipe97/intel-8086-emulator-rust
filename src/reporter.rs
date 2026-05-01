use crate::{
    cpu::ExecutionResult,
    disassembler::Disassembler,
    instructions::{
        decoded_instruction::{DecodedInstruction, DecodedInstructionExtraAttributes},
        encodings::{CpuFlags, OperationType, RegisterName},
        operands::{MemoryDisplacementInfo, Operand},
    },
};
use anyhow::Result;

pub enum CpuVersion {
    Intel8086,
    Intel8088,
}

pub struct Reporter {
    disassembler: Disassembler,
    cpu_version: CpuVersion,
    total_clocks: i32,
}

struct InstructionClocks {
    base: i32,
    ea: i32,
    transfers: i32, // We dont compute if memory is in odd address.
}

impl Reporter {
    pub fn new(cpu_version: CpuVersion) -> Reporter {
        let mut disassembler = Disassembler::new();
        disassembler.add_bits_16_header();
        Reporter {
            disassembler,
            cpu_version,
            total_clocks: 0,
        }
    }

    pub fn analyze_instruction_execution(
        &mut self,
        instruction: DecodedInstruction,
        prev_execution_result: ExecutionResult,
        new_execution_result: ExecutionResult,
    ) {
        self.disassembler.add_instruction(&instruction);
        self.disassembler.begin_comment_after_instruction();
        self.report_clocks(&instruction, &new_execution_result);
        self.report_flags_values(prev_execution_result.flags, new_execution_result.flags);
        self.disassembler.add_new_line();
    }

    fn report_clocks(
        &mut self,
        instruction: &DecodedInstruction,
        execution_result: &ExecutionResult,
    ) {
        let clocks_option = match instruction.operation {
            OperationType::Mov => self.calculate_clocks_mov(
                instruction.operands.destination,
                instruction.operands.source,
                instruction
                    .extra_attributes
                    .contains(DecodedInstructionExtraAttributes::IS_WIDE),
            ),
            OperationType::Add | OperationType::Xor | OperationType::Cmp => self
                .calculate_clocks_add(
                    instruction.operands.destination,
                    instruction.operands.source,
                ),
            OperationType::Inc | OperationType::Dec => self.calculate_clocks_inc(
                instruction.operands.destination,
                instruction
                    .extra_attributes
                    .contains(DecodedInstructionExtraAttributes::IS_WIDE),
            ),
            OperationType::Test => self.calculate_clocks_test(
                instruction.operands.destination,
                instruction.operands.source,
                instruction
                    .extra_attributes
                    .contains(DecodedInstructionExtraAttributes::IS_WIDE),
            ),
            OperationType::Je | OperationType::Jnz | OperationType::Jne | OperationType::Jb => {
                let base = if execution_result.condition_branch_taken {
                    16
                } else {
                    4
                };

                Some(InstructionClocks {
                    base,
                    transfers: 0,
                    ea: 0,
                })
            }
            OperationType::Shr => self.calculate_clocks_shr(
                instruction.operands.destination,
                instruction.operands.source,
            ),
            _ => None,
        };

        if let Some(clocks) = clocks_option {
            let total_clocks = clocks.base + clocks.ea + clocks.transfers;
            self.total_clocks += total_clocks;
            let mut explanation = String::with_capacity(32);
            explanation.push_str(clocks.base.to_string().as_str());
            if clocks.ea > 0 {
                explanation.push('+');
                explanation.push_str(clocks.ea.to_string().as_str());
                explanation.push_str("ea");
            }
            if clocks.transfers > 0 {
                explanation.push('+');
                explanation.push_str(clocks.transfers.to_string().as_str());
                explanation.push_str("transfers");
            }

            let comment = format!("Clocks: {} = ({explanation})", self.total_clocks);

            self.disassembler
                .add_comment_after_instruction(comment.as_str());
        }
    }

    fn calculate_clocks_shr(
        &self,
        destination: Operand,
        source: Operand,
    ) -> Option<InstructionClocks> {
        let mut clocks = InstructionClocks {
            base: 0,
            ea: 0,
            transfers: 0,
        };

        if let Operand::Register(_) = destination
            && let Operand::Immediate(_) = source
        {
            clocks.base = 2;
        } else {
            todo!("other combinations not implemented");
        }

        return Some(clocks);
    }

    fn calculate_clocks_inc(
        &self,
        destination: Operand,
        is_wide: bool,
    ) -> Option<InstructionClocks> {
        let mut clocks = InstructionClocks {
            base: 0,
            ea: 0,
            transfers: 0,
        };

        match destination {
            Operand::Register(reg_info) => {
                if reg_info.count == 1 {
                    // If register is 8 bits
                    clocks.base = 3;
                } else {
                    // If register is 16 bits
                    clocks.base = 2;
                }
            }
            Operand::Memory(mem_info) => {
                clocks.base = 15;
                clocks.ea = self.calculate_clocks_effective_address(mem_info);
                if is_wide {
                    match self.cpu_version {
                        CpuVersion::Intel8086 => (),
                        CpuVersion::Intel8088 => {
                            // Two transfers, we must add 4 clocks per transfer.
                            clocks.transfers = 2 * 4;
                        }
                    }
                }
            }
            _ => return None,
        }

        return Some(clocks);
    }

    fn calculate_clocks_mov(
        &self,
        destination: Operand,
        source: Operand,
        is_wide: bool,
    ) -> Option<InstructionClocks> {
        let mut clocks = InstructionClocks {
            base: 0,
            ea: 0,
            transfers: 0,
        };

        // TODO: Refactor this, as this pattern wont scale well, if we wanted to report
        // clocks for all instrucitons.
        if let Operand::Memory(dst_mem_info) = destination
            && let Operand::Register(src_reg_info) = source
        {
            // Memory to accumulator (AL or AX)
            if src_reg_info.register_name == RegisterName::A && src_reg_info.offset != 1 {
                clocks.base = 10;
                // According to manual when source or destination is the accumulator
                // there is no cost for EA calculation. How is this possible?
                // did not incur extra Effective Address (EA) calculation clocks because they
                // used specialized opcodes (A0h-A3h) that encoded the address directly,
                // skipping the standard ModRM byte used by other instructions.
                // This design optimized common tasks for efficiency.
                // Remember that on this variant of mov, the memory only have a dirrect address: eg [200]
                // No calculations are allowed
                // https://stackoverflow.com/questions/79306380/8086-memory-to-accumulator-encoding-why-do-mov-al-absolute-and-mov-ah-abso
            } else {
                clocks.base = 9;

                // TODO: This code is absolute trash, refactor ASAP.
                clocks.ea = self.calculate_clocks_effective_address(dst_mem_info);

                let is_16_bits = src_reg_info.count == 2;
                // TODO: We need access to cpu registers to compute this, make them available in the ExecutionReport struct
                // let ea_address = self.calculate_memory_effective_address(dst_mem_info)?;

                match self.cpu_version {
                    CpuVersion::Intel8086 => {
                        // If effective address is odd, add 4 extra clocks for each transfer
                        // if is_16_bits && (ea_address.offset & 0b1) == 0b1 {
                        //     transfers_clocks = 4;
                        // }
                    }
                    CpuVersion::Intel8088 => {
                        // Add 16 bits for each word transfer as the 8088 buss is 8 bits
                        if is_16_bits {
                            clocks.transfers = 4;
                        }
                    }
                }
            }
        } else if let Operand::Register(dst_reg_info) = destination
            && let Operand::Memory(src_mem_info) = source
        {
            // Accumulator (AL or AX) to Memory
            if dst_reg_info.register_name == RegisterName::A && dst_reg_info.offset != 1 {
                clocks.base = 10;
            } else {
                // TODO: Get this directly from decoded instruction, here we just need to check if instruction is doing EA calculations and transfers penalties.
                clocks.base = 8;

                // TODO: This code is absolute trash, refactor ASAP.
                clocks.ea = self.calculate_clocks_effective_address(src_mem_info);

                let is_16_bits = dst_reg_info.count == 2;
                // let ea_address = self.calculate_memory_effective_address(src_mem_info)?;

                match self.cpu_version {
                    CpuVersion::Intel8086 => {
                        // If effective address is odd, add 4 extra clocks for each transfer
                        // if is_16_bits && (ea_address.offset & 0b1) == 0b1 {
                        //     transfers_clocks = 4;
                        // }
                    }
                    CpuVersion::Intel8088 => {
                        // Add 16 bits for each word transfer as the 8088 buss is 8 bits
                        if is_16_bits {
                            clocks.transfers = 4;
                        }
                    }
                }
            }
        } else if let Operand::Register(_) = destination
            && let Operand::Immediate(_) = source
        {
            clocks.base = 4;
        } else if let Operand::Register(_) = destination
            && let Operand::Register(_) = source
        {
            clocks.base = 2;
        } else if let Operand::Memory(mem_dst_info) = destination
            && let Operand::Immediate(_) = source
        {
            clocks.base = 10;
            clocks.ea = self.calculate_clocks_effective_address(mem_dst_info);
            match self.cpu_version {
                CpuVersion::Intel8086 => (),
                CpuVersion::Intel8088 => {
                    // Add 16 bits for each word transfer as the 8088 buss is 8 bits
                    if is_wide {
                        clocks.transfers = 4;
                    }
                }
            }
        }

        Some(clocks)
    }

    fn calculate_clocks_add(
        &self,
        destination: Operand,
        source: Operand,
    ) -> Option<InstructionClocks> {
        let mut clocks = InstructionClocks {
            base: 0,
            ea: 0,
            transfers: 0,
        };

        // TODO: Refactor this, as this pattern wont scale well, if we wanted to report
        // clocks for all instrucitons.
        if let Operand::Memory(dst_mem_info) = destination
            && let Operand::Register(src_reg_info) = source
        {
            clocks.base = 16;

            // TODO: This code is absolute trash, refactor ASAP.
            clocks.ea = self.calculate_clocks_effective_address(dst_mem_info);

            let is_16_bits = src_reg_info.count == 2;
            // TODO: We need access to cpu registers to compute this, make them available in the ExecutionReport struct
            // let ea_address = self.calculate_memory_effective_address(dst_mem_info)?;

            match self.cpu_version {
                CpuVersion::Intel8086 => {
                    // If effective address is odd, add 4 extra clocks for each transfer
                    // if is_16_bits && (ea_address.offset & 0b1) == 0b1 {
                    //     transfers_clocks = 4;
                    // }
                }
                CpuVersion::Intel8088 => {
                    // Add 16 bits for each word transfer as the 8088 buss is 8 bits
                    if is_16_bits {
                        clocks.transfers = 4;
                    }
                }
            }
        } else if let Operand::Register(dst_reg_info) = destination
            && let Operand::Memory(src_mem_info) = source
        {
            // TODO: Get this directly from decoded instruction, here we just need to check if instruction is doing EA calculations and transfers penalties.
            clocks.base = 9;

            // TODO: This code is absolute trash, refactor ASAP.
            clocks.ea = self.calculate_clocks_effective_address(src_mem_info);

            let is_16_bits = dst_reg_info.count == 2;
            // let ea_address = self.calculate_memory_effective_address(src_mem_info)?;

            match self.cpu_version {
                CpuVersion::Intel8086 => {
                    // If effective address is odd, add 4 extra clocks for each transfer
                    // if is_16_bits && (ea_address.offset & 0b1) == 0b1 {
                    //     transfers_clocks = 4;
                    // }
                }
                CpuVersion::Intel8088 => {
                    // Add 16 bits for each word transfer as the 8088 buss is 8 bits
                    if is_16_bits {
                        clocks.transfers = 4;
                    }
                }
            }
        } else if let Operand::Register(_) = destination
            && let Operand::Immediate(_) = source
        {
            clocks.base = 4;
        } else if let Operand::Register(_) = destination
            && let Operand::Register(_) = source
        {
            clocks.base = 3;
        }

        Some(clocks)
    }

    fn calculate_clocks_test(
        &self,
        destination: Operand,
        source: Operand,
        is_wide: bool,
    ) -> Option<InstructionClocks> {
        let mut clocks = InstructionClocks {
            base: 0,
            ea: 0,
            transfers: 0,
        };

        // TODO: Refactor this, as this pattern wont scale well, if we wanted to report
        // clocks for all instrucitons.
        if let Operand::Register(_) = destination
            && let Operand::Register(_) = source
        {
            clocks.base = 3;
        } else if let Operand::Register(dst_reg_info) = destination
            && let Operand::Memory(src_mem_info) = source
        {
            // TODO: Get this directly from decoded instruction, here we just need to check if instruction is doing EA calculations and transfers penalties.
            clocks.base = 9;

            // TODO: This code is absolute trash, refactor ASAP.
            clocks.ea = self.calculate_clocks_effective_address(src_mem_info);

            let is_16_bits = dst_reg_info.count == 2;
            // let ea_address = self.calculate_memory_effective_address(src_mem_info)?;

            match self.cpu_version {
                CpuVersion::Intel8086 => {
                    // If effective address is odd, add 4 extra clocks for each transfer
                    // if is_16_bits && (ea_address.offset & 0b1) == 0b1 {
                    //     transfers_clocks = 4;
                    // }
                }
                CpuVersion::Intel8088 => {
                    // Add 16 bits for each word transfer as the 8088 buss is 8 bits
                    if is_16_bits {
                        clocks.transfers = 4;
                    }
                }
            }
        } else if let Operand::Register(dst_reg_info) = destination
            && let Operand::Immediate(_) = source
        {
            // Accumulator (AX or AL) AH is not accumulator, immediate
            if dst_reg_info.register_name == RegisterName::A && dst_reg_info.offset != 1 {
                clocks.base = 4;
            } else {
                clocks.base = 5;
            }
        } else if let Operand::Memory(dst_mem_info) = destination
            && let Operand::Immediate(_) = source
        {
            clocks.base = 11;
            clocks.ea = self.calculate_clocks_effective_address(dst_mem_info);

            match self.cpu_version {
                CpuVersion::Intel8086 => (),
                CpuVersion::Intel8088 => {
                    if is_wide {
                        clocks.transfers = 4;
                    }
                }
            }
        }

        Some(clocks)
    }

    fn calculate_clocks_effective_address(&self, mem_displ_info: MemoryDisplacementInfo) -> i32 {
        if mem_displ_info.terms[0].register_name == RegisterName::BP
            && mem_displ_info.terms[1].register_name == RegisterName::SI
            && mem_displ_info.displacement > 0
        {
            return 12;
        } else if mem_displ_info.terms[0].register_name == RegisterName::B
            && mem_displ_info.terms[1].register_name == RegisterName::DI
            && mem_displ_info.displacement > 0
        {
            return 12;
        } else if mem_displ_info.terms[0].register_name == RegisterName::BP
            && mem_displ_info.terms[1].register_name == RegisterName::DI
            && mem_displ_info.displacement > 0
        {
            return 11;
        } else if mem_displ_info.terms[0].register_name == RegisterName::B
            && mem_displ_info.terms[1].register_name == RegisterName::SI
            && mem_displ_info.displacement > 0
        {
            return 12;
        } else if mem_displ_info.terms[0].register_name == RegisterName::BP
            && mem_displ_info.terms[1].register_name == RegisterName::SI
            && mem_displ_info.displacement == 0
        {
            return 8;
        } else if mem_displ_info.terms[0].register_name == RegisterName::B
            && mem_displ_info.terms[1].register_name == RegisterName::DI
            && mem_displ_info.displacement == 0
        {
            return 8;
        } else if mem_displ_info.terms[0].register_name == RegisterName::BP
            && mem_displ_info.terms[1].register_name == RegisterName::DI
            && mem_displ_info.displacement == 0
        {
            return 7;
        } else if mem_displ_info.terms[0].register_name == RegisterName::B
            && mem_displ_info.terms[1].register_name == RegisterName::SI
            && mem_displ_info.displacement == 0
        {
            return 7;
        } else if (mem_displ_info.terms[0].register_name == RegisterName::B
            || mem_displ_info.terms[0].register_name == RegisterName::BP
            || mem_displ_info.terms[0].register_name == RegisterName::SI
            || mem_displ_info.terms[0].register_name == RegisterName::DI)
            && mem_displ_info.terms[1].register_name == RegisterName::None
            && mem_displ_info.displacement > 0
        {
            return 9;
        } else if (mem_displ_info.terms[0].register_name == RegisterName::B
            || mem_displ_info.terms[0].register_name == RegisterName::BP
            || mem_displ_info.terms[0].register_name == RegisterName::SI
            || mem_displ_info.terms[0].register_name == RegisterName::DI)
            && mem_displ_info.terms[1].register_name == RegisterName::None
            && mem_displ_info.displacement == 0
        {
            return 5;
        } else if mem_displ_info.terms[0].register_name == RegisterName::None
            && mem_displ_info.terms[1].register_name == RegisterName::None
            && mem_displ_info.displacement > 0
        {
            return 6;
        }

        return 0;
    }

    fn report_flags_values(&mut self, prev_flags: u16, new_flags: u16) {
        let mut flags_before_report = String::with_capacity(6);
        if (prev_flags & CpuFlags::ZF.bits()) > 0 {
            flags_before_report.push_str("Z");
        }
        if (prev_flags & CpuFlags::SF.bits()) > 0 {
            flags_before_report.push_str("S");
        }
        if (prev_flags & CpuFlags::PF.bits()) > 0 {
            flags_before_report.push_str("P");
        }
        if (prev_flags & CpuFlags::CF.bits()) > 0 {
            flags_before_report.push_str("C");
        }
        if (prev_flags & CpuFlags::AF.bits()) > 0 {
            flags_before_report.push_str("A");
        }
        if (prev_flags & CpuFlags::OF.bits()) > 0 {
            flags_before_report.push_str("O");
        }

        let mut flags_after_report = String::with_capacity(6);
        if (new_flags & CpuFlags::ZF.bits()) > 0 {
            flags_after_report.push_str("Z");
        }
        if (new_flags & CpuFlags::SF.bits()) > 0 {
            flags_after_report.push_str("S");
        }
        if (new_flags & CpuFlags::PF.bits()) > 0 {
            flags_after_report.push_str("P");
        }
        if (new_flags & CpuFlags::CF.bits()) > 0 {
            flags_after_report.push_str("C");
        }
        if (new_flags & CpuFlags::AF.bits()) > 0 {
            flags_after_report.push_str("A");
        }
        if (new_flags & CpuFlags::OF.bits()) > 0 {
            flags_after_report.push_str("O");
        }
        if flags_before_report.len() | flags_after_report.len() > 0 {
            self.disassembler.add_comment_after_instruction(
                format!("| Flags: {flags_before_report} -> {flags_after_report}").as_str(),
            );
        }
    }

    pub fn save_to_file(&self, path: &str) -> Result<()> {
        self.disassembler.save_to_file(path)?;
        Ok(())
    }
}
