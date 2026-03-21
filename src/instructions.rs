use std::fmt::{Display, Error, Formatter, Result};

use anyhow::anyhow;

/// Represents all possible instructions supported by the simulator
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OperationType {
    None,
    Mov,
}

impl OperationType {
    pub const NONE: OperationType = OperationType::None;
}

impl Display for OperationType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Mov => write!(f, "mov"),
            Self::None => write!(f, "none"),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum InstructionBitsUsage {
    // NOTE(casey): The 0 value, indicating the end of the instruction encoding array
    End,
    Literal,
    D,
    S,
    W,
    Mod,
    Reg,
    Rm,
    Disp,
    Data,
    WMakesDataWide,
    SR,
    IpInc,
    // Used to track how many possible bits usages we support, this is not an actual flag in 8086.
    Count,
}

/// The general purpose registers, order of the values matter in decoding.
#[derive(Debug, Clone, Copy)]
pub enum RegisterName {
    A,
    C,
    D,
    B,
    SP,
    BP,
    SI,
    DI,
}

#[derive(Debug, Clone, Copy)]
pub struct RegisterInfo {
    /// Which register (A, B, C, D, SP, BP, SI, DI)
    pub register_name: RegisterName,

    /// Byte offset within the 16-bit register (0 = low byte, 1 = high byte)
    pub offset: i32,

    /// Number of bytes accessed (1 = 8-bit, 2 = 16-bit)
    pub count: i32,
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

        let mut col: usize = 0;
        if self.count == 2 {
            col = 2;
        } else {
            // & 0b1 is a defensive progrraming thecnique,
            // in case offset is neither 0 or 1.
            col = (self.offset as usize) & 0b1;
        }

        let register_name = REGISTERS_MAPPINGS[row][col];
        return write!(f, "{}", register_name);
    }
}

/// The distint operand types that support the simulator
#[derive(Debug, Clone, Copy)]
pub enum Operand {
    /// To represent that the instruction dont have this operan
    /// for example jmp instructions only have destination operand.
    /// so the source is Operand::None
    None,
    Register(RegisterInfo),
}

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

    pub operands: OperandsUsage,
}

impl DecodedInstruction {
    pub const DEFAULT: Self = Self {
        operation: OperationType::None,
        size: 0,
        operands: OperandsUsage {
            destination: Operand::None,
            source: Operand::None,
        },
    };
}

#[derive(Clone, Copy)]
pub struct InstructionBits {
    // The usage of these bits, ef reg, rm, mod, etc.
    pub usage: InstructionBitsUsage,

    // Number of bits for this part, eg reg field have 3 bits
    pub bit_count: u8,

    // Ammount we need to left shift the original byte (8 bits)
    // so we extract the field, eg 00reg000. Needs a 3 bits shift
    pub shift: u8,

    // The actual bytes, depending on the usage this may need different things.
    // Eg if usage is a literal, then this is the opcode it should match.
    pub value: u8,
}

impl InstructionBits {
    const DEFAULT: Self = Self {
        usage: InstructionBitsUsage::End,
        bit_count: 0,
        shift: 0,
        value: 0,
    };
}

pub struct InstructionEncoding {
    pub op: OperationType,

    // Each item represent a part of the entire instruction encoding. Eg:
    // reg field which are 3 bytes
    //
    // We use 'static lifetime because we want this to be at the executable
    // and know this will never change.
    pub bits: &'static [InstructionBits],
}

const MOD: InstructionBits = InstructionBits {
    usage: InstructionBitsUsage::Mod,
    bit_count: 2,
    ..InstructionBits::DEFAULT
};

const REG: InstructionBits = InstructionBits {
    usage: InstructionBitsUsage::Reg,
    bit_count: 3,
    ..InstructionBits::DEFAULT
};

const RM: InstructionBits = InstructionBits {
    usage: InstructionBitsUsage::Rm,
    bit_count: 3,
    ..InstructionBits::DEFAULT
};

const W: InstructionBits = InstructionBits {
    usage: InstructionBitsUsage::W,
    bit_count: 1,
    ..InstructionBits::DEFAULT
};

const D: InstructionBits = InstructionBits {
    usage: InstructionBitsUsage::D,
    bit_count: 1,
    ..InstructionBits::DEFAULT
};

const END: InstructionBits = InstructionBits::DEFAULT;

pub const INSTRUCTION_ENCODINGS_TABLE: &[InstructionEncoding] = &[InstructionEncoding {
    op: OperationType::Mov,
    bits: &[
        InstructionBits {
            // Register/memory to/from register
            usage: InstructionBitsUsage::Literal,
            bit_count: 6,
            value: 0b100010,
            ..InstructionBits::DEFAULT
        },
        D,
        W,
        MOD,
        REG,
        RM,
    ],
}];

/// NOTE(casey): This is the "Intel-specified" maximum length of an instruction, including prefixes\
pub const MAX_INSTRUCTION_BYTE_COUNT: u8 = 15;
