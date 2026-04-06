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

        /// Direction Flag (DF) - this flag is used by some instructions like MOVSB, MOBSW to process data chains,
        /// when this flag is set to 0 - the processing is done forward,
        /// when this flag is set to 1 the processing is done backward.
        const DF = 1 << 10;

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

    /// Negate. Makes operand negative (two's complement).
    ///
    /// Algorithm:
    /// Invert all bits of the operand
    /// Add 1 to inverted operand
    ///
    /// Example:
    /// MOV AL, 5   ; AL = 05h
    /// NEG AL      ; AL = 0FBh (-5)
    /// NEG AL      ; AL = 05h (5)
    /// RET
    Neg,

    /// ASCII Adjust after Subtraction.
    /// Corrects result in AH and AL after subtraction when working with BCD values.
    ///
    /// Algorithm:
    /// if low nibble of AL > 9 or AF = 1 then:
    ///     AL = AL - 6
    ///     AH = AH - 1
    ///     AF = 1
    ///     CF = 1
    /// else
    ///     AF = 0
    ///     CF = 0
    /// in both cases:
    /// clear the high nibble of AL.
    ///
    /// Example:
    /// MOV AX, 02FFh  ; AH = 02, AL = 0FFh
    /// AAS            ; AH = 01, AL = 09
    /// RET
    Aas,

    /// Decimal adjust After Subtraction.
    /// Corrects the result of subtraction of two packed BCD values.
    ///
    /// Algorithm:
    /// if low nibble of AL > 9 or AF = 1 then:
    ///     AL = AL - 6
    ///     AF = 1
    /// if AL > 9Fh or CF = 1 then:
    ///     AL = AL - 60h
    ///     CF = 1
    ///
    /// Example:
    /// MOV AL, 0FFh  ; AL = 0FFh (-1)
    /// DAS           ; AL = 99h, CF = 1
    /// RET
    Das,

    /// Unsigned multiply.
    ///
    /// Algorithm:
    /// when operand is a byte:
    /// AX = AL * operand.
    ///
    /// when operand is a word:
    /// (DX AX) = AX * operand.
    ///
    /// Example:
    /// MOV AL, 200   ; AL = 0C8h
    /// MOV BL, 4
    /// MUL BL        ; AX = 0320h (800)
    /// RET
    ///
    /// CF=OF=0 when high section of the result is zero.
    Mul,

    /// Signed multiply.
    ///
    /// Algorithm:
    /// when operand is a byte:
    /// AX = AL * operand.
    ///
    /// when operand is a word:
    /// (DX AX) = AX * operand.
    ///
    /// Example:
    /// MOV AL, -2
    /// MOV BL, -4
    /// IMUL BL      ; AX = 8
    /// RET
    Imul,

    /// ASCII Adjust after Multiplication.
    /// Corrects the result of multiplication of two BCD values.
    ///
    /// Algorithm:
    /// AH = AL / 10
    /// AL = remainder
    ///
    /// Example:
    /// MOV AL, 15   ; AL = 0Fh
    /// AAM          ; AH = 01, AL = 05
    /// RET
    ///
    /// Affects, ZF, SF, PF
    Aam,

    /// Unsigned divide.
    ///
    /// Algorithm:
    /// when operand is a byte:
    /// AL = AX / operand
    /// AH = remainder (modulus)
    ///
    /// when operand is a word:
    /// AX = (DX AX) / operand
    /// DX = remainder (modulus)
    ///
    /// Example:
    /// MOV AX, 203   ; AX = 00CBh
    /// MOV BL, 4
    /// DIV BL        ; AL = 50 (32h), AH = 3
    /// RET
    ///
    /// Don't affects any flag.
    ///
    /// Because the 8086 always looks at both registers during a word division,
    /// you must make sure DX is cleared (set to 0) before you run the DIV instruction
    /// if your number only fits in AX. If you forget to clear DX, the CPU will include whatever
    /// random garbage data was left in DX in the division, and you will get an incorrect result.
    Div,

    /// Unsigned divide.
    ///
    /// Algorithm:
    /// when operand is a byte:
    /// AL = AX / operand
    /// AH = remainder (modulus)
    ///
    /// when operand is a word:
    /// AX = (DX AX) / operand
    /// DX = remainder (modulus)
    ///
    /// Example:
    /// MOV AX, -203 ; AX = 0FF35h
    /// MOV BL, 4
    /// IDIV BL      ; AL = -50 (0CEh), AH = -3 (0FDh)
    /// RET
    ///
    /// Don't affects any flag.
    ///
    /// Because the 8086 always looks at both registers during a word division,
    /// you must make sure DX is cleared (set to 0) before you run the DIV instruction
    /// if your number only fits in AX. If you forget to clear DX, the CPU will include whatever
    /// random garbage data was left in DX in the division, and you will get an incorrect result.
    Idiv,

    /// ASCII Adjust before Division.
    /// Prepares two BCD values for division.
    ///
    /// Algorithm:
    /// AL = (AH * 10) + AL
    /// AH = 0
    ///
    /// Example:
    /// MOV AX, 0105h   ; AH = 01, AL = 05
    /// AAD             ; AH = 00, AL = 0Fh (15)
    /// RET
    ///
    /// Affects ZF, SF, PF
    Aad,

    /// Convert byte into word.
    ///
    /// Algorithm:
    ///
    /// if high bit of AL = 1 then:
    ///     AH = 255 (0FFh)
    /// else
    ///     AH = 0
    ///
    /// Example:
    /// MOV AX, 0   ; AH = 0, AL = 0
    /// MOV AL, -5  ; AX = 000FBh (251)
    /// CBW         ; AX = 0FFFBh (-5)
    /// RET
    ///
    /// Don't affect flags.
    ///
    /// This is like a sign extension using AH AL registers.
    Cbw,

    /// Convert Word to Double word.
    ///
    /// Algorithm:
    /// if high bit of AX = 1 then:
    ///     DX = 65535 (0FFFFh)
    /// else
    ///     DX = 0
    ///
    /// Example:
    /// MOV DX, 0   ; DX = 0
    /// MOV AX, 0   ; AX = 0
    /// MOV AX, -5  ; DX AX = 00000h:0FFFBh
    /// CWD         ; DX AX = 0FFFFh:0FFFBh
    /// RET
    ///
    /// Don't affect flags.
    ///
    /// This is like a sign extension using DX AX registers.
    Cwd,

    /// Invert each bit of the operand.
    ///
    /// Algorithm:
    /// if bit is 1 turn it to 0.
    /// if bit is 0 turn it to 1.
    ///
    /// Example:
    /// MOV AL, 00011011b
    /// NOT AL   ; AL = 11100100b
    /// RET
    ///
    /// Don't affect flags.
    Not,

    /// Shift operand1 Left. The number of shifts is set by operand2.
    ///
    /// Algorithm:
    /// Shift all bits left, the bit that goes off is set to CF.
    /// Zero bit is inserted to the right-most position.
    ///
    /// Example:
    /// MOV AL, 11100000b
    /// SHL AL, 1         ; AL = 11000000b,  CF=1.
    /// RET
    ///
    /// Affects CF, OF.
    ///
    /// OF=0 if first operand keeps original sign.
    Shl,

    /// Shift operand1 Right. The number of shifts is set by operand2.
    ///
    /// Algorithm:
    /// Shift all bits right, the bit that goes off is set to CF.
    /// Zero bit is inserted to the left-most position.
    ///
    /// Example:
    /// MOV AL, 00000111b
    /// SHR AL, 1         ; AL = 00000011b,  CF=1.
    /// RET
    ///
    /// Affects CF, OF.
    ///
    /// OF=0 if first operand keeps original sign.
    Shr,

    /// Shift Arithmetic operand1 Right. The number of shifts is set by operand2.
    ///
    /// Algorithm:
    ///
    /// Shift all bits right, the bit that goes off is set to CF.
    /// The sign bit that is inserted to the left-most position has the same value as before shift.
    ///
    /// Example:
    /// MOV AL, 0E0h      ; AL = 11100000b
    /// SAR AL, 1         ; AL = 11110000b,  CF=0.
    ///
    /// MOV BL, 4Ch       ; BL = 01001100b
    /// SAR BL, 1         ; BL = 00100110b,  CF=0.
    /// RET
    ///
    /// Affects CF, OF.
    ///
    /// OF=0 if first operand keeps original sign.
    Sar,

    /// Rotate operand1 left. The number of rotates is set by operand2.
    ///
    /// Algorithm:
    /// shift all bits left, the bit that goes off is set to CF and the same bit is inserted to the right-most position.
    ///
    /// Example:
    /// MOV AL, 1Ch       ; AL = 00011100b
    /// ROL AL, 1         ; AL = 00111000b,  CF=0.
    /// RET
    ///
    /// Affects CF, OF.
    ///
    /// OF=0 if first operand keeps original sign.
    Rol,

    /// Rotate operand1 right. The number of rotates is set by operand2.
    ///
    /// Algorithm:
    /// shift all bits right, the bit that goes off is set to CF and the same bit is inserted to the left-most position.
    ///
    /// Example:
    /// MOV AL, 1Ch       ; AL = 00011100b
    /// ROR AL, 1         ; AL = 00001110b,  CF=0.
    /// RET
    ///
    /// Affects CF, OF.
    ///
    /// OF=0 if first operand keeps original sign.
    Ror,

    /// Rotate operand1 left through Carry Flag. The number of rotates is set by operand2.
    /// When immediate is greater then 1, assembler generates several RCL xx, 1 instructions because 8086 has machine code only for this instruction (the same principle works for all other shift/rotate instructions).
    ///
    /// Algorithm:
    /// shift all bits left, the bit that goes off is set to CF and previous value of CF is inserted to the right-most position.
    ///
    /// Example:
    /// STC               ; set carry (CF=1).
    /// MOV AL, 1Ch       ; AL = 00011100b
    /// RCL AL, 1         ; AL = 00111001b,  CF=0.
    /// RET
    Rcl,

    /// Rotate operand1 right through Carry Flag. The number of rotates is set by operand2.
    ///
    /// Algorithm:
    /// shift all bits right, the bit that goes off is set to CF and previous value of CF is inserted to the left-most position.
    ///
    /// Example:
    /// STC               ; set carry (CF=1).
    /// MOV AL, 1Ch       ; AL = 00011100b
    /// RCR AL, 1         ; AL = 10001110b,  CF=0.
    /// RET
    Rcr,

    /// Bitwise AND.
    ///
    /// Logical AND between all bits of two operands. Result is stored in operand1.
    ///
    /// These rules apply:
    /// 1 AND 1 = 1
    /// 1 AND 0 = 0
    /// 0 AND 1 = 0
    /// 0 AND 0 = 0
    ///
    /// Example:
    /// MOV AL, 'a'        ; AL = 01100001b
    /// AND AL, 11011111b  ; AL = 01000001b  ('A')
    /// RET
    ///
    /// Affect CF, ZF, SF, OF, PF
    And,

    /// Logical AND between all bits of two operands for flags only.
    /// These flags are effected: ZF, SF, PF. Result is not stored anywhere.
    ///
    /// These rules apply:
    /// 1 AND 1 = 1
    /// 1 AND 0 = 0
    /// 0 AND 1 = 0
    /// 0 AND 0 = 0
    ///
    /// Example:
    /// MOV AL, 00000101b
    /// TEST AL, 1         ; ZF = 0.
    /// TEST AL, 10b       ; ZF = 1.
    /// RET
    Test,

    /// Bitwise or.
    ///
    /// Logical OR between all bits of two operands. Result is stored in first operand.
    ///
    /// These rules apply:
    ///
    /// 1 OR 1 = 1
    /// 1 OR 0 = 1
    /// 0 OR 1 = 1
    /// 0 OR 0 = 0
    ///
    /// Example:
    /// MOV AL, 'A'       ; AL = 01000001b
    /// OR AL, 00100000b  ; AL = 01100001b  ('a')
    /// RET
    Or,

    /// Bitwise XOR
    /// Logical XOR (Exclusive OR) between all bits of two operands. Result is stored in first operand.
    ///
    /// These rules apply:
    ///
    /// 1 XOR 1 = 0
    /// 1 XOR 0 = 1
    /// 0 XOR 1 = 1
    /// 0 XOR 0 = 0
    ///
    /// Example:
    /// MOV AL, 00000111b
    /// XOR AL, 00000010b    ; AL = 00000101b
    /// RET
    Xor,

    /// Repeat following MOVSB, MOVSW, LODSB, LODSW, STOSB, STOSW instructions CX times.
    ///
    /// Algorithm:
    ///
    /// check_cx:
    ///
    /// if CX <> 0 then
    /// do following chain instruction
    ///     CX = CX - 1
    ///     go back to check_cx
    ///     else
    /// exit from REP cycle
    Rep,

    /// Repeat following CMPSB, CMPSW, SCASB, SCASW instructions while ZF = 1 (result is Equal), maximum CX times.
    ///
    /// Algorithm:
    ///
    /// check_cx:
    ///
    /// if CX <> 0 then
    ///     do following chain instruction
    ///     CX = CX - 1
    ///     if ZF = 1 then:
    ///         go back to check_cx
    ///     else
    ///         exit from REPE cycle
    /// else
    ///     exit from REPE cycle
    Repe,

    /// Repeat following CMPSB, CMPSW, SCASB, SCASW instructions while ZF = 0 (result is Not Equal), maximum CX times.
    ///
    /// Algorithm:
    ///
    /// check_cx:
    ///
    /// if CX <> 0 then
    ///     do following chain instruction
    ///     CX = CX - 1
    ///     if ZF = 0 then:
    ///         go back to check_cx
    ///     else
    ///         exit from REPNE cycle
    /// else
    ///     exit from REPNE cycle
    Repne,

    /// Copy byte at DS:[SI] to ES:[DI]. Update SI and DI.
    ///
    /// Algorithm:
    ///
    /// ES:[DI] = DS:[SI]
    ///     if DF = 0 then
    ///     SI = SI + 1
    ///     DI = DI + 1
    /// else
    ///     SI = SI - 1
    ///     DI = DI - 1
    ///
    /// Example:
    ///
    /// ORG 100h
    ///
    /// CLD
    /// LEA SI, a1
    /// LEA DI, a2
    /// MOV CX, 5
    /// REP MOVSB
    ///
    /// RET
    ///
    /// a1 DB 1,2,3,4,5
    /// a2 DB 5 DUP(0)
    Movsb,

    /// Copy word at DS:[SI] to ES:[DI]. Update SI and DI.
    ///
    /// Algorithm:
    ///
    /// ES:[DI] = DS:[SI]
    /// if DF = 0 then
    ///     SI = SI + 2
    ///     DI = DI + 2
    /// else
    ///     SI = SI - 2
    ///     DI = DI - 2
    /// Example:
    ///
    /// ORG 100h
    /// CLD
    /// LEA SI, a1
    /// LEA DI, a2
    /// MOV CX, 5
    /// REP MOVSW
    /// RET
    ///
    /// a1 DW 1,2,3,4,5
    /// a2 DW 5 DUP(0)
    Movsw,

    /// Load byte at DS:[SI] into AL. Update SI.
    ///
    /// Algorithm:
    ///
    /// AL = DS:[SI]
    /// if DF = 0 then
    ///     SI = SI + 1
    /// else
    ///     SI = SI - 1
    ///
    /// Example:
    ///
    /// ORG 100h
    ///
    /// LEA SI, a1
    /// MOV CX, 5
    /// MOV AH, 0Eh
    ///
    /// m: LODSB
    /// INT 10h
    /// LOOP m
    ///
    /// RET
    ///
    /// a1 DB 'H', 'e', 'l', 'l', 'o'
    Lodsb,

    /// Load word at DS:[SI] into AX. Update SI.
    ///
    /// Algorithm:
    ///
    /// AX = DS:[SI]
    /// if DF = 0 then
    ///     SI = SI + 2
    /// else
    ///     SI = SI - 2
    ///
    /// Example:
    ///
    /// ORG 100h
    ///
    /// LEA SI, a1
    /// MOV CX, 5
    ///
    /// REP LODSW   ; finally there will be 555h in AX.
    ///
    /// RET
    ///
    /// a1 dw 111h, 222h, 333h, 444h, 555h
    Lodsw,

    /// Store byte in AL into ES:[DI]. Update DI.
    ///
    /// Algorithm:
    ///
    /// ES:[DI] = AL
    /// if DF = 0 then
    ///     DI = DI + 1
    /// else
    ///     DI = DI - 1
    ///
    /// Example:
    ///
    /// ORG 100h
    ///
    /// LEA DI, a1
    /// MOV AL, 12h
    /// MOV CX, 5
    ///
    /// REP STOSB
    ///
    /// RET
    ///
    /// a1 DB 5 dup(0)
    Stosb,

    /// Store word in AX into ES:[DI]. Update DI.
    ///
    /// Algorithm:
    ///
    /// ES:[DI] = AX
    /// if DF = 0 then
    ///     DI = DI + 2
    /// else
    ///     DI = DI - 2
    ///
    /// Example:
    ///
    /// ORG 100h
    ///
    /// LEA DI, a1
    /// MOV AX, 1234h
    /// MOV CX, 5
    ///
    /// REP STOSW
    ///
    /// RET
    ///
    /// a1 DW 5 dup(0)
    Stosw,

    /// Compare bytes: ES:[DI] from DS:[SI].
    ///
    /// Algorithm:
    ///
    /// DS:[SI] - ES:[DI]
    /// set flags according to result:
    /// OF, SF, ZF, AF, PF, CF
    /// if DF = 0 then
    ///     SI = SI + 1
    ///     DI = DI + 1
    /// else
    ///     SI = SI - 1
    ///     DI = DI - 1
    Cmpsb,

    /// Compare words: ES:[DI] from DS:[SI].
    ///
    /// Algorithm:
    ///
    /// DS:[SI] - ES:[DI]
    /// set flags according to result:
    /// OF, SF, ZF, AF, PF, CF
    /// if DF = 0 then
    ///     SI = SI + 2
    ///     DI = DI + 2
    /// else
    ///     SI = SI - 2
    ///     DI = DI - 2
    Cmpsw,

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

    /// Returns true if the operation is a prefix.
    pub fn is_prefix(&self) -> bool {
        match self {
            Self::Rep | Self::Repe | Self::Repne => true,
            _ => false,
        }
    }
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
            Self::Neg => write!(f, "neg"),
            Self::Aas => write!(f, "aas"),
            Self::Das => write!(f, "das"),
            Self::Mul => write!(f, "mul"),
            Self::Imul => write!(f, "imul"),
            Self::Aam => write!(f, "aam"),
            Self::Div => write!(f, "div"),
            Self::Idiv => write!(f, "idiv"),
            Self::Aad => write!(f, "aad"),
            Self::Cbw => write!(f, "cbw"),
            Self::Cwd => write!(f, "cwd"),
            Self::Not => write!(f, "not"),
            Self::Shl => write!(f, "shl"),
            Self::Shr => write!(f, "shr"),
            Self::Sar => write!(f, "sar"),
            Self::Rol => write!(f, "rol"),
            Self::Ror => write!(f, "ror"),
            Self::Rcl => write!(f, "rcl"),
            Self::Rcr => write!(f, "rcr"),
            Self::And => write!(f, "and"),
            Self::Test => write!(f, "test"),
            Self::Or => write!(f, "or"),
            Self::Xor => write!(f, "xor"),
            Self::Jnz => write!(f, "jnz"),
            Self::Rep => write!(f, "rep"),
            Self::Repe => write!(f, "repe"),
            Self::Repne => write!(f, "repne"),
            Self::Movsb => write!(f, "movsb"),
            Self::Movsw => write!(f, "movsw"),
            Self::Lodsb => write!(f, "lodsb"),
            Self::Lodsw => write!(f, "lodsw"),
            Self::Stosb => write!(f, "stosb"),
            Self::Stosw => write!(f, "stosw"),
            Self::Cmpsb => write!(f, "cmpsb"),
            Self::Cmpsw => write!(f, "cmpsw"),
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

    /// If D = 0; Instruction source is specified in REG field.
    /// If D = 1; Instruction destination is specified in REG field.
    D,

    /// If S = 0; No sign extension.
    /// If S = 1; Sign extend 8-bit immediate data to 16 bits.
    S,

    /// If W = 0; Instruction operates on byte data.
    /// If W = 1; Instruction operates on word data.
    W,

    /// If V = 0; Shift/rotate count is 1.
    /// If V = 1; Shift rotate count is specified in CL register.
    V,

    /// If Z = 0; Repeat/loop while zero flag is clear.
    /// If Z = 1; Repeat/loop while zero flag is set.
    Z,

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
