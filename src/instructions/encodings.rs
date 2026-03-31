use bitflags::bitflags;
use std::fmt::{Display, Formatter, Result};

bitflags! {
    #[derive(Debug, PartialEq, Copy, Clone)]
    pub struct CpuFlags: u16 {
        /// Carry Flag (CF) - this flag is set to 1 when there is an unsigned overflow.
        /// For example when you add bytes 255 + 1 (result is not in range 0...255).
        /// When there is no overflow this flag is set to 0.
        const CF = 1 << 0;

        /// Parity Flag (PF) - this flag is set to 1 when there is even number of one bits in result,
        /// and to 0 when there is odd number of one bits.
        /// Even if result is a word only 8 low bits are analyzed!
        const PF = 1 << 2;

        /// Auxiliary Flag (AF) - set to 1 when there is an unsigned overflow for low nibble (4 bits).
        const AF = 1 << 4;

        /// Zero Flag (ZF) - set to 1 when result is zero. For none zero result this flag is set to 0.
        const ZF = 1 << 6;

        /// Sign Flag (SF) - set to 1 when result is negative. When result is positive it is set to 0.
        /// Actually this flag take the value of the most significant bit.
        const SF = 1 << 7;

        /// Overflow Flag (OF) - set to 1 when there is a signed overflow. For example,
        /// when you add bytes 100 + 50 (result is not in range -128...127).
        const OF = 1 << 11;
    }
}

/// Represents all possible instructions supported by the simulator
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OperationType {
    None,
    Mov,
    Add,
    Sub,
    Cmp,

    /// In 8086 stack Push and Pop only supports 16 bits
    /// Subtract 2 from SP register. (Stack grows downward in the memory)
    /// Write the value of source to the address SS:SP.
    Push,

    /// In 8086 stack Push and Pop only supports 16 bits
    /// Write the value at the address SS:SP to destination.
    /// Add 2 to SP register.
    Pop,

    /// Exchange values of two operands.
    Xchg,

    /// Transfers from I/O device to accumulator (AX or AL).
    ///
    /// There are two different forms of IN and OUT instructions: the direct I/O instructions and
    /// variable I/O instructions. Either of these two types of instructions can be used to transfer a byte
    /// or a word of data. All data transfers take place between an I/O device and the MPU’s accumulator register.
    ///
    /// Second operand is a port number. If required to access port number over 255 - DX register should be used.
    ///
    /// The port number identifies the external I/O device.
    In,

    /// Transfers from accumulator (AX or AL) to I/O device.
    ///
    /// There are two different forms of IN and OUT instructions: the direct I/O instructions and
    /// variable I/O instructions. Either of these two types of instructions can be used to transfer a byte
    /// or a word of data. All data transfers take place between an I/O device and the MPU’s accumulator register.
    ///
    /// Second operand is a port number. If required to access port number over 255 - DX register should be used.
    ///
    /// The port number identifies the external I/O device.
    Out,

    /// Translates to something like `mov al,[ds:bx + al]`, Note that the al in the
    /// address calculation is invalid, However Xlat can use it.
    //
    /// Copy value of memory byte at DS:[BX + unsigned AL] to AL register.
    ///
    /// In common applications you will hardly find any usage for xlat instruction.
    /// It's archaism from 8086 times, compiler will certainly not use it, and most
    /// of hand written assembly neither, as usually you can use simple mov al,[bx+si]
    /// or something similar.
    ///
    /// See: https://stackoverflow.com/a/47560869/4086981
    Xlat,

    /// Computes the effective address of the second operand (the source operand) and
    /// stores it in the first operand (destination operand). The source operand is a
    /// memory address (offset part) specified with one of the processors addressing modes;
    /// the destination operand is a general-purpose register. The address-size and operand-size
    /// attributes affect the action performed by this instruction, as shown in the following table.
    /// The operand-size attribute of the instruction is determined by the chosen register;
    /// the address-size attribute is determined by the attribute of the code segment.
    ///
    /// Seconds operand must be a memory address speciified with an addressing mode: eg [rbx+2*rs1].
    Lea,

    /// Loads a far pointer (segment selector and offset) (segment:offset) from the second operand (source operand)
    /// into a segment register and the first operand (destination operand). The source operand
    /// specifies a 48-bit or a 32-bit pointer in memory depending on the current setting of the
    /// operand-size attribute (32 bits or 16 bits, respectively). The instruction opcode and the
    /// destination operand specify a segment register/general-purpose register pair.
    /// The 16-bit segment selector from the source operand is loaded into the segment register
    /// specified with the opcode (DS, SS, ES, FS, or GS). The 32-bit or 16-bit offset is loaded
    /// into the register specified with the destination operand.
    ///
    /// This is ussed to get the address of some memory data that is outside of the current segment.
    ///
    /// See: https://www.felixcloutier.com/x86/lds:les:lfs:lgs:lss
    Lds,

    /// See: OperationType::Lds docs
    /// See: https://www.felixcloutier.com/x86/lds:les:lfs:lgs:lss
    Les,

    /// The LAHF (Load AH from Flags) instruction in 8086 assembly language copies
    /// the lower 8 bits of the 16-bit flag register into the AH register.
    ///
    /// It acts as a data transfer tool to move flag statuses—specifically SF, ZF, AF, PF, and CF into AH
    /// for testing or saving without affecting the flags themselves.
    Lahf,

    /// Store AH into the lower 8 bits of the 16-bit flag register.
    Sahf,

    /// Store flags register in the stack.
    ///
    /// Algorithm:
    /// 1. SP = SP - 2
    /// 2. SS:[SP] (top of the stack) = flags
    ///
    /// Decrement stack pointer by 2 (bytes) and store there the flags register.
    Pushf,

    /// Add with Carry.
    ///
    /// Algorithm:
    /// operand1 = operand1 + operand2 + CF
    ///
    /// Example:
    /// STC        ; set CF = 1
    /// MOV AL, 5  ; AL = 5
    /// ADC AL, 1  ; AL = 7
    /// RET
    ///
    /// See example usage of this instruction: https://stackoverflow.com/q/44540078/4086981
    Adc,

    /// Increment.
    ///
    /// Algorithm:
    /// operand = operand + 1
    ///
    /// Example:
    /// MOV AL, 4
    /// INC AL       ; AL = 5
    /// RET
    ///
    /// This insntruction dont affect the CF (Carry Flag),
    /// check this to see the reasons: https://stackoverflow.com/a/13435633/4086981
    Inc,

    /// ASCII Adjust after Addition.
    /// Corrects result in AH and AL after addition when working with unpacked BCD values.
    /// It works according to the following Algorithm:
    /// if low nibble of AL > 9 or AF = 1 then:
    ///     AL = AL + 6
    ///     AH = AH + 1
    ///     AF = 1
    ///     CF = 1
    /// else
    ///     AF = 0
    ///     CF = 0
    /// in both cases:
    /// clear the high nibble of AL.
    ///
    /// Example:
    /// MOV AX, 15   ; AH = 00, AL = 0Fh
    /// AAA          ; AH = 01, AL = 05
    /// RET
    /// See: https://stackoverflow.com/q/18945247/4086981
    ///
    /// Just affects AF and CF flags.
    Aaa,

    /// Decimal adjust After Addition.
    /// Corrects the result of addition of two packed BCD values.
    ///
    /// Algorithm:
    /// if low nibble of AL > 9 or AF = 1 then:
    /// AL = AL + 6
    /// AF = 1
    /// if AL > 9Fh or CF = 1 then:
    /// AL = AL + 60h
    /// CF = 1
    ///
    /// Example:
    /// MOV AL, 0Fh  ; AL = 0Fh (15)
    /// DAA          ; AL = 15h
    /// RET
    Daa,

    /// Subtract with Borrow.
    ///
    /// Algorithm:
    /// operand1 = operand1 - operand2 - CF
    ///
    /// Example:
    /// STC
    /// MOV AL, 5
    /// SBB AL, 3    ; AL = 5 - 3 - 1 = 1
    /// RET
    Sbb,

    /// Get flags register from the stack.
    ///
    /// Algorithm:
    /// 1. flags = SS:[SP] (top of the stack)
    /// 2. SP = SP + 2
    ///
    /// Read item at the top of the stack and then increment stack pointer to remove the
    /// item from the stack. Remember in 8086 an other architectures stack grows downward and shrink upward.
    Popf,

    /// Decrement.
    ///
    /// Algorithm:
    /// operand = operand - 1
    ///
    /// Example:
    /// MOV AL, 255  ; AL = 0FFh (255 or -1)
    /// DEC AL       ; AL = 0FEh (254 or -2)
    /// RET
    ///
    /// CF flag is unchanged.
    Dec,

    /// Jump if Not Zero (Not Equal).
    Jnz,
    /// Jump if Zero (Equal).
    Je,
    /// Jump if Less (<).
    /// Jump if Not Greater or Equal (not >=).
    Jl,
    /// Jump if Less or Equal (<=).
    /// Jump if Not Greater (not >).
    Jle,
    /// Jump if below 0
    Jb,
    /// Jump if below 0 or equal to 0
    Jbe,
    /// Jump if Parity Even.
    Jp,
    /// Jump if Overflow.
    Jo,
    /// Jump if Sign.
    Js,
    /// Jump if Not Equal (<>).
    /// Jump if Not Zero.
    Jne,
    /// Jump if Greater or Equal (>=).
    /// Jump if Not Less (not <).
    Jnl,
    /// Jump if Greater (>).
    /// Jump if Not Less or Equal (not <=).
    Jnle,
    /// Jump if Above or Equal (>=).
    // Jump if Not Below (not <).
    // Jump if Not Carry.
    Jnb,
    /// Jump if Above (>).
    /// Jump if Not Below or Equal (not <=).
    Jnbe,
    /// Jump if Parity Odd (No Parity).
    Jnp,
    /// Jump if Not Overflow.
    Jno,
    /// Jump if Not Sign.
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
            Self::Push => write!(f, "push"),
            Self::Pop => write!(f, "pop"),
            Self::Xchg => write!(f, "xchg"),
            Self::In => write!(f, "in"),
            Self::Out => write!(f, "out"),
            Self::Xlat => write!(f, "xlat"),
            Self::Lea => write!(f, "lea"),
            Self::Lds => write!(f, "lds"),
            Self::Les => write!(f, "les"),
            Self::Lahf => write!(f, "lahf"),
            Self::Sahf => write!(f, "sahf"),
            Self::Pushf => write!(f, "pushf"),
            Self::Popf => write!(f, "popf"),
            Self::Adc => write!(f, "adc"),
            Self::Inc => write!(f, "inc"),
            Self::Aaa => write!(f, "aaa"),
            Self::Daa => write!(f, "daa"),
            Self::Sbb => write!(f, "sbb"),
            Self::Dec => write!(f, "dec"),
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

    /// Used as an implicit field in the instruction table to force a register
    /// w(idth) when using mod+rm fields. This is useful for the case when this register
    /// must not be the same width of the reg field.
    /// For example, without this, the `in al, dx` instruction would be decoded
    /// as `in al, dl` which is an invalid operation because second operand should always be dx for IN instruction.
    ModRmW,
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

    /// The CPU affected flags, when this operation is executed by the CPU.
    pub affected_cpu_flags: CpuFlags,
}
