#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TargetPlatform {
    Amd64Linux,
    Arm64Linux,
    Amd64Windows,
    Arm64Windows,
    Amd64Darwin,
    Arm64Darwin,
}
