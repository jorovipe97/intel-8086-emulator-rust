use crate::instructions::encodings::CpuFlags;

use super::encodings::OperationType;
use super::operands::Operand;

// Holds the operands of the decoded instruction.
#[derive(Debug, Clone, Copy)]
pub struct OperandsUsage {
    pub destination: Operand,
    pub source: Operand,
}

/// Holds the information of an instruction after decoding from binary.
#[derive(Debug, Clone, Copy)]
pub struct DecodedInstruction {
    pub operation: OperationType,

    /// Size of the instruction in bytes.
    pub size: usize,

    /// The operands of the instruction.
    pub operands: OperandsUsage,

    /// Do this instruction have the W flag set to 1?
    pub is_w_field_set: bool,

    pub affected_cpu_flags: CpuFlags,
}

impl DecodedInstruction {
    pub const DEFAULT: Self = Self {
        operation: OperationType::None,
        size: 0,
        operands: OperandsUsage {
            destination: Operand::None,
            source: Operand::None,
        },
        is_w_field_set: false,
        affected_cpu_flags: CpuFlags::empty(),
    };
}
