use crate::platforms::linux::elf::object::ELFObject;

#[derive(Debug, Clone)]
pub enum IRCompiledObject {
    ELF(ELFObject),
    // 다른 Linux 기반 오브젝트 파일 형식 추가 가능
}

impl IRCompiledObject {}

#[derive(Debug, Clone)]
pub struct IRLinkedObject {}
