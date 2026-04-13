use super::encodings::OperationType;
use super::operands::Operand;
use crate::instructions::encodings::CpuFlags;
use bitflags::bitflags;

// Holds the operands of the decoded instruction.
#[derive(Debug, Clone, Copy)]
pub struct OperandsUsage {
    pub destination: Operand,
    pub source: Operand,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct DecodedInstructionExtraAttributes: u32 {
        const IS_WIDE = 1 << 0;
        const IS_INDIRECT_FAR_JUMP = 1 << 1;

        /// The decoded instruction has memory operands eg: [bp + 2]
        const HAS_EFFECTIVE_ADDRESS_CALCULATION = 1 << 2;
    }
}

/// Holds the information of an instruction after decoding from binary.
#[derive(Debug, Clone, Copy)]
pub struct DecodedInstruction {
    /// The prefixes of the decoded instruction, note that a given instruction
    /// may have multiple prefixes, ex lock cs mov ax, [bx]
    /// Prefixes are in a stack-allocated array because we know we may have up to 2 prefixes.
    pub prefixes: [OperationType; 2],

    /// Count of prefixes in the decoded instruciton.
    pub prefixes_count: usize,
    pub operation: OperationType,

    /// Size of the instruction in bytes.
    pub size: usize,

    /// The operands of the instruction.
    pub operands: OperandsUsage,

    /// Holds additional info needed by the disasembler or cpu simulation.
    /// eg: is data wide, or is a far jump.
    pub extra_attributes: DecodedInstructionExtraAttributes,

    pub affected_cpu_flags: CpuFlags,
}

impl DecodedInstruction {
    pub const DEFAULT: Self = Self {
        prefixes: [OperationType::None; 2],
        prefixes_count: 0,
        operation: OperationType::None,
        size: 0,
        operands: OperandsUsage {
            destination: Operand::None,
            source: Operand::None,
        },
        extra_attributes: DecodedInstructionExtraAttributes::empty(),
        affected_cpu_flags: CpuFlags::empty(),
    };
}
