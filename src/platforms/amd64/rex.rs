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
