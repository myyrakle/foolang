/// Defines the supported compilation targets based on architecture and operating system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    Amd64Linux,   // 64-bit x86 architecture on Linux
    Arm64Linux,   // 64-bit ARM architecture on Linux
    Amd64Windows, // 64-bit x86 architecture on Windows
    Arm64Windows, // 64-bit ARM architecture on Windows
    Amd64Darwin,  // 64-bit x86 architecture on macOS
    Arm64Darwin,  // 64-bit ARM architecture on macOS
}

/// Detects the current compilation target based on architecture and OS.
pub fn detect_current_target() -> Target {
    if cfg!(target_arch = "x86_64") {
        if cfg!(target_os = "linux") {
            Target::Amd64Linux
        } else if cfg!(target_os = "windows") {
            Target::Amd64Windows
        } else if cfg!(target_os = "macos") {
            Target::Amd64Darwin
        } else {
            unimplemented!("Unsupported OS for x86_64 architecture")
        }
    } else if cfg!(target_arch = "aarch64") {
        if cfg!(target_os = "linux") {
            Target::Arm64Linux
        } else if cfg!(target_os = "windows") {
            Target::Arm64Windows
        } else if cfg!(target_os = "macos") {
            Target::Arm64Darwin
        } else {
            unimplemented!("Unsupported OS for aarch64 architecture")
        }
    } else {
        unimplemented!("Unsupported architecture")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_detect_current_target() {
        let target = detect_current_target();
        println!("Detected target: {:?}", target);
        // Note: This test will only print the detected target.
    }
}
