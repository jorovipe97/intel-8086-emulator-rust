use super::encodings::RegisterName;
use std::fmt::{Display, Error, Formatter, Result};

#[derive(Debug, Clone, Copy)]
pub struct RegisterInfo {
    /// Which register (A, B, C, D, SP, BP, SI, DI)
    pub register_name: RegisterName,

    /// Byte offset within the 16-bit register (0 = low byte, 1 = high byte)
    pub offset: i32,

    /// Number of bytes accessed (1 = 8-bit, 2 = 16-bit)
    pub count: i32,
}

impl RegisterInfo {
    pub const NONE: RegisterInfo = RegisterInfo {
        register_name: RegisterName::None,
        count: 2,
        offset: 0,
    };
}

const REGISTERS_MAPPINGS: [[&'static str; 3]; 8] = [
    ["al", "ah", "ax"],
    ["cl", "ch", "cx"],
    ["dl", "dh", "dx"],
    ["bl", "bh", "bx"],
    ["sp", "sp", "sp"],
    ["bp", "bp", "bp"],
    ["si", "si", "si"],
    ["di", "di", "di"],
];
impl Display for RegisterInfo {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let row = self.register_name as usize;
        // Ensures we we dont have more than 8 registers.
        if row >= 8 {
            return Err(Error);
        }

        let col: usize = if self.count == 2 {
            2
        } else {
            // & 0b1 is a defensive progrraming thecnique,
            // in case offset is neither 0 or 1.
            (self.offset as usize) & 0b1
        };

        let register_name = REGISTERS_MAPPINGS[row][col];
        return write!(f, "{}", register_name);
    }
}

/// Reperesent the memory displacement operation on memory operands
#[derive(Debug, Clone, Copy)]
pub struct MemoryDisplacementInfo {
    pub terms: [RegisterInfo; 2],
    pub displacement: i32,
}

impl Display for MemoryDisplacementInfo {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let mut had_terms = false;
        write!(f, "[")?;
        for term in self.terms {
            if term.register_name == RegisterName::None {
                continue;
            }

            if had_terms {
                write!(f, "+")?;
            }

            write!(f, "{}", term.to_string())?;

            had_terms = true;
        }

        // Print the displacement if we had no register terms OR if the displacement is non-zero
        if !had_terms || self.displacement != 0 {
            write!(f, "+{}", self.displacement)?;
        }

        return write!(f, "]");
    }
}

/// The distint operand types that support the simulator
#[derive(Debug, Clone, Copy)]
pub enum Operand {
    /// To represent that the instruction dont have a given operand
    /// for example jmp instructions only have destination operand.
    /// so the source is Operand::None
    None,
    Register(RegisterInfo),
    Memory(MemoryDisplacementInfo),
    Immediate(i32),
    InstructionPointerIncrement(i32),
}
