pub mod instruction;
pub mod register;
pub mod rex;

/// ModR/M byte constants for common register operations
pub mod modrm {
    /// LEA RSI, [RIP+disp32] - RIP-relative addressing mode
    /// Binary: 00 110 101 (Mod=00, Reg=110(RSI), R/M=101(RIP-relative))
    pub const LEA_RSI_RIP_REL: u8 = 0x35;

    /// XOR RDI, RDI - Register-direct mode
    /// Binary: 11 111 111 (Mod=11, Reg=111(RDI), R/M=111(RDI))
    pub const XOR_RDI_RDI: u8 = 0xFF;
}

/// Linux system call numbers for x86-64
pub mod syscall {
    /// sys_write - Write to file descriptor
    pub const SYS_WRITE: u8 = 1;

    /// sys_exit - Terminate process
    pub const SYS_EXIT: u8 = 60;
}

/// Standard file descriptors
pub mod fd {
    /// Standard output
    pub const STDOUT: u8 = 1;
}

#[cfg(test)]
mod tests {
    use crate::platforms::amd64::{instruction::Instruction, register::Register};

    // Register tests
    #[test]
    fn test_register_as_u8() {
        assert_eq!(Register::RAX.as_u8(), 0xC0);
        assert_eq!(Register::RCX.as_u8(), 0xC1);
        assert_eq!(Register::RDX.as_u8(), 0xC2);
        assert_eq!(Register::RBX.as_u8(), 0xC3);
        assert_eq!(Register::RSP.as_u8(), 0xC4);
        assert_eq!(Register::RBP.as_u8(), 0xC5);
        assert_eq!(Register::RSI.as_u8(), 0xC6);
        assert_eq!(Register::RDI.as_u8(), 0xC7);
        assert_eq!(Register::R8.as_u8(), 0xC8);
        assert_eq!(Register::R9.as_u8(), 0xC9);
        assert_eq!(Register::R10.as_u8(), 0xCA);
        assert_eq!(Register::R11.as_u8(), 0xCB);
        assert_eq!(Register::R12.as_u8(), 0xCC);
        assert_eq!(Register::R13.as_u8(), 0xCD);
        assert_eq!(Register::R14.as_u8(), 0xCE);
        assert_eq!(Register::R15.as_u8(), 0xCF);
    }

    #[test]
    fn test_register_as_i32() {
        assert_eq!(Register::RAX.as_i32(), 0xC0);
        assert_eq!(Register::RBX.as_i32(), 0xC3);
        assert_eq!(Register::R8.as_i32(), 0xC8);
        assert_eq!(Register::R15.as_i32(), 0xCF);
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
