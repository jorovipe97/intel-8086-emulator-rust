use std::fmt::{Display, Error, Formatter, Result, write};

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
    /// Segment registers.
    SR,

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

#[derive(Debug, Clone, Copy)]
pub struct ImmediateInfo {
    pub value: i32,
}

impl Display for ImmediateInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        return write!(f, "{}", self.value);
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
    Memory(MemoryDisplacementInfo),
    Immediate(ImmediateInfo),
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

    /// The operands of the instruction.
    pub operands: OperandsUsage,

    /// Do this instruction have the W flag set to 1?
    pub is_w_field_set: bool,
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
    pub shift: u8, // TODO: Delete?

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

const W_MAKES_DATA_WIDE: InstructionBits = InstructionBits {
    usage: InstructionBitsUsage::WMakesDataWide,
    bit_count: 0,
    value: 1,
    ..InstructionBits::DEFAULT
};

const D: InstructionBits = InstructionBits {
    usage: InstructionBitsUsage::D,
    bit_count: 1,
    ..InstructionBits::DEFAULT
};

const DATA: InstructionBits = InstructionBits {
    usage: InstructionBitsUsage::Data,
    ..InstructionBits::DEFAULT
};

const DATA_IF_W: InstructionBits = InstructionBits {
    usage: InstructionBitsUsage::Data,
    value: 1,
    ..InstructionBits::DEFAULT
};

/// Allows to declare an implicit d, so decoder knows if whould use reg field
/// as the destination (d==1) or the source (d==0).
const fn implicit_d(value: u8) -> InstructionBits {
    InstructionBits {
        usage: InstructionBitsUsage::D,
        bit_count: 0,
        shift: 0,
        value,
    }
}
// I first implemented this as a macro, but this was overcomplicating things, For this scenario
// a const fn is simpler and works.
// macro_rules! implicit_d {
//     ($val:literal) => {{
//         InstructionBits {
//             usage: InstructionBitsUsage::D,
//             bit_count: 0,
//             shift: 0,
//             value: $val,
//         }
//     }};
// }

/// Allows to declare an implicit reg, so decoder will always decode to the given reg
const fn implicit_reg(value: u8) -> InstructionBits {
    InstructionBits {
        usage: InstructionBitsUsage::Reg,
        bit_count: 0,
        shift: 0,
        value,
    }
}

/// Allows to declare an implicit Mod field
const fn implicit_mod(value: u8) -> InstructionBits {
    InstructionBits {
        usage: InstructionBitsUsage::Mod,
        bit_count: 0,
        shift: 0,
        value,
    }
}

/// Allows to declare an implicit R/M field
const fn implicit_rm(value: u8) -> InstructionBits {
    InstructionBits {
        usage: InstructionBitsUsage::Rm,
        bit_count: 0,
        shift: 0,
        value,
    }
}

const END: InstructionBits = InstructionBits::DEFAULT;

pub const INSTRUCTION_ENCODINGS_TABLE: &[InstructionEncoding] = &[
    InstructionEncoding {
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
    },
    InstructionEncoding {
        op: OperationType::Mov,
        bits: &[
            InstructionBits {
                // Immediate to register/memory
                usage: InstructionBitsUsage::Literal,
                bit_count: 7,
                value: 0b1100011,
                ..InstructionBits::DEFAULT
            },
            W,
            W_MAKES_DATA_WIDE,
            MOD,
            InstructionBits {
                usage: InstructionBitsUsage::Literal,
                bit_count: 3,
                value: 0b000,
                ..InstructionBits::DEFAULT
            },
            RM,
            implicit_d(0), // Destination is not in reg field
            DATA,
            DATA_IF_W,
        ],
    },
    InstructionEncoding {
        op: OperationType::Mov,
        bits: &[
            InstructionBits {
                // Immediate to register
                usage: InstructionBitsUsage::Literal,
                bit_count: 4,
                value: 0b1011,
                ..InstructionBits::DEFAULT
            },
            W,
            REG,
            implicit_d(1),
            DATA,
            DATA_IF_W,
        ],
    },
    InstructionEncoding {
        op: OperationType::Mov,
        bits: &[
            InstructionBits {
                // Memory to accumulator, This mode specifies the exact memory offset in the instruction. The segment is implicitly the Data Segment (DS)
                // Eg: mov ax, [123]
                usage: InstructionBitsUsage::Literal,
                bit_count: 7,
                value: 0b1010000,
                ..InstructionBits::DEFAULT
            },
            W,
            implicit_d(1),       // Destination is a reg field (The accumulator)
            implicit_reg(0b000), // 000 -> AX when w is 1. Or AL when w is 0.
            implicit_mod(0b00),  // Memory mode, no displacement follows...
            implicit_rm(0b110),  // ...except when R/M = 110. Then 16 bit displacement follows
        ],
    },
    InstructionEncoding {
        op: OperationType::Mov,
        bits: &[
            InstructionBits {
                // Accumulator to memory, This mode specifies the exact memory offset in the instruction.
                // The segment is implicitly the Data Segment (DS)
                // Eg: mov [123], ax
                usage: InstructionBitsUsage::Literal,
                bit_count: 7,
                value: 0b1010001,
                ..InstructionBits::DEFAULT
            },
            W,
            implicit_d(0),       // Source is the reg field (The accumulator)
            implicit_reg(0b000), // 000 -> AX when w is 1. Or AL when w is 0.
            implicit_mod(0b00),  // Memory mode, no displacement follows...
            implicit_rm(0b110),  // ...except when R/M = 110. Then 16 bit displacement follows
        ],
    },
];

/// NOTE(casey): This is the "Intel-specified" maximum length of an instruction, including prefixes\
pub const MAX_INSTRUCTION_BYTE_COUNT: u8 = 15;
