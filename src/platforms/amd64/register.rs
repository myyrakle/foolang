/// AMD64 register enumeration with ModR/M byte encoding
///
/// This enum represents the general-purpose registers in AMD64 architecture
/// and their corresponding ModR/M byte values for register-to-register operations.
/// The encoding uses Mod=11 (register-direct mode) with the register in the R/M field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Register {
    // 64-bit General Purpose Registers (RAX through RDI)
    /// RAX - Accumulator register
    /// ModR/M byte: 0xC0 (11 000 000)
    RAX = 0xC0,

    /// RCX - Counter register
    /// ModR/M byte: 0xC1 (11 000 001)
    RCX = 0xC1,

    /// RDX - Data register
    /// ModR/M byte: 0xC2 (11 000 010)
    RDX = 0xC2,

    /// RBX - Base register
    /// ModR/M byte: 0xC3 (11 000 011)
    RBX = 0xC3,

    /// RSP - Stack pointer register
    /// ModR/M byte: 0xC4 (11 000 100)
    RSP = 0xC4,

    /// RBP - Base pointer register
    /// ModR/M byte: 0xC5 (11 000 101)
    RBP = 0xC5,

    /// RSI - Source index register
    /// ModR/M byte: 0xC6 (11 000 110)
    RSI = 0xC6,

    /// RDI - Destination index register
    /// ModR/M byte: 0xC7 (11 000 111)
    RDI = 0xC7,

    // Extended 64-bit Registers (R8 through R15)
    /// R8 - Extended register 8
    /// ModR/M byte: 0xC8 (11 001 000) - requires REX prefix
    R8 = 0xC8,

    /// R9 - Extended register 9
    /// ModR/M byte: 0xC9 (11 001 001) - requires REX prefix
    R9 = 0xC9,

    /// R10 - Extended register 10
    /// ModR/M byte: 0xCA (11 001 010) - requires REX prefix
    R10 = 0xCA,

    /// R11 - Extended register 11
    /// ModR/M byte: 0xCB (11 001 011) - requires REX prefix
    R11 = 0xCB,

    /// R12 - Extended register 12
    /// ModR/M byte: 0xCC (11 001 100) - requires REX prefix
    R12 = 0xCC,

    /// R13 - Extended register 13
    /// ModR/M byte: 0xCD (11 001 101) - requires REX prefix
    R13 = 0xCD,

    /// R14 - Extended register 14
    /// ModR/M byte: 0xCE (11 001 110) - requires REX prefix
    R14 = 0xCE,

    /// R15 - Extended register 15
    /// ModR/M byte: 0xCF (11 001 111) - requires REX prefix
    R15 = 0xCF,
}

impl Register {
    /// Returns the register encoding as u8
    ///
    /// # Examples
    ///
    /// ```
    /// use foolang::platforms::amd64::Register;
    ///
    /// assert_eq!(Register::RAX.as_u8(), 0xC0);
    /// assert_eq!(Register::RBX.as_u8(), 0xC3);
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
    /// assert_eq!(Register::RAX.as_i32(), 0xC0);
    /// assert_eq!(Register::R15.as_i32(), 0xCF);
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
        (self as u8) >= 0xC8
    }
}
