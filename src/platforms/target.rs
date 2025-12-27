use std::{fmt::Display, str::FromStr};

use serde::Deserialize;

/// Defines the supported compilation targets based on architecture and operating system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    Amd64Linux,   // 64-bit x86 architecture on Linux
    Arm64Linux,   // 64-bit ARM architecture on Linux
    Amd64Windows, // 64-bit x86 architecture on Windows
    Arm64Windows, // 64-bit ARM architecture on Windows
    Amd64Darwin,  // 64-bit x86 architecture on macOS
    Arm64Darwin,  // 64-bit ARM architecture on macOS
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
            "amd64-linux" => Ok(Target::Amd64Linux),
            "arm64-linux" => Ok(Target::Arm64Linux),
            "amd64-windows" => Ok(Target::Amd64Windows),
            "arm64-windows" => Ok(Target::Arm64Windows),
            "amd64-darwin" => Ok(Target::Amd64Darwin),
            "arm64-darwin" => Ok(Target::Arm64Darwin),
            "webassembly" => Ok(Target::WebAssembly),
            "javascript" => Ok(Target::JavaScript),
            _ => Err(serde::de::Error::custom(format!("Unknown target: {}", s))),
        }
    }
}

impl Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let target_str = match self {
            Target::Amd64Linux => "amd64-linux",
            Target::Arm64Linux => "arm64-linux",
            Target::Amd64Windows => "amd64-windows",
            Target::Arm64Windows => "arm64-windows",
            Target::Amd64Darwin => "amd64-darwin",
            Target::Arm64Darwin => "arm64-darwin",
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
            "amd64-linux" => Ok(Target::Amd64Linux),
            "arm64-linux" => Ok(Target::Arm64Linux),
            "amd64-windows" => Ok(Target::Amd64Windows),
            "arm64-windows" => Ok(Target::Arm64Windows),
            "amd64-darwin" => Ok(Target::Amd64Darwin),
            "arm64-darwin" => Ok(Target::Arm64Darwin),
            _ => Err(format!("Unknown target: {}", input).into()),
        }
    }
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
