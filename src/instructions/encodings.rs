use std::fmt::{Display, Formatter, Result};

/// Represents all possible instructions supported by the simulator
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OperationType {
    None,
    Mov,
    Add,
    Sub,
    Cmp,
    Jnz,
    Je,
    Jl,
    Jle,
    Jb,
    Jbe,
    Jp,
    Jo,
    Js,
    Jne,
    Jnl,
    Jnle,
    Jnb,
    Jnbe,
    Jnp,
    Jno,
    Jns,
    /// Loop CX times.
    Loop,
    /// Loop while zero/equal
    LoopZ,
    /// Loop while not zero/equal
    LoopNz,
    /// Jump on CX zero
    Jcxz,
}

impl OperationType {
    pub const NONE: OperationType = OperationType::None;
}

impl Display for OperationType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Mov => write!(f, "mov"),
            Self::None => write!(f, "none"),
            Self::Add => write!(f, "add"),
            Self::Sub => write!(f, "sub"),
            Self::Cmp => write!(f, "cmp"),
            Self::Jnz => write!(f, "jnz"),
            Self::Je => write!(f, "je"),
            Self::Jl => write!(f, "jl"),
            Self::Jle => write!(f, "jle"),
            Self::Jb => write!(f, "jb"),
            Self::Jbe => write!(f, "jbe"),
            Self::Jp => write!(f, "jp"),
            Self::Jo => write!(f, "jo"),
            Self::Js => write!(f, "js"),
            Self::Jne => write!(f, "js"),
            Self::Jnl => write!(f, "jnl"),
            Self::Jnle => write!(f, "jnle"),
            Self::Jnb => write!(f, "jnb"),
            Self::Jnbe => write!(f, "jnbe"),
            Self::Jnp => write!(f, "jnp"),
            Self::Jno => write!(f, "jno"),
            Self::Jns => write!(f, "jns"),
            Self::Loop => write!(f, "loop"),
            Self::LoopZ => write!(f, "loopz"),
            Self::LoopNz => write!(f, "loopnz"),
            Self::Jcxz => write!(f, "jcxz"),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum InstructionBitsUsage {
    // NOTE(casey): The 0 value, indicating the end of the instruction encoding array
    End,
    Literal,
    D,

    /// If S = 0; No sign extension.
    /// If S = 1; Sign extend 8-bit immediate data to 16 bits.
    S,
    W,
    Mod,
    Reg,
    Rm,
    Disp,
    Data,
    WMakesDataWide,
    /// Segment registers.
    // SR,

    /// Instruction Pointer Increment.
    IpInc,

    // Used to track how many possible bits usages we support, this is not an actual flag in 8086.
    // TODO: Can we remove it?
    Count,
}

/// The general purpose registers, order of the values matter in decoding.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RegisterName {
    A,
    C,
    D,
    B,
    SP,
    BP,
    SI,
    DI,
    None,
}

#[derive(Clone, Copy)]
pub struct InstructionBits {
    // The usage of these bits, ef reg, rm, mod, etc.
    pub usage: InstructionBitsUsage,

    // Number of bits for this part, eg reg field have 3 bits
    pub bit_count: u8,

    // The actual bytes, depending on the usage this may need different things.
    // Eg if usage is a literal, then this is the opcode it should match.
    pub value: u8,
}

impl InstructionBits {
    pub const DEFAULT: Self = Self {
        usage: InstructionBitsUsage::End,
        bit_count: 0,
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
