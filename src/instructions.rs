pub mod decoded_instruction;
pub mod encodings;
pub mod operands;

use encodings::{InstructionBits, InstructionBitsUsage, InstructionEncoding, OperationType};

use crate::instructions::encodings::CpuFlags;

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

const SR: InstructionBits = InstructionBits {
    usage: InstructionBitsUsage::SR,
    bit_count: 2,
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
};

const S: InstructionBits = InstructionBits {
    usage: InstructionBitsUsage::S,
    bit_count: 1,
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

const IP_INC: InstructionBits = InstructionBits {
    usage: InstructionBitsUsage::IpInc,
    ..InstructionBits::DEFAULT
};

/// Allows to declare an implicit d, so decoder knows if should use reg field
/// as the destination (d==1) or the source (d==0).
const fn implicit_d(value: u8) -> InstructionBits {
    InstructionBits {
        usage: InstructionBitsUsage::D,
        bit_count: 0,
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

/// Allows to declare an implicit w, so decoder knows if should use reg of 16 bits
/// or 8 bits.
const fn implicit_w(value: u8) -> InstructionBits {
    InstructionBits {
        usage: InstructionBitsUsage::W,
        bit_count: 0,
        value,
    }
}

/// When an instruction has two registers operands, this explicitly set
/// the width (w) of the register calculated using mod+rm fields.
///
/// This is useful when you want to force the mod+rm register to have a different
/// width than the register calculated using the reg + w field.
const fn implicit_mod_rm_w(value: u8) -> InstructionBits {
    InstructionBits {
        usage: InstructionBitsUsage::ModRmW,
        bit_count: 0,
        value,
    }
}

/// Allows to declare an implicit reg, so decoder will always decode to the given reg
const fn implicit_reg(value: u8) -> InstructionBits {
    InstructionBits {
        usage: InstructionBitsUsage::Reg,
        bit_count: 0,
        value,
    }
}

/// Allows to declare an implicit Mod field
const fn implicit_mod(value: u8) -> InstructionBits {
    InstructionBits {
        usage: InstructionBitsUsage::Mod,
        bit_count: 0,
        value,
    }
}

/// Allows to declare an implicit R/M field
const fn implicit_rm(value: u8) -> InstructionBits {
    InstructionBits {
        usage: InstructionBitsUsage::Rm,
        bit_count: 0,
        value,
    }
}

const ARITHMETIC_AND_LOGIC_FLAGS: CpuFlags = CpuFlags::from_bits_truncate(
    CpuFlags::CF.bits()
        | CpuFlags::ZF.bits()
        | CpuFlags::SF.bits()
        | CpuFlags::OF.bits()
        | CpuFlags::PF.bits()
        | CpuFlags::AF.bits(),
);

/// This table hold the encodings of all the instructions suported by this
/// emulator.
pub const INSTRUCTION_ENCODINGS_TABLE: &[InstructionEncoding] = &[
    InstructionEncoding {
        op: OperationType::Mov,
        bits: &[
            InstructionBits {
                // Register/memory to/from register
                usage: InstructionBitsUsage::Literal,
                bit_count: 6,
                value: 0b100010,
            },
            D,
            W,
            MOD,
            REG,
            RM,
        ],
        affected_cpu_flags: CpuFlags::empty(), // No flags affected,
    },
    InstructionEncoding {
        op: OperationType::Mov,
        bits: &[
            InstructionBits {
                // Immediate to register/memory
                usage: InstructionBitsUsage::Literal,
                bit_count: 7,
                value: 0b1100011,
            },
            W,
            W_MAKES_DATA_WIDE,
            MOD,
            InstructionBits {
                usage: InstructionBitsUsage::Literal,
                bit_count: 3,
                value: 0b000,
            },
            RM,
            implicit_d(0), // Destination is not in reg field
            DATA,
            DATA_IF_W,
        ],
        affected_cpu_flags: CpuFlags::empty(), // No flags affected,
    },
    InstructionEncoding {
        op: OperationType::Mov,
        bits: &[
            InstructionBits {
                // Immediate to register
                usage: InstructionBitsUsage::Literal,
                bit_count: 4,
                value: 0b1011,
            },
            W,
            W_MAKES_DATA_WIDE,
            REG,
            implicit_d(1),
            DATA,
            DATA_IF_W,
        ],
        affected_cpu_flags: CpuFlags::empty(), // No flags affected,
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
            },
            W,
            implicit_d(1),       // Destination is a reg field (The accumulator)
            implicit_reg(0b000), // 000 -> AX when w is 1. Or AL when w is 0.
            implicit_mod(0b00),  // Memory mode, no displacement follows...
            implicit_rm(0b110),  // ...except when R/M = 110. Then 16 bit displacement follows
        ],
        affected_cpu_flags: CpuFlags::empty(), // No flags affected,
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
            },
            W,
            implicit_d(0),       // Source is the reg field (The accumulator)
            implicit_reg(0b000), // 000 -> AX when w is 1. Or AL when w is 0.
            implicit_mod(0b00),  // Memory mode, no displacement follows...
            implicit_rm(0b110),  // ...except when R/M = 110. Then 16 bit displacement follows
        ],
        affected_cpu_flags: CpuFlags::empty(), // No flags affected,
    },
    InstructionEncoding {
        op: OperationType::Mov,
        bits: &[
            InstructionBits {
                // Register/memory to segment register.
                usage: InstructionBitsUsage::Literal,
                bit_count: 8,
                value: 0b1000_1110,
            },
            MOD,
            InstructionBits {
                usage: InstructionBitsUsage::Literal,
                bit_count: 1,
                value: 0b0,
            },
            SR,
            RM,
            implicit_w(1), // We assumme always wide, as segment register only support 16 bits data.
            implicit_d(1), // Segment register acts as destination.
        ],
        affected_cpu_flags: CpuFlags::empty(), // No flags affected,
    },
    InstructionEncoding {
        op: OperationType::Mov,
        bits: &[
            InstructionBits {
                // Segment register to Register/Memory
                usage: InstructionBitsUsage::Literal,
                bit_count: 8,
                value: 0b1000_1100,
            },
            MOD,
            InstructionBits {
                usage: InstructionBitsUsage::Literal,
                bit_count: 1,
                value: 0b0,
            },
            SR,
            RM,
            implicit_w(1), // We assumme always wide, as segment register only support 16 bits data.
            implicit_d(0), // Segment register acts as source.
        ],
        affected_cpu_flags: CpuFlags::empty(), // No flags affected,
    },
    InstructionEncoding {
        op: OperationType::Add,
        bits: &[
            InstructionBits {
                // Register/memory with register to either
                usage: InstructionBitsUsage::Literal,
                bit_count: 6,
                value: 0b000000,
            },
            D,
            W,
            MOD,
            REG,
            RM,
        ],
        affected_cpu_flags: ARITHMETIC_AND_LOGIC_FLAGS,
    },
    InstructionEncoding {
        op: OperationType::Add,
        bits: &[
            InstructionBits {
                // Immediate to register/memory
                usage: InstructionBitsUsage::Literal,
                bit_count: 6,
                value: 0b100000,
            },
            S,
            W,
            W_MAKES_DATA_WIDE,
            implicit_d(0), // Destination is not in reg field. If destination is register it is in rm field.
            MOD,
            InstructionBits {
                usage: InstructionBitsUsage::Literal,
                bit_count: 3,
                value: 0b000,
            },
            RM,
            DATA,
            DATA_IF_W,
        ],
        affected_cpu_flags: ARITHMETIC_AND_LOGIC_FLAGS,
    },
    InstructionEncoding {
        op: OperationType::Add,
        bits: &[
            InstructionBits {
                // Immediate to accumulator (A)
                usage: InstructionBitsUsage::Literal,
                bit_count: 7,
                value: 0b0000010,
            },
            W,
            W_MAKES_DATA_WIDE,
            implicit_d(1),       // Destination is the reg field (The accumulator)
            implicit_reg(0b000), // 000 -> AX when w is 1. Or AL when w is 0.
            // implicit_mod(0b11),  // Register mode
            DATA,
            DATA_IF_W,
        ],
        affected_cpu_flags: ARITHMETIC_AND_LOGIC_FLAGS,
    },
    InstructionEncoding {
        op: OperationType::Sub,
        bits: &[
            InstructionBits {
                // Register/memory with register to either
                usage: InstructionBitsUsage::Literal,
                bit_count: 6,
                value: 0b001010,
            },
            D,
            W,
            MOD,
            REG,
            RM,
        ],
        affected_cpu_flags: ARITHMETIC_AND_LOGIC_FLAGS,
    },
    InstructionEncoding {
        op: OperationType::Sub,
        bits: &[
            InstructionBits {
                // Immediate to register/memory
                usage: InstructionBitsUsage::Literal,
                bit_count: 6,
                value: 0b100000,
            },
            S,
            W,
            W_MAKES_DATA_WIDE,
            implicit_d(0), // Destination is not in reg field. If destination is register it is in rm field.
            MOD,
            InstructionBits {
                usage: InstructionBitsUsage::Literal,
                bit_count: 3,
                value: 0b101,
            },
            RM,
            DATA,
            DATA_IF_W,
        ],
        affected_cpu_flags: ARITHMETIC_AND_LOGIC_FLAGS,
    },
    InstructionEncoding {
        op: OperationType::Sub,
        bits: &[
            InstructionBits {
                // Immediate to accumulator (A)
                usage: InstructionBitsUsage::Literal,
                bit_count: 7,
                value: 0b0010110,
            },
            W,
            W_MAKES_DATA_WIDE,
            implicit_d(1),       // Destination is the reg field (The accumulator)
            implicit_reg(0b000), // 000 -> AX when w is 1. Or AL when w is 0.
            // implicit_mod(0b11),  // Register mode
            DATA,
            DATA_IF_W,
        ],
        affected_cpu_flags: ARITHMETIC_AND_LOGIC_FLAGS,
    },
    InstructionEncoding {
        op: OperationType::Cmp,
        bits: &[
            InstructionBits {
                // Register/memory with register to either
                usage: InstructionBitsUsage::Literal,
                bit_count: 6,
                value: 0b001110,
            },
            D,
            W,
            MOD,
            REG,
            RM,
        ],
        affected_cpu_flags: ARITHMETIC_AND_LOGIC_FLAGS,
    },
    InstructionEncoding {
        op: OperationType::Cmp,
        bits: &[
            InstructionBits {
                // Immediate to register/memory
                usage: InstructionBitsUsage::Literal,
                bit_count: 6,
                value: 0b100000,
            },
            S,
            W,
            W_MAKES_DATA_WIDE,
            implicit_d(0), // Destination is not in reg field. If destination is register it is in rm field.
            MOD,
            InstructionBits {
                usage: InstructionBitsUsage::Literal,
                bit_count: 3,
                value: 0b111,
            },
            RM,
            DATA,
            DATA_IF_W,
        ],
        affected_cpu_flags: ARITHMETIC_AND_LOGIC_FLAGS,
    },
    InstructionEncoding {
        op: OperationType::Cmp,
        bits: &[
            InstructionBits {
                // Immediate to accumulator (A)
                usage: InstructionBitsUsage::Literal,
                bit_count: 7,
                value: 0b0011110,
            },
            W,
            W_MAKES_DATA_WIDE,
            implicit_d(1),       // Destination is the reg field (The accumulator)
            implicit_reg(0b000), // 000 -> AX when w is 1. Or AL when w is 0.
            // implicit_mod(0b11),  // Register mode
            DATA,
            DATA_IF_W,
        ],
        affected_cpu_flags: ARITHMETIC_AND_LOGIC_FLAGS,
    },
    InstructionEncoding {
        op: OperationType::Push,
        bits: &[
            // Register/Memory
            InstructionBits {
                usage: InstructionBitsUsage::Literal,
                bit_count: 8,
                value: 0b1111_1111,
            },
            MOD,
            implicit_d(1), // Set to 1, so that source operand is computed from the mod field.
            InstructionBits {
                usage: InstructionBitsUsage::Literal,
                bit_count: 3,
                value: 0b110,
            },
            RM,
        ],
        affected_cpu_flags: CpuFlags::empty(),
    },
    InstructionEncoding {
        op: OperationType::Push,
        bits: &[
            // Register
            InstructionBits {
                usage: InstructionBitsUsage::Literal,
                bit_count: 5,
                value: 0b01010,
            },
            REG,
            implicit_w(1), // Always use 16-bits register, as push don't support 8 bit operand.
            implicit_d(0), // Set to 0, so that source operand is computed from the reg field.
        ],
        affected_cpu_flags: CpuFlags::empty(),
    },
    InstructionEncoding {
        op: OperationType::Push,
        bits: &[
            // Segment register
            InstructionBits {
                usage: InstructionBitsUsage::Literal,
                bit_count: 3,
                value: 0b000,
            },
            SR,
            InstructionBits {
                usage: InstructionBitsUsage::Literal,
                bit_count: 3,
                value: 0b110,
            },
            implicit_d(0), // Set to 0, so that source operand is computed from the segment reg field.
        ],
        affected_cpu_flags: CpuFlags::empty(),
    },
    InstructionEncoding {
        op: OperationType::Pop,
        bits: &[
            // Register/Memory
            InstructionBits {
                usage: InstructionBitsUsage::Literal,
                bit_count: 8,
                value: 0b1000_1111,
            },
            MOD,
            implicit_d(0), // Set to 0, so that destination operand is computed from the mod field.
            InstructionBits {
                usage: InstructionBitsUsage::Literal,
                bit_count: 3,
                value: 0b000,
            },
            RM,
        ],
        affected_cpu_flags: CpuFlags::empty(),
    },
    InstructionEncoding {
        op: OperationType::Pop,
        bits: &[
            // Register
            InstructionBits {
                usage: InstructionBitsUsage::Literal,
                bit_count: 5,
                value: 0b01011,
            },
            REG,
            implicit_w(1), // Always use 16-bits register, as pop don't support 8 bit operand.
            implicit_d(1), // Set to 1, so that destination operand is computed from the reg field.
        ],
        affected_cpu_flags: CpuFlags::empty(),
    },
    InstructionEncoding {
        op: OperationType::Pop,
        bits: &[
            // Segment register
            InstructionBits {
                usage: InstructionBitsUsage::Literal,
                bit_count: 3,
                value: 0b000,
            },
            SR,
            InstructionBits {
                usage: InstructionBitsUsage::Literal,
                bit_count: 3,
                value: 0b111,
            },
            implicit_d(1), // Set to 1, so that destination operand is computed from the segment reg field.
        ],
        affected_cpu_flags: CpuFlags::empty(),
    },
    InstructionEncoding {
        op: OperationType::Xchg,
        bits: &[
            InstructionBits {
                // Register/memory with register
                usage: InstructionBitsUsage::Literal,
                bit_count: 7,
                value: 0b1000011,
            },
            // This just to always calculate destination from reg field and source from mod field,
            // note that semantically both operands are considered source and destinations as we are doing a swap.
            // xchg ax, [mem]
            // is exactly the same as
            // xchg [mem], ax
            implicit_d(1),
            W,
            MOD,
            REG,
            RM,
        ],
        affected_cpu_flags: CpuFlags::empty(), // No flags affected,
    },
    InstructionEncoding {
        op: OperationType::Xchg,
        bits: &[
            InstructionBits {
                // Register with accumulator
                usage: InstructionBitsUsage::Literal,
                bit_count: 5,
                value: 0b10010,
            },
            REG,
            // This just to always calculate destination from reg field and source from mod field,
            // note that semantically both operands are considered source and destinations as we are doing a swap.
            // xchg ax, [mem]
            // is exactly the same as
            // xchg [mem], ax
            implicit_d(1),
            // This mode always operates on 16 bit register
            implicit_w(1),
            implicit_mod(0b11), // Register
            implicit_rm(0b000), // Always AX
        ],
        affected_cpu_flags: CpuFlags::empty(), // No flags affected,
    },
    InstructionEncoding {
        op: OperationType::In,
        bits: &[
            InstructionBits {
                // Fixed port
                usage: InstructionBitsUsage::Literal,
                bit_count: 7,
                value: 0b1110010,
            },
            W,
            // Destination is always the accumulator
            implicit_d(1),
            // AX if w=1, or AL if w=0
            implicit_reg(0b000),
            DATA,
        ],
        affected_cpu_flags: CpuFlags::empty(), // No flags affected,
    },
    InstructionEncoding {
        op: OperationType::In,
        bits: &[
            InstructionBits {
                // Variable port
                usage: InstructionBitsUsage::Literal,
                bit_count: 7,
                value: 0b1110110,
            },
            W,
            // Destination is always the accumulator
            implicit_d(1),
            // AX if w=1, or AL if w=0
            implicit_reg(0b000),
            // Forces the register calculated using mod+rm always have
            // a (w)idth of 1 (16 bits) no matter the value of the W flag.
            implicit_mod_rm_w(1),
            // Source operand is a register
            implicit_mod(0b11),
            // Always DX register
            implicit_rm(0b010),
        ],
        affected_cpu_flags: CpuFlags::empty(), // No flags affected,
    },
    InstructionEncoding {
        op: OperationType::Out,
        bits: &[
            InstructionBits {
                // Fixed port
                usage: InstructionBitsUsage::Literal,
                bit_count: 7,
                value: 0b1110011,
            },
            W,
            // Source is always the accumulator
            implicit_d(0),
            // AX if w=1, or AL if w=0
            implicit_reg(0b000),
            DATA,
        ],
        affected_cpu_flags: CpuFlags::empty(), // No flags affected,
    },
    InstructionEncoding {
        op: OperationType::Out,
        bits: &[
            InstructionBits {
                // Variable port
                usage: InstructionBitsUsage::Literal,
                bit_count: 7,
                value: 0b1110111,
            },
            W,
            // Source is always the accumulator
            implicit_d(0),
            // AX if w=1, or AL if w=0
            implicit_reg(0b000),
            // Forces the register calculated using mod+rm always have
            // a (w)idth of 1 (16 bits) no matter the value of the W flag.
            implicit_mod_rm_w(1),
            // Source operand is a register
            implicit_mod(0b11),
            // Always DX register
            implicit_rm(0b010),
        ],
        affected_cpu_flags: CpuFlags::empty(), // No flags affected,
    },
    InstructionEncoding {
        op: OperationType::Xlat,
        bits: &[InstructionBits {
            usage: InstructionBitsUsage::Literal,
            bit_count: 8,
            value: 0b11010111,
        }],
        affected_cpu_flags: CpuFlags::empty(),
    },
    InstructionEncoding {
        op: OperationType::Lea,
        bits: &[
            InstructionBits {
                usage: InstructionBitsUsage::Literal,
                bit_count: 8,
                value: 0b10001101,
            },
            implicit_d(1), // Register is always the destination.
            implicit_w(1), // Destination is always 16 bits
            MOD,
            REG,
            RM,
        ],
        affected_cpu_flags: CpuFlags::empty(),
    },
    InstructionEncoding {
        op: OperationType::Lds,
        bits: &[
            InstructionBits {
                usage: InstructionBitsUsage::Literal,
                bit_count: 8,
                value: 0b11000101,
            },
            implicit_d(1), // Register is always the destination.
            implicit_w(1), // Destination is always 16 bits
            MOD,
            REG,
            RM,
        ],
        affected_cpu_flags: CpuFlags::empty(),
    },
    InstructionEncoding {
        op: OperationType::Les,
        bits: &[
            InstructionBits {
                usage: InstructionBitsUsage::Literal,
                bit_count: 8,
                value: 0b11000100,
            },
            implicit_d(1), // Register is always the destination.
            implicit_w(1), // Destination is always 16 bits
            MOD,
            REG,
            RM,
        ],
        affected_cpu_flags: CpuFlags::empty(),
    },
    InstructionEncoding {
        op: OperationType::Jnz,
        bits: &[
            InstructionBits {
                usage: InstructionBitsUsage::Literal,
                bit_count: 8,
                value: 0b0111_0101,
            },
            // Altough this is x86 reference, the jmp instructions has a single destination operand.
            // See: https://www.felixcloutier.com/x86/jmp
            // For this case, the target operand specifies a relative offset (a signed displacement relative to the current value of the instruction pointer in the IP register).
            // A near jump to a relative offset of 8-bits (rel8) is referred to as a short jump. The CS register is not changed on near and short jumps.
            //
            // The BitsIpInc is to indicate that the destination operand is an Instruction Pointer Increment
            // However the actual data is extracted from data.
            IP_INC,
            // Destination is in the mod operand.
            implicit_d(0),
        ],
        affected_cpu_flags: CpuFlags::empty(),
    },
    InstructionEncoding {
        op: OperationType::Je,
        bits: &[
            InstructionBits {
                usage: InstructionBitsUsage::Literal,
                bit_count: 8,
                value: 0b0111_0100,
            },
            IP_INC,
            // Destination is in the mod operand.
            implicit_d(0),
        ],
        affected_cpu_flags: CpuFlags::empty(),
    },
    InstructionEncoding {
        op: OperationType::Jl,
        bits: &[
            InstructionBits {
                usage: InstructionBitsUsage::Literal,
                bit_count: 8,
                value: 0b0111_1100,
            },
            IP_INC,
            // Destination is in the mod operand.
            implicit_d(0),
        ],
        affected_cpu_flags: CpuFlags::empty(),
    },
    InstructionEncoding {
        op: OperationType::Jle,
        bits: &[
            InstructionBits {
                usage: InstructionBitsUsage::Literal,
                bit_count: 8,
                value: 0b0111_1110,
            },
            IP_INC,
            // Destination is in the mod operand.
            implicit_d(0),
        ],
        affected_cpu_flags: CpuFlags::empty(),
    },
    InstructionEncoding {
        op: OperationType::Jb,
        bits: &[
            InstructionBits {
                usage: InstructionBitsUsage::Literal,
                bit_count: 8,
                value: 0b0111_0010,
            },
            IP_INC,
            // Destination is in the mod operand.
            implicit_d(0),
        ],
        affected_cpu_flags: CpuFlags::empty(),
    },
    InstructionEncoding {
        op: OperationType::Jbe,
        bits: &[
            InstructionBits {
                usage: InstructionBitsUsage::Literal,
                bit_count: 8,
                value: 0b0111_0110,
            },
            IP_INC,
            // Destination is in the mod operand.
            implicit_d(0),
        ],
        affected_cpu_flags: CpuFlags::empty(),
    },
    InstructionEncoding {
        op: OperationType::Jp,
        bits: &[
            InstructionBits {
                usage: InstructionBitsUsage::Literal,
                bit_count: 8,
                value: 0b0111_1010,
            },
            IP_INC,
            // Destination is in the mod operand.
            implicit_d(0),
        ],
        affected_cpu_flags: CpuFlags::empty(),
    },
    InstructionEncoding {
        op: OperationType::Jo,
        bits: &[
            InstructionBits {
                usage: InstructionBitsUsage::Literal,
                bit_count: 8,
                value: 0b0111_0000,
            },
            IP_INC,
            // Destination is in the mod operand.
            implicit_d(0),
        ],
        affected_cpu_flags: CpuFlags::empty(),
    },
    InstructionEncoding {
        op: OperationType::Js,
        bits: &[
            InstructionBits {
                usage: InstructionBitsUsage::Literal,
                bit_count: 8,
                value: 0b0111_1000,
            },
            IP_INC,
            // Destination is in the mod operand.
            implicit_d(0),
        ],
        affected_cpu_flags: CpuFlags::empty(),
    },
    InstructionEncoding {
        op: OperationType::Jne,
        bits: &[
            InstructionBits {
                usage: InstructionBitsUsage::Literal,
                bit_count: 8,
                value: 0b0111_0101,
            },
            IP_INC,
            // Destination is in the mod operand.
            implicit_d(0),
        ],
        affected_cpu_flags: CpuFlags::empty(),
    },
    InstructionEncoding {
        op: OperationType::Jnl,
        bits: &[
            InstructionBits {
                usage: InstructionBitsUsage::Literal,
                bit_count: 8,
                value: 0b0111_1101,
            },
            IP_INC,
            // Destination is in the mod operand.
            implicit_d(0),
        ],
        affected_cpu_flags: CpuFlags::empty(),
    },
    InstructionEncoding {
        op: OperationType::Jnle,
        bits: &[
            InstructionBits {
                usage: InstructionBitsUsage::Literal,
                bit_count: 8,
                value: 0b0111_1111,
            },
            IP_INC,
            // Destination is in the mod operand.
            implicit_d(0),
        ],
        affected_cpu_flags: CpuFlags::empty(),
    },
    InstructionEncoding {
        op: OperationType::Jnb,
        bits: &[
            InstructionBits {
                usage: InstructionBitsUsage::Literal,
                bit_count: 8,
                value: 0b0111_0011,
            },
            IP_INC,
            // Destination is in the mod operand.
            implicit_d(0),
        ],
        affected_cpu_flags: CpuFlags::empty(),
    },
    InstructionEncoding {
        op: OperationType::Jnbe,
        bits: &[
            InstructionBits {
                usage: InstructionBitsUsage::Literal,
                bit_count: 8,
                value: 0b0111_0111,
            },
            IP_INC,
            // Destination is in the mod operand.
            implicit_d(0),
        ],
        affected_cpu_flags: CpuFlags::empty(),
    },
    InstructionEncoding {
        op: OperationType::Jnp,
        bits: &[
            InstructionBits {
                usage: InstructionBitsUsage::Literal,
                bit_count: 8,
                value: 0b0111_1011,
            },
            IP_INC,
            // Destination is in the mod operand.
            implicit_d(0),
        ],
        affected_cpu_flags: CpuFlags::empty(),
    },
    InstructionEncoding {
        op: OperationType::Jno,
        bits: &[
            InstructionBits {
                usage: InstructionBitsUsage::Literal,
                bit_count: 8,
                value: 0b0111_0001,
            },
            IP_INC,
            // Destination is in the mod operand.
            implicit_d(0),
        ],
        affected_cpu_flags: CpuFlags::empty(),
    },
    InstructionEncoding {
        op: OperationType::Jns,
        bits: &[
            InstructionBits {
                usage: InstructionBitsUsage::Literal,
                bit_count: 8,
                value: 0b0111_1001,
            },
            IP_INC,
            // Destination is in the mod operand.
            implicit_d(0),
        ],
        affected_cpu_flags: CpuFlags::empty(),
    },
    InstructionEncoding {
        op: OperationType::Loop,
        bits: &[
            InstructionBits {
                usage: InstructionBitsUsage::Literal,
                bit_count: 8,
                value: 0b1110_0010,
            },
            IP_INC,
            // Destination is in the mod operand.
            implicit_d(0),
        ],
        affected_cpu_flags: CpuFlags::empty(),
    },
    InstructionEncoding {
        op: OperationType::LoopZ,
        bits: &[
            InstructionBits {
                usage: InstructionBitsUsage::Literal,
                bit_count: 8,
                value: 0b1110_0001,
            },
            IP_INC,
            // Destination is in the mod operand.
            implicit_d(0),
        ],
        affected_cpu_flags: CpuFlags::empty(),
    },
    InstructionEncoding {
        op: OperationType::LoopNz,
        bits: &[
            InstructionBits {
                usage: InstructionBitsUsage::Literal,
                bit_count: 8,
                value: 0b1110_0000,
            },
            IP_INC,
            // Destination is in the mod operand.
            implicit_d(0),
        ],
        affected_cpu_flags: CpuFlags::empty(),
    },
    InstructionEncoding {
        op: OperationType::Jcxz,
        bits: &[
            InstructionBits {
                usage: InstructionBitsUsage::Literal,
                bit_count: 8,
                value: 0b1110_0011,
            },
            IP_INC,
            // Destination is in the mod operand.
            implicit_d(0),
        ],
        affected_cpu_flags: CpuFlags::empty(),
    },
];

/// NOTE(casey): This is the "Intel-specified" maximum length of an instruction, including prefixes\
pub const MAX_INSTRUCTION_BYTE_COUNT: u8 = 15;
