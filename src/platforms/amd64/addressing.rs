/// x86-64 주소 지정 모드 관련 상수 정의
///
/// ModR/M 바이트와 SIB 바이트 인코딩에 사용되는 상수들을 정의합니다.

// ModR/M 바이트 Mod 필드 값
/// Mod = 00: 메모리 접근, displacement 없음 (특수 케이스 제외)
pub const MODRM_MOD_MEMORY_NO_DISP: u8 = 0b00;

/// Mod = 01: 메모리 접근, 8비트 displacement
pub const MODRM_MOD_MEMORY_DISP8: u8 = 0b01;

/// Mod = 10: 메모리 접근, 32비트 displacement
pub const MODRM_MOD_MEMORY_DISP32: u8 = 0b10;

/// Mod = 11: 레지스터 간 직접 연산 (메모리 접근 없음)
pub const MODRM_MOD_REGISTER_DIRECT: u8 = 0b11;

// ModR/M 바이트 R/M 필드 특수 값
/// R/M = 100: SIB 바이트 사용 (Mod != 11일 때)
pub const MODRM_RM_SIB_FOLLOWS: u8 = 0b100;

/// R/M = 101: 특수 의미 (Mod에 따라 다름)
/// - Mod = 00: RIP-relative addressing [RIP + disp32]
/// - Mod = 01/10: [RBP + disp8/32]
pub const MODRM_RM_SPECIAL: u8 = 0b101;

// SIB 바이트 필드 값
/// SIB Scale = 00: scale factor x1
pub const SIB_SCALE_1: u8 = 0b00;

/// SIB Scale = 01: scale factor x2
pub const SIB_SCALE_2: u8 = 0b01;

/// SIB Scale = 10: scale factor x4
pub const SIB_SCALE_4: u8 = 0b10;

/// SIB Scale = 11: scale factor x8
pub const SIB_SCALE_8: u8 = 0b11;

/// SIB Index = 100: 인덱스 레지스터 없음
pub const SIB_INDEX_NONE: u8 = 0b100;

/// SIB Base = 101: 특수 의미 (Mod에 따라 다름)
/// - Mod = 00: base 없음, disp32만 사용
/// - Mod = 01/10: RBP 레지스터 사용
pub const SIB_BASE_RBP: u8 = 0b101;

// 비트 시프트 및 마스크
/// ModR/M의 Mod 필드 시프트 (상위 2비트)
pub const MODRM_MOD_SHIFT: u8 = 6;

/// ModR/M의 Reg 필드 시프트 (중간 3비트)
pub const MODRM_REG_SHIFT: u8 = 3;

/// SIB의 Scale 필드 시프트 (상위 2비트)
pub const SIB_SCALE_SHIFT: u8 = 6;

/// SIB의 Index 필드 시프트 (중간 3비트)
pub const SIB_INDEX_SHIFT: u8 = 3;

/// 3비트 필드 마스크 (레지스터 번호 등)
pub const BITS_3_MASK: u8 = 0x7;

/// [RBP + disp32] 주소 지정을 위한 ModR/M 바이트 생성
///
/// # Parameters
/// - `reg`: Reg 필드에 들어갈 레지스터 번호 (0-7)
///
/// # Returns
/// ModR/M 바이트: [Mod=10 | Reg | R/M=100]
pub fn modrm_rbp_disp32(reg_num: u8) -> u8 {
    (MODRM_MOD_MEMORY_DISP32 << MODRM_MOD_SHIFT)
        | ((reg_num & BITS_3_MASK) << MODRM_REG_SHIFT)
        | MODRM_RM_SIB_FOLLOWS
}

/// [RBP + disp32]를 위한 SIB 바이트 생성
///
/// Scale=1, Index=none, Base=RBP
///
/// # Returns
/// SIB 바이트: [Scale=00 | Index=100 | Base=101]
pub fn sib_rbp_no_index() -> u8 {
    (SIB_SCALE_1 << SIB_SCALE_SHIFT) | (SIB_INDEX_NONE << SIB_INDEX_SHIFT) | SIB_BASE_RBP
}

/// [RIP + disp32] 주소 지정을 위한 ModR/M 바이트 생성
///
/// # Parameters
/// - `reg`: Reg 필드에 들어갈 레지스터 번호 (0-7)
///
/// # Returns
/// ModR/M 바이트: [Mod=00 | Reg | R/M=101]
pub fn modrm_rip_relative(reg_num: u8) -> u8 {
    (MODRM_MOD_MEMORY_NO_DISP << MODRM_MOD_SHIFT)
        | ((reg_num & BITS_3_MASK) << MODRM_REG_SHIFT)
        | MODRM_RM_SPECIAL
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modrm_rbp_disp32() {
        // mov rax, [rbp + disp32] - Reg=0(RAX)
        assert_eq!(modrm_rbp_disp32(0), 0x84);
        // 0x84 = 10 000 100 = Mod=10, Reg=0, R/M=100(SIB)

        // mov rsi, [rbp + disp32] - Reg=6(RSI)
        assert_eq!(modrm_rbp_disp32(6), 0xB4);
        // 0xB4 = 10 110 100 = Mod=10, Reg=6, R/M=100(SIB)
    }

    #[test]
    fn test_sib_rbp_no_index() {
        // [rbp] with no index, scale=1
        assert_eq!(sib_rbp_no_index(), 0x25);
        // 0x25 = 00 100 101 = Scale=00(1), Index=100(none), Base=101(RBP)
    }

    #[test]
    fn test_modrm_rip_relative() {
        // lea rsi, [rip + disp32] - Reg=6(RSI)
        assert_eq!(modrm_rip_relative(6), 0x35);
        // 0x35 = 00 110 101 = Mod=00, Reg=6(RSI), R/M=101(RIP-relative)

        // lea rax, [rip + disp32] - Reg=0(RAX)
        assert_eq!(modrm_rip_relative(0), 0x05);
        // 0x05 = 00 000 101 = Mod=00, Reg=0(RAX), R/M=101(RIP-relative)
    }
}
