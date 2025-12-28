/// Linux system call numbers for x86-64
pub mod amd64 {
    /// sys_read - Read from file descriptor
    pub const SYS_READ: u8 = 0;

    /// sys_write - Write to file descriptor
    pub const SYS_WRITE: u8 = 1;

    /// sys_exit - Terminate process
    pub const SYS_EXIT: u8 = 60;
}

/// Linux system call numbers for ARM64
pub mod arm64 {
    /// sys_read - Read from file descriptor
    pub const SYS_READ: u8 = 63;

    /// sys_write - Write to file descriptor
    pub const SYS_WRITE: u8 = 64;

    /// sys_exit - Terminate process
    pub const SYS_EXIT: u8 = 93;
}
