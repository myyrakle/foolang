use std::{fmt::Display, str::FromStr};

use serde::Deserialize;

/// Defines the supported compilation targets based on architecture and operating system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    LinuxAmd64,   // 64-bit x86 architecture on Linux
    LinuxArm64,   // 64-bit ARM architecture on Linux
    WindowsAmd64, // 64-bit x86 architecture on Windows
    WindowsArm64, // 64-bit ARM architecture on Windows
    DarwinAmd64,  // 64-bit x86 architecture on macOS
    DarwinArm64,  // 64-bit ARM architecture on macOS
    WebAssembly,  // WebAssembly target
    JavaScript,   // JavaScript target
}

impl Default for Target {
    fn default() -> Self {
        detect_current_target()
    }
}

impl<'de> Deserialize<'de> for Target {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        match s {
            "linux-amd64" => Ok(Target::LinuxAmd64),
            "linux-arm64" => Ok(Target::LinuxArm64),
            "windows-amd64" => Ok(Target::WindowsAmd64),
            "windows-arm64" => Ok(Target::WindowsArm64),
            "darwin-amd64" => Ok(Target::DarwinAmd64),
            "darwin-arm64" => Ok(Target::DarwinArm64),
            "webassembly" => Ok(Target::WebAssembly),
            "javascript" => Ok(Target::JavaScript),
            _ => Err(serde::de::Error::custom(format!("Unknown target: {}", s))),
        }
    }
}

impl Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let target_str = match self {
            Target::LinuxAmd64 => "linux-amd64",
            Target::LinuxArm64 => "linux-arm64",
            Target::WindowsAmd64 => "windows-amd64",
            Target::WindowsArm64 => "windows-arm64",
            Target::DarwinAmd64 => "darwin-amd64",
            Target::DarwinArm64 => "darwin-arm64",
            Target::WebAssembly => "webassembly",
            Target::JavaScript => "javascript",
        };
        write!(f, "{}", target_str)
    }
}

impl FromStr for Target {
    type Err = Box<dyn std::error::Error + Send + Sync>;

    fn from_str(input: &str) -> Result<Target, Self::Err> {
        match input {
            "amd64-linux" => Ok(Target::LinuxAmd64),
            "arm64-linux" => Ok(Target::LinuxArm64),
            "windows-amd64" => Ok(Target::WindowsAmd64),
            "windows-arm64" => Ok(Target::WindowsArm64),
            "darwin-amd64" => Ok(Target::DarwinAmd64),
            "darwin-arm64" => Ok(Target::DarwinArm64),
            _ => Err(format!("Unknown target: {}", input).into()),
        }
    }
}

/// Detects the current compilation target based on architecture and OS.
pub fn detect_current_target() -> Target {
    if cfg!(target_arch = "x86_64") {
        if cfg!(target_os = "linux") {
            Target::LinuxAmd64
        } else if cfg!(target_os = "windows") {
            Target::WindowsAmd64
        } else if cfg!(target_os = "macos") {
            Target::DarwinAmd64
        } else {
            unimplemented!("Unsupported OS for x86_64 architecture")
        }
    } else if cfg!(target_arch = "aarch64") {
        if cfg!(target_os = "linux") {
            Target::LinuxArm64
        } else if cfg!(target_os = "windows") {
            Target::WindowsArm64
        } else if cfg!(target_os = "macos") {
            Target::DarwinArm64
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
