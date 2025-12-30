/// AMD64 instruction set enumeration with binary hex codes
///
/// This enum represents various AMD64 (x86-64) instructions and their corresponding
/// binary opcodes. The opcodes are represented as i32 values for easy manipulation.
///
/// Note: Some instructions require REX prefixes for 64-bit operation.
/// Use RexPrefix enum for prefix bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum Instruction {
    // Data Movement Instructions
    /// MOV - Move data between registers/memory
    /// Opcode: 0x89 (MOV r/m32, r32)
    Mov = 0x89,

    /// MOV - Move immediate to register/memory
    /// Opcode: 0xC7 (MOV r/m64, imm32)
    MovImm = 0xC7,

    /// PUSH - Push onto stack
    /// Opcode: 0x50 (PUSH r64)
    Push = 0x50,

    /// POP - Pop from stack
    /// Opcode: 0x58 (POP r64)
    Pop = 0x58,

    /// LEA - Load effective address
    /// Opcode: 0x8D
    Lea = 0x8D,

    // Arithmetic Instructions
    /// ADD - Add
    /// Opcode: 0x01 (ADD r/m32, r32)
    Add = 0x01,

    /// SUB - Subtract
    /// Opcode: 0x29 (SUB r/m32, r32)
    Sub = 0x29,

    /// MUL - Unsigned multiply
    /// Opcode: 0xF7 (requires ModR/M with /4 extension in reg field)
    /// Internal discriminant uses unique value to avoid enum collision
    Mul = 0x10F7,

    /// IMUL - Signed multiply (two-byte form)
    /// Opcode: 0x0F AF (two byte opcode)
    IMul = 0x0FAF,

    /// DIV - Unsigned divide
    /// Opcode: 0xF7 (requires ModR/M with /6 extension in reg field)
    /// Internal discriminant uses unique value to avoid enum collision
    Div = 0x20F7,

    /// IDIV - Signed divide
    /// Opcode: 0xF7 (requires ModR/M with /7 extension in reg field)
    /// Internal discriminant uses unique value to avoid enum collision
    IDiv = 0x30F7,

    /// INC - Increment
    /// Opcode: 0xFF (requires ModR/M with /0 extension in reg field)
    /// Internal discriminant uses unique value to avoid enum collision
    Inc = 0x10FF,

    /// DEC - Decrement
    /// Opcode: 0xFF (requires ModR/M with /1 extension in reg field)
    /// Internal discriminant uses unique value to avoid enum collision
    Dec = 0x20FF,

    /// NEG - Negate
    /// Opcode: 0xF7 (requires ModR/M with /3 extension in reg field)
    /// Internal discriminant uses unique value to avoid enum collision
    Neg = 0x40F7,

    // Logical Instructions
    /// AND - Logical AND
    /// Opcode: 0x21
    And = 0x21,

    /// OR - Logical OR
    /// Opcode: 0x09
    Or = 0x09,

    /// XOR - Logical XOR
    /// Opcode: 0x31
    Xor = 0x31,

    /// NOT - Logical NOT
    /// Opcode: 0xF7 (requires ModR/M with /2 extension in reg field)
    /// Internal discriminant uses unique value to avoid enum collision
    Not = 0x50F7,

    /// SHL - Shift left
    /// Opcode: 0xD3 (requires ModR/M with /4 extension in reg field)
    /// Internal discriminant uses unique value to avoid enum collision
    Shl = 0x10D3,

    /// SHR - Shift right
    /// Opcode: 0xD3 (requires ModR/M with /5 extension in reg field)
    /// Internal discriminant uses unique value to avoid enum collision
    Shr = 0x20D3,

    /// SAR - Arithmetic shift right
    /// Opcode: 0xD3 (requires ModR/M with /7 extension in reg field)
    /// Internal discriminant uses unique value to avoid enum collision
    Sar = 0x30D3,

    // Comparison and Test Instructions
    /// CMP - Compare
    /// Opcode: 0x39
    Cmp = 0x39,

    /// TEST - Test bits
    /// Opcode: 0x85
    Test = 0x85,

    // Control Flow Instructions
    /// JMP - Unconditional jump
    /// Opcode: 0xE9 (near jump)
    Jmp = 0xE9,

    /// JE/JZ - Jump if equal/zero
    /// Opcode: 0x74 (short jump)
    Je = 0x74,

    /// JNE/JNZ - Jump if not equal/not zero
    /// Opcode: 0x75 (short jump)
    Jne = 0x75,

    /// JG - Jump if greater
    /// Opcode: 0x7F (short jump)
    Jg = 0x7F,

    /// JL - Jump if less
    /// Opcode: 0x7C (short jump)
    Jl = 0x7C,

    /// JGE - Jump if greater or equal
    /// Opcode: 0x7D (short jump)
    Jge = 0x7D,

    /// JLE - Jump if less or equal
    /// Opcode: 0x7E (short jump)
    Jle = 0x7E,

    /// CALL - Call procedure
    /// Opcode: 0xE8 (near call)
    Call = 0xE8,

    /// RET - Return from procedure
    /// Opcode: 0xC3
    Ret = 0xC3,

    // Special Instructions
    /// NOP - No operation
    /// Opcode: 0x90
    Nop = 0x90,

    /// INT - Software interrupt
    /// Opcode: 0xCD
    Int = 0xCD,

    /// SYSCALL - Fast system call
    /// Opcode: 0x0F05 (two byte opcode)
    Syscall = 0x0F05,

    /// LEAVE - High level procedure exit
    /// Opcode: 0xC9
    Leave = 0xC9,

    /// ENTER - High level procedure entry
    /// Opcode: 0xC8
    Enter = 0xC8,
}

impl Instruction {
    /// Returns the instruction opcode as a byte array
    ///
    /// # Examples
    ///
    /// ```
    /// use foolang::platforms::amd64::Instruction;
    ///
    /// let add_opcode = Instruction::Add.as_bytes();
    /// assert_eq!(add_opcode, vec![0x01]);
    /// ```
    pub fn as_bytes(self) -> Vec<u8> {
        let value = self as i32;

        // For instructions with /digit extensions, extract the actual opcode
        // Internal discriminants use high bytes for uniqueness, but opcode is in low bytes
        match self {
            // 0xF7 opcode family (MUL, DIV, IDIV, NEG, NOT)
            Instruction::Mul | Instruction::Div | Instruction::IDiv
            | Instruction::Neg | Instruction::Not => vec![0xF7],

            // 0xFF opcode family (INC, DEC)
            Instruction::Inc | Instruction::Dec => vec![0xFF],

            // 0xD3 opcode family (SHL, SHR, SAR)
            Instruction::Shl | Instruction::Shr | Instruction::Sar => vec![0xD3],

            // All other instructions
            _ => {
                // Handle multi-byte opcodes
                if value > 0xFF {
                    // Two-byte opcode
                    if value <= 0xFFFF {
                        vec![(value >> 8) as u8, (value & 0xFF) as u8]
                    } else {
                        // Three or more bytes (rare, but handled)
                        let mut bytes = Vec::new();
                        let mut v = value;
                        while v > 0 {
                            bytes.insert(0, (v & 0xFF) as u8);
                            v >>= 8;
                        }
                        bytes
                    }
                } else {
                    // Single-byte opcode
                    vec![value as u8]
                }
            }
        }
    }

    /// Returns the instruction opcode as i32
    pub fn as_i32(self) -> i32 {
        self as i32
    }

    /// Returns the instruction name as a string
    pub fn name(self) -> &'static str {
        match self {
            Instruction::Mov => "MOV",
            Instruction::MovImm => "MOV",
            Instruction::Push => "PUSH",
            Instruction::Pop => "POP",
            Instruction::Lea => "LEA",
            Instruction::Add => "ADD",
            Instruction::Sub => "SUB",
            Instruction::Mul => "MUL",
            Instruction::IMul => "IMUL",
            Instruction::Div => "DIV",
            Instruction::IDiv => "IDIV",
            Instruction::Inc => "INC",
            Instruction::Dec => "DEC",
            Instruction::Neg => "NEG",
            Instruction::And => "AND",
            Instruction::Or => "OR",
            Instruction::Xor => "XOR",
            Instruction::Not => "NOT",
            Instruction::Shl => "SHL",
            Instruction::Shr => "SHR",
            Instruction::Sar => "SAR",
            Instruction::Cmp => "CMP",
            Instruction::Test => "TEST",
            Instruction::Jmp => "JMP",
            Instruction::Je => "JE",
            Instruction::Jne => "JNE",
            Instruction::Jg => "JG",
            Instruction::Jl => "JL",
            Instruction::Jge => "JGE",
            Instruction::Jle => "JLE",
            Instruction::Call => "CALL",
            Instruction::Ret => "RET",
            Instruction::Nop => "NOP",
            Instruction::Int => "INT",
            Instruction::Syscall => "SYSCALL",
            Instruction::Leave => "LEAVE",
            Instruction::Enter => "ENTER",
        }
    }
}

impl Instruction {
    /// Returns SYSCALL instruction bytes as a const array
    pub const SYSCALL_BYTES: [u8; 2] = [0x0F, 0x05];

    /// MOV immediate to 64-bit register base opcode (0xB8 + register number)
    /// Example: 0xB8 for RAX, 0xB9 for RCX, etc.
    pub const MOV_IMM64_BASE: u8 = 0xB8;

    /// Call instruction opcode (near relative call)
    pub const CALL_NEAR: u8 = 0xE8;

    /// Size of 32-bit immediate/displacement in bytes
    pub const DISPLACEMENT_32_SIZE: usize = 4;

    /// Addend for PC-relative calculations (call instruction next address)
    pub const CALL_ADDEND: i64 = -4;

    /// ModR/M mode bits for RIP-relative addressing
    /// mod=00, r/m=101
    pub const MODRM_RIP_RELATIVE_RM: u8 = 0x05;

    /// Bit shift amount for ModR/M reg field
    pub const MODRM_REG_SHIFT: u8 = 3;

    /// Bit mask for register number (lower 3 bits)
    pub const REG_NUMBER_MASK: u8 = 0x7;

    /// Returns the ModR/M reg field extension for instructions that require /digit encoding
    ///
    /// For instructions like MUL, DIV, INC, etc., the opcode alone is not sufficient.
    /// The ModR/M byte's reg field (bits 3-5) must contain a specific digit extension.
    ///
    /// Returns None for instructions that don't use /digit encoding.
    pub fn modrm_extension(self) -> Option<u8> {
        match self {
            Instruction::Inc => Some(0), // /0
            Instruction::Dec => Some(1), // /1
            Instruction::Not => Some(2), // /2
            Instruction::Neg => Some(3), // /3
            Instruction::Mul => Some(4), // /4
            Instruction::Shl => Some(4), // /4
            Instruction::Shr => Some(5), // /5
            Instruction::Div => Some(6), // /6
            Instruction::IDiv => Some(7), // /7
            Instruction::Sar => Some(7), // /7
            _ => None,
        }
    }
}
