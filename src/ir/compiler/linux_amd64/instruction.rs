use crate::{
    ir::{
        ast::local::LocalStatement, compiler::linux_amd64::call::compile_call_instruction,
        error::IRError,
    },
    platforms::linux::elf::object::ELFObject,
};

pub fn compile_statements(
    statements: &[LocalStatement],
    object: &mut ELFObject,
) -> Result<(), IRError> {
    for statement in statements {
        compile_statement(statement, object)?;
    }
    Ok(())
}

fn compile_statement(stmt: &LocalStatement, object: &mut ELFObject) -> Result<(), IRError> {
    match stmt {
        LocalStatement::Instruction(statement) => {
            use crate::ir::ast::local::instruction::InstructionStatement;
            match statement {
                InstructionStatement::Call(instruction) => {
                    compile_call_instruction(instruction, object)?;
                }
                InstructionStatement::Add(_) => {
                    return Err(IRError::new("Add instruction not yet implemented"));
                }
                InstructionStatement::Return(_instruction) => todo!(),
                InstructionStatement::Sub(_instruction) => todo!(),
                InstructionStatement::Mul(_instruction) => todo!(),
                InstructionStatement::Div(_instruction) => todo!(),
                InstructionStatement::Branch(_instruction) => todo!(),
                InstructionStatement::Jump(_instruction) => todo!(),
                InstructionStatement::Compare(_instruction) => todo!(),
                InstructionStatement::Alloca(_instruction) => todo!(),
                InstructionStatement::Load(_instruction) => todo!(),
                InstructionStatement::Store(_instruction) => todo!(),
            }
        }
        LocalStatement::Assignment(_) => {
            return Err(IRError::new("Assignment statement not yet implemented"));
        }
        LocalStatement::Label(_) => {
            return Err(IRError::new("Label statement not yet implemented"));
        }
    }
    Ok(())
}
