/// REX prefix bytes for 64-bit mode
///
/// REX prefixes enable 64-bit operand size and access to extended registers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RexPrefix {
    /// REX (0x40) - Base REX prefix
    Rex = 0x40,

    /// REX.W (0x48) - 64-bit operand size
    RexW = 0x48,

    /// REX.R (0x44) - Extension of ModR/M reg field
    RexR = 0x44,

    /// REX.X (0x42) - Extension of SIB index field
    RexX = 0x42,

    /// REX.B (0x41) - Extension of ModR/M r/m field, SIB base, or opcode reg
    RexB = 0x41,
}

/// REX prefix 조합 상수
impl RexPrefix {
    /// REX.W + REX.B (0x49) - 64-bit operand with extended base register (R8-R15)
    pub const REX_WB: u8 = 0x49;

    /// REX.W + REX.R (0x4C) - 64-bit operand with extended reg field (R8-R15)
    pub const REX_WR: u8 = 0x4C;

    /// REX.W + REX.R + REX.B (0x4D) - 64-bit operand with both extended registers
    pub const REX_WRB: u8 = 0x4D;
}
