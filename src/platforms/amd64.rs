/// AMD64 register enumeration with binary encoding codes
/// 
/// This enum represents the general-purpose registers in AMD64 architecture
/// and their corresponding binary encoding values used in instruction encoding.
/// The encoding values correspond to the register field in ModR/M byte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Register {
    // 64-bit General Purpose Registers (RAX through RDI)
    /// RAX - Accumulator register
    /// Encoding: 0x0
    RAX = 0x0,
    
    /// RCX - Counter register
    /// Encoding: 0x1
    RCX = 0x1,
    
    /// RDX - Data register
    /// Encoding: 0x2
    RDX = 0x2,
    
    /// RBX - Base register
    /// Encoding: 0x3
    RBX = 0x3,
    
    /// RSP - Stack pointer register
    /// Encoding: 0x4
    RSP = 0x4,
    
    /// RBP - Base pointer register
    /// Encoding: 0x5
    RBP = 0x5,
    
    /// RSI - Source index register
    /// Encoding: 0x6
    RSI = 0x6,
    
    /// RDI - Destination index register
    /// Encoding: 0x7
    RDI = 0x7,
    
    // Extended 64-bit Registers (R8 through R15)
    /// R8 - Extended register 8
    /// Encoding: 0x8 (requires REX prefix)
    R8 = 0x8,
    
    /// R9 - Extended register 9
    /// Encoding: 0x9 (requires REX prefix)
    R9 = 0x9,
    
    /// R10 - Extended register 10
    /// Encoding: 0xA (requires REX prefix)
    R10 = 0xA,
    
    /// R11 - Extended register 11
    /// Encoding: 0xB (requires REX prefix)
    R11 = 0xB,
    
    /// R12 - Extended register 12
    /// Encoding: 0xC (requires REX prefix)
    R12 = 0xC,
    
    /// R13 - Extended register 13
    /// Encoding: 0xD (requires REX prefix)
    R13 = 0xD,
    
    /// R14 - Extended register 14
    /// Encoding: 0xE (requires REX prefix)
    R14 = 0xE,
    
    /// R15 - Extended register 15
    /// Encoding: 0xF (requires REX prefix)
    R15 = 0xF,
}

impl Register {
    /// Returns the register encoding as u8
    /// 
    /// # Examples
    /// 
    /// ```
    /// use foolang::platforms::amd64::Register;
    /// 
    /// assert_eq!(Register::RAX.as_u8(), 0x0);
    /// assert_eq!(Register::RBX.as_u8(), 0x3);
    /// ```
    pub fn as_u8(self) -> u8 {
        self as u8
    }
    
    /// Returns the register encoding as i32
    /// 
    /// # Examples
    /// 
    /// ```
    /// use foolang::platforms::amd64::Register;
    /// 
    /// assert_eq!(Register::RAX.as_i32(), 0x0);
    /// assert_eq!(Register::R15.as_i32(), 0xF);
    /// ```
    pub fn as_i32(self) -> i32 {
        self as i32
    }
    
    /// Returns the register name as a string
    /// 
    /// # Examples
    /// 
    /// ```
    /// use foolang::platforms::amd64::Register;
    /// 
    /// assert_eq!(Register::RAX.name(), "RAX");
    /// assert_eq!(Register::R8.name(), "R8");
    /// ```
    pub fn name(self) -> &'static str {
        match self {
            Register::RAX => "RAX",
            Register::RCX => "RCX",
            Register::RDX => "RDX",
            Register::RBX => "RBX",
            Register::RSP => "RSP",
            Register::RBP => "RBP",
            Register::RSI => "RSI",
            Register::RDI => "RDI",
            Register::R8 => "R8",
            Register::R9 => "R9",
            Register::R10 => "R10",
            Register::R11 => "R11",
            Register::R12 => "R12",
            Register::R13 => "R13",
            Register::R14 => "R14",
            Register::R15 => "R15",
        }
    }
    
    /// Returns true if the register requires REX prefix (R8-R15)
    /// 
    /// # Examples
    /// 
    /// ```
    /// use foolang::platforms::amd64::Register;
    /// 
    /// assert_eq!(Register::RAX.requires_rex(), false);
    /// assert_eq!(Register::R8.requires_rex(), true);
    /// ```
    pub fn requires_rex(self) -> bool {
        (self as u8) >= 0x8
    }
}

/// AMD64 instruction set enumeration with binary hex codes
/// 
/// This enum represents various AMD64 (x86-64) instructions and their corresponding
/// binary opcodes. The opcodes are represented as i32 values for easy manipulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum Instruction {
    // Data Movement Instructions
    /// MOV - Move data between registers/memory
    /// Opcode: 0x89 (MOV r/m32, r32)
    Mov = 0x89,
    
    /// MOVQ - Move quadword
    /// Opcode: 0x48 (REX.W prefix for 64-bit operands)
    MovQ = 0x48,
    
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
    /// Opcode: 0xF7 (with /4 extension)
    Mul = 0xF704,
    
    /// IMUL - Signed multiply
    /// Opcode: 0xAF (0F AF - two byte opcode)
    IMul = 0x0FAF,
    
    /// DIV - Unsigned divide
    /// Opcode: 0xF7 (with /6 extension)
    Div = 0xF706,
    
    /// IDIV - Signed divide
    /// Opcode: 0xF7 (with /7 extension)
    IDiv = 0xF707,
    
    /// INC - Increment
    /// Opcode: 0xFF (with /0 extension)
    Inc = 0xFF00,
    
    /// DEC - Decrement
    /// Opcode: 0xFF (with /1 extension)
    Dec = 0xFF01,
    
    /// NEG - Negate
    /// Opcode: 0xF7 (with /3 extension)
    Neg = 0xF703,
    
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
    /// Opcode: 0xF7 (with /2 extension)
    Not = 0xF702,
    
    /// SHL - Shift left
    /// Opcode: 0xD3 (with /4 extension)
    Shl = 0xD304,
    
    /// SHR - Shift right
    /// Opcode: 0xD3 (with /5 extension)
    Shr = 0xD305,
    
    /// SAR - Arithmetic shift right
    /// Opcode: 0xD3 (with /7 extension)
    Sar = 0xD307,
    
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
    
    /// Returns the instruction opcode as i32
    pub fn as_i32(self) -> i32 {
        self as i32
    }
    
    /// Returns the instruction name as a string
    pub fn name(self) -> &'static str {
        match self {
            Instruction::Mov => "MOV",
            Instruction::MovQ => "MOVQ",
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

#[cfg(test)]
mod tests {
    use super::*;

    // Register tests
    #[test]
    fn test_register_as_u8() {
        assert_eq!(Register::RAX.as_u8(), 0x0);
        assert_eq!(Register::RCX.as_u8(), 0x1);
        assert_eq!(Register::RDX.as_u8(), 0x2);
        assert_eq!(Register::RBX.as_u8(), 0x3);
        assert_eq!(Register::RSP.as_u8(), 0x4);
        assert_eq!(Register::RBP.as_u8(), 0x5);
        assert_eq!(Register::RSI.as_u8(), 0x6);
        assert_eq!(Register::RDI.as_u8(), 0x7);
        assert_eq!(Register::R8.as_u8(), 0x8);
        assert_eq!(Register::R9.as_u8(), 0x9);
        assert_eq!(Register::R10.as_u8(), 0xA);
        assert_eq!(Register::R11.as_u8(), 0xB);
        assert_eq!(Register::R12.as_u8(), 0xC);
        assert_eq!(Register::R13.as_u8(), 0xD);
        assert_eq!(Register::R14.as_u8(), 0xE);
        assert_eq!(Register::R15.as_u8(), 0xF);
    }

    #[test]
    fn test_register_as_i32() {
        assert_eq!(Register::RAX.as_i32(), 0x0);
        assert_eq!(Register::RBX.as_i32(), 0x3);
        assert_eq!(Register::R8.as_i32(), 0x8);
        assert_eq!(Register::R15.as_i32(), 0xF);
    }

    #[test]
    fn test_register_name() {
        assert_eq!(Register::RAX.name(), "RAX");
        assert_eq!(Register::RCX.name(), "RCX");
        assert_eq!(Register::RDX.name(), "RDX");
        assert_eq!(Register::RBX.name(), "RBX");
        assert_eq!(Register::RSP.name(), "RSP");
        assert_eq!(Register::RBP.name(), "RBP");
        assert_eq!(Register::RSI.name(), "RSI");
        assert_eq!(Register::RDI.name(), "RDI");
        assert_eq!(Register::R8.name(), "R8");
        assert_eq!(Register::R9.name(), "R9");
        assert_eq!(Register::R10.name(), "R10");
        assert_eq!(Register::R11.name(), "R11");
        assert_eq!(Register::R12.name(), "R12");
        assert_eq!(Register::R13.name(), "R13");
        assert_eq!(Register::R14.name(), "R14");
        assert_eq!(Register::R15.name(), "R15");
    }

    #[test]
    fn test_register_requires_rex() {
        // Registers RAX-RDI don't require REX for basic access
        assert_eq!(Register::RAX.requires_rex(), false);
        assert_eq!(Register::RCX.requires_rex(), false);
        assert_eq!(Register::RDX.requires_rex(), false);
        assert_eq!(Register::RBX.requires_rex(), false);
        assert_eq!(Register::RSP.requires_rex(), false);
        assert_eq!(Register::RBP.requires_rex(), false);
        assert_eq!(Register::RSI.requires_rex(), false);
        assert_eq!(Register::RDI.requires_rex(), false);
        
        // Extended registers R8-R15 require REX prefix
        assert_eq!(Register::R8.requires_rex(), true);
        assert_eq!(Register::R9.requires_rex(), true);
        assert_eq!(Register::R10.requires_rex(), true);
        assert_eq!(Register::R11.requires_rex(), true);
        assert_eq!(Register::R12.requires_rex(), true);
        assert_eq!(Register::R13.requires_rex(), true);
        assert_eq!(Register::R14.requires_rex(), true);
        assert_eq!(Register::R15.requires_rex(), true);
    }

    #[test]
    fn test_register_equality() {
        assert_eq!(Register::RAX, Register::RAX);
        assert_ne!(Register::RAX, Register::RBX);
        assert_eq!(Register::R15, Register::R15);
        assert_ne!(Register::R8, Register::R9);
    }

    // Instruction tests
    #[test]
    fn test_instruction_as_i32() {
        assert_eq!(Instruction::Add.as_i32(), 0x01);
        assert_eq!(Instruction::Mov.as_i32(), 0x89);
        assert_eq!(Instruction::Ret.as_i32(), 0xC3);
    }

    #[test]
    fn test_instruction_as_bytes() {
        assert_eq!(Instruction::Add.as_bytes(), vec![0x01]);
        assert_eq!(Instruction::Mov.as_bytes(), vec![0x89]);
        assert_eq!(Instruction::Syscall.as_bytes(), vec![0x0F, 0x05]);
        assert_eq!(Instruction::IMul.as_bytes(), vec![0x0F, 0xAF]);
    }

    #[test]
    fn test_instruction_name() {
        assert_eq!(Instruction::Add.name(), "ADD");
        assert_eq!(Instruction::Mov.name(), "MOV");
        assert_eq!(Instruction::Syscall.name(), "SYSCALL");
    }

    #[test]
    fn test_instruction_equality() {
        assert_eq!(Instruction::Add, Instruction::Add);
        assert_ne!(Instruction::Add, Instruction::Sub);
    }
}
