/// AMD64 register enumeration
///
/// This enum represents the general-purpose registers in AMD64 architecture
/// as register numbers (0-15). These numbers are used in ModR/M and REX byte encoding.
///
/// Register numbers 0-7 are the original x86 registers (RAX-RDI).
/// Register numbers 8-15 are the extended registers (R8-R15) that require REX prefix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Register {
    // 64-bit General Purpose Registers (register numbers 0-7)
    /// RAX - Accumulator register (register number 0)
    RAX = 0,

    /// RCX - Counter register (register number 1)
    RCX = 1,

    /// RDX - Data register (register number 2)
    RDX = 2,

    /// RBX - Base register (register number 3)
    RBX = 3,

    /// RSP - Stack pointer register (register number 4)
    RSP = 4,

    /// RBP - Base pointer register (register number 5)
    RBP = 5,

    /// RSI - Source index register (register number 6)
    RSI = 6,

    /// RDI - Destination index register (register number 7)
    RDI = 7,

    // Extended 64-bit Registers (register numbers 8-15, require REX prefix)
    /// R8 - Extended register 8 (register number 8)
    R8 = 8,

    /// R9 - Extended register 9 (register number 9)
    R9 = 9,

    /// R10 - Extended register 10 (register number 10)
    R10 = 10,

    /// R11 - Extended register 11 (register number 11)
    R11 = 11,

    /// R12 - Extended register 12 (register number 12)
    R12 = 12,

    /// R13 - Extended register 13 (register number 13)
    R13 = 13,

    /// R14 - Extended register 14 (register number 14)
    R14 = 14,

    /// R15 - Extended register 15 (register number 15)
    R15 = 15,
}

impl Register {
    /// Returns the register number (0-15) for use in ModR/M or REX encoding
    ///
    /// # Examples
    ///
    /// ```
    /// use foolang::platforms::amd64::Register;
    ///
    /// assert_eq!(Register::RAX.number(), 0);
    /// assert_eq!(Register::RBX.number(), 3);
    /// assert_eq!(Register::R8.number(), 8);
    /// ```
    pub fn number(self) -> u8 {
        self as u8
    }

    /// Returns the register encoding as u8 (same as number())
    ///
    /// # Examples
    ///
    /// ```
    /// use foolang::platforms::amd64::Register;
    ///
    /// assert_eq!(Register::RAX.as_u8(), 0);
    /// assert_eq!(Register::RBX.as_u8(), 3);
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
    /// assert_eq!(Register::RAX.as_i32(), 0);
    /// assert_eq!(Register::R15.as_i32(), 15);
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

    /// Returns true if the register requires REX prefix (R8-R15, register numbers 8-15)
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
        (self as u8) >= 8
    }
}

/// Constructs a ModR/M byte for register-to-register operations
///
/// ModR/M byte format: [Mod(2) | Reg(3) | R/M(3)]
/// - Mod = 11 for register-direct mode
/// - Reg = source/destination register (bits 3-5)
/// - R/M = source/destination register (bits 0-2)
///
/// # Examples
///
/// ```
/// use foolang::platforms::amd64::{Register, modrm_reg_reg};
///
/// // MOV RAX, RBX (destination RAX, source RBX)
/// assert_eq!(modrm_reg_reg(Register::RAX, Register::RBX), 0xC3);
/// // 0xC3 = 11 000 011 = Mod=11, Reg=0(RAX), R/M=3(RBX)
/// ```
pub fn modrm_reg_reg(reg: Register, rm: Register) -> u8 {
    0xC0 | ((reg.number() & 0x7) << 3) | (rm.number() & 0x7)
}

/// Constructs a ModR/M byte with /digit extension for operations like MUL, DIV, etc.
///
/// ModR/M byte format: [Mod(2) | Digit(3) | R/M(3)]
/// - Mod = 11 for register-direct mode
/// - Digit = opcode extension (bits 3-5)
/// - R/M = register operand (bits 0-2)
///
/// # Examples
///
/// ```
/// use foolang::platforms::amd64::{Register, modrm_digit_reg};
///
/// // MUL RBX (opcode F7 /4)
/// assert_eq!(modrm_digit_reg(4, Register::RBX), 0xE3);
/// // 0xE3 = 11 100 011 = Mod=11, Digit=4, R/M=3(RBX)
/// ```
pub fn modrm_digit_reg(digit: u8, rm: Register) -> u8 {
    0xC0 | ((digit & 0x7) << 3) | (rm.number() & 0x7)
}
