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

// ModR/M byte 구성 상수
/// ModR/M의 Mod 필드: 11 = register-direct 모드 (레지스터 간 직접 연산)
const MODRM_MOD_REGISTER_DIRECT: u8 = 0b11;

/// ModR/M의 Reg 필드 시프트 비트 수 (3비트)
const MODRM_REG_FIELD_SHIFT: u8 = 3;

/// 레지스터 번호 마스크 (하위 3비트만 사용)
const REGISTER_NUMBER_MASK: u8 = 0x7;

/// ModR/M 바이트 생성: register-direct 모드의 베이스 값 (Mod=11, Reg=0, R/M=0)
const MODRM_REGISTER_DIRECT_BASE: u8 = MODRM_MOD_REGISTER_DIRECT << 6;

/// ModR/M 바이트를 생성합니다 (레지스터 간 연산용)
///
/// x86-64 명령어에서 두 레지스터 간 연산을 인코딩할 때 사용합니다.
///
/// ModR/M byte format: [Mod(2) | Reg(3) | R/M(3)]
/// - Mod = 11: register-direct 모드 (메모리 접근 없이 레지스터 간 직접 연산)
/// - Reg = 목적지/소스 레지스터 (bits 3-5)
/// - R/M = 소스/목적지 레지스터 (bits 0-2)
///
/// # Parameters
/// - `reg`: Reg 필드에 인코딩할 레지스터 (보통 destination)
/// - `rm`: R/M 필드에 인코딩할 레지스터 (보통 source)
///
/// # Examples
///
/// ```
/// use foolang::platforms::amd64::{Register, modrm_reg_reg};
///
/// // MOV RAX, RBX (RAX = destination, RBX = source)
/// assert_eq!(modrm_reg_reg(Register::RAX, Register::RBX), 0xC3);
/// // 0xC3 = 11 000 011 = Mod=11, Reg=0(RAX), R/M=3(RBX)
/// ```
pub fn modrm_reg_reg(reg: Register, rm: Register) -> u8 {
    MODRM_REGISTER_DIRECT_BASE
        | ((reg.number() & REGISTER_NUMBER_MASK) << MODRM_REG_FIELD_SHIFT)
        | (rm.number() & REGISTER_NUMBER_MASK)
}

/// ModR/M 바이트를 생성합니다 (opcode extension 방식)
///
/// 일부 x86-64 명령어(MUL, DIV, INC, DEC 등)는 opcode만으로 구분되지 않고,
/// ModR/M 바이트의 Reg 필드에 추가 opcode digit을 인코딩합니다.
///
/// ModR/M byte format: [Mod(2) | Digit(3) | R/M(3)]
/// - Mod = 11: register-direct 모드
/// - Digit = opcode extension (bits 3-5) - 명령어 구분용
/// - R/M = 피연산자 레지스터 (bits 0-2)
///
/// # Parameters
/// - `opcode_digit`: Reg 필드에 인코딩할 opcode extension (0-7)
/// - `rm`: R/M 필드에 인코딩할 피연산자 레지스터
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
pub fn modrm_digit_reg(opcode_digit: u8, rm: Register) -> u8 {
    MODRM_REGISTER_DIRECT_BASE
        | ((opcode_digit & REGISTER_NUMBER_MASK) << MODRM_REG_FIELD_SHIFT)
        | (rm.number() & REGISTER_NUMBER_MASK)
}

/// RSP 레지스터를 대상으로 하는 register-direct ModR/M 바이트 생성
///
/// SUB rsp, imm32 또는 ADD rsp, imm32 등의 명령어에서 사용됩니다.
///
/// # Parameters
/// - `opcode_digit`: Reg 필드에 인코딩할 opcode extension (0-7)
///
/// # Returns
/// ModR/M 바이트: [Mod=11 | Digit | R/M=100(RSP)]
///
/// # Examples
///
/// ```
/// use foolang::platforms::amd64::register::modrm_digit_rsp;
///
/// // SUB rsp, imm32 (opcode 81 /5)
/// assert_eq!(modrm_digit_rsp(5), 0xEC);
/// // 0xEC = 11 101 100 = Mod=11, Digit=5, R/M=4(RSP)
///
/// // ADD rsp, imm32 (opcode 81 /0)
/// assert_eq!(modrm_digit_rsp(0), 0xC4);
/// // 0xC4 = 11 000 100 = Mod=11, Digit=0, R/M=4(RSP)
/// ```
pub fn modrm_digit_rsp(opcode_digit: u8) -> u8 {
    MODRM_REGISTER_DIRECT_BASE
        | ((opcode_digit & REGISTER_NUMBER_MASK) << MODRM_REG_FIELD_SHIFT)
        | Register::RSP.number()
}

/// ModR/M의 Mod 필드: 01 = [reg + disp8] (8비트 변위)
const MODRM_MOD_DISP8: u8 = 0b01;

/// ModR/M의 Mod 필드: 10 = [reg + disp32] (32비트 변위)
const MODRM_MOD_DISP32: u8 = 0b10;

/// ModR/M 바이트를 생성합니다 (메모리 주소 지정: [base_reg + disp8])
///
/// 메모리 접근 시 베이스 레지스터에 8비트 변위를 더한 주소를 인코딩합니다.
///
/// ModR/M byte format: [Mod(2) | Reg(3) | R/M(3)]
/// - Mod = 01: [reg + disp8] 모드 (8비트 변위)
/// - Reg = 목적지/소스 레지스터 또는 opcode extension
/// - R/M = 베이스 레지스터 (bits 0-2)
///
/// # Parameters
/// - `reg`: Reg 필드에 인코딩할 레지스터
/// - `base`: R/M 필드에 인코딩할 베이스 레지스터
///
/// # Examples
///
/// ```
/// use foolang::platforms::amd64::{Register, modrm_reg_base_disp8};
///
/// // LEA RAX, [RBP + disp8]
/// assert_eq!(modrm_reg_base_disp8(Register::RAX, Register::RBP), 0x45);
/// // 0x45 = 01 000 101 = Mod=01, Reg=0(RAX), R/M=5(RBP)
/// ```
pub fn modrm_reg_base_disp8(reg: Register, base: Register) -> u8 {
    (MODRM_MOD_DISP8 << 6)
        | ((reg.number() & REGISTER_NUMBER_MASK) << MODRM_REG_FIELD_SHIFT)
        | (base.number() & REGISTER_NUMBER_MASK)
}

/// ModR/M 바이트를 생성합니다 (메모리 주소 지정: [base_reg + disp32])
///
/// 메모리 접근 시 베이스 레지스터에 32비트 변위를 더한 주소를 인코딩합니다.
///
/// ModR/M byte format: [Mod(2) | Reg(3) | R/M(3)]
/// - Mod = 10: [reg + disp32] 모드 (32비트 변위)
/// - Reg = 목적지/소스 레지스터 또는 opcode extension
/// - R/M = 베이스 레지스터 (bits 0-2)
///
/// # Parameters
/// - `reg`: Reg 필드에 인코딩할 레지스터
/// - `base`: R/M 필드에 인코딩할 베이스 레지스터
///
/// # Examples
///
/// ```
/// use foolang::platforms::amd64::{Register, modrm_reg_base_disp32};
///
/// // LEA RAX, [RBP + disp32]
/// assert_eq!(modrm_reg_base_disp32(Register::RAX, Register::RBP), 0x85);
/// // 0x85 = 10 000 101 = Mod=10, Reg=0(RAX), R/M=5(RBP)
/// ```
pub fn modrm_reg_base_disp32(reg: Register, base: Register) -> u8 {
    (MODRM_MOD_DISP32 << 6)
        | ((reg.number() & REGISTER_NUMBER_MASK) << MODRM_REG_FIELD_SHIFT)
        | (base.number() & REGISTER_NUMBER_MASK)
}
