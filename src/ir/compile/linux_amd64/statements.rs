use crate::{
    ir::{
        ast::local::{
            assignment::{AssignmentStatement, AssignmentStatementValue},
            instruction::InstructionStatement,
            LocalStatement,
        },
        compile::linux_amd64::{
            branch::{
                compile_branch_instruction, compile_jump_instruction, compile_label_definition,
            },
            call::compile_call_instruction,
            function::FunctionContext,
            return_::compile_return_instruction,
        },
        error::{IRError, IRErrorKind},
    },
    platforms::{
        amd64::addressing::{modrm_rbp_disp32, sib_rbp_no_index},
        linux::elf::object::ELFObject,
    },
};

pub fn compile_statements(
    statements: &[LocalStatement],
    context: &mut FunctionContext,
    object: &mut ELFObject,
) -> Result<(), IRError> {
    for statement in statements {
        compile_statement(statement, context, object)?;
    }

    Ok(())
}

fn compile_statement(
    stmt: &LocalStatement,
    context: &mut FunctionContext,
    object: &mut ELFObject,
) -> Result<(), IRError> {
    match stmt {
        LocalStatement::Instruction(statement) => {
            compile_instruction_statement(statement, context, object)?;
        }
        LocalStatement::Assignment(assignment_statement) => {
            compile_assignment_statement(assignment_statement, context, object)?;
        }
        LocalStatement::Label(label_definition) => {
            compile_label_definition(label_definition, context, object)?;
        }
    }

    Ok(())
}

fn compile_assignment_statement(
    assignment_statement: &AssignmentStatement,
    context: &mut FunctionContext,
    object: &mut ELFObject,
) -> Result<(), IRError> {
    use crate::ir::compile::linux_amd64::function::VariableLocation;
    use crate::platforms::amd64::{
        instruction::Instruction, register::modrm_reg_reg, register::Register, rex::RexPrefix,
    };

    // assignment value 컴파일 (결과는 RAX에 저장됨)
    match &assignment_statement.value {
        AssignmentStatementValue::Literal(_literal) => {
            return Err(IRError::new(
                IRErrorKind::NotImplemented,
                "Literal assignment not yet implemented",
            ));
        }
        AssignmentStatementValue::Instruction(InstructionStatement::Add(_)) => {
            return Err(IRError::new(
                IRErrorKind::NotImplemented,
                "Add instruction not yet implemented",
            ));
        }
        AssignmentStatementValue::Instruction(InstructionStatement::Sub(_)) => {
            return Err(IRError::new(
                IRErrorKind::NotImplemented,
                "Sub instruction not yet implemented",
            ));
        }
        AssignmentStatementValue::Instruction(InstructionStatement::Mul(_)) => {
            return Err(IRError::new(
                IRErrorKind::NotImplemented,
                "Mul instruction not yet implemented",
            ));
        }
        AssignmentStatementValue::Instruction(InstructionStatement::Div(_)) => {
            return Err(IRError::new(
                IRErrorKind::NotImplemented,
                "Div instruction not yet implemented",
            ));
        }
        AssignmentStatementValue::Instruction(InstructionStatement::Call(instruction)) => {
            // call instruction 컴파일 (결과는 RAX에)
            compile_call_instruction(instruction, context, object)?;
        }
        AssignmentStatementValue::Instruction(InstructionStatement::Compare(_)) => {
            return Err(IRError::new(
                IRErrorKind::NotImplemented,
                "Compare instruction not yet implemented",
            ));
        }
        AssignmentStatementValue::Instruction(InstructionStatement::Alloca(_)) => {
            return Err(IRError::new(
                IRErrorKind::NotImplemented,
                "Alloca instruction not yet implemented",
            ));
        }
        AssignmentStatementValue::Instruction(InstructionStatement::Load(_)) => {
            return Err(IRError::new(
                IRErrorKind::NotImplemented,
                "Load instruction not yet implemented",
            ));
        }
        _ => {
            return Err(IRError::new(
                IRErrorKind::AssignmentRequired,
                "Not supported instruction in assignment",
            ));
        }
    }

    // 변수 할당 (레지스터 우선, 부족하면 스택)
    let var_name = assignment_statement.name.name.clone();
    let var_loc = context.allocate_variable(var_name);

    // RAX의 값을 변수 위치에 저장
    match var_loc {
        VariableLocation::Register(dst_reg) => {
            // mov dst_reg, rax
            if dst_reg != Register::RAX {
                // R8-R15는 REX.R 비트 필요
                if dst_reg.requires_rex() {
                    object.text_section.data.push(RexPrefix::REX_WR);
                } else {
                    object.text_section.data.push(RexPrefix::RexW as u8);
                }
                // MOV r, r/m (reg 필드가 destination, r/m 필드가 source)
                object.text_section.data.push(Instruction::MovLoad as u8);
                object
                    .text_section
                    .data
                    .push(modrm_reg_reg(dst_reg, Register::RAX));
            }
            // RAX에 할당된 경우 이미 RAX에 있으므로 아무것도 안함
        }
        VariableLocation::Stack(offset) => {
            // mov [rbp + offset], rax
            object.text_section.data.push(RexPrefix::RexW as u8);
            object.text_section.data.push(Instruction::Mov as u8);

            // ModR/M byte: [RBP + disp32] addressing
            object
                .text_section
                .data
                .push(modrm_rbp_disp32(Register::RAX.number()));

            // SIB byte: scale=1, index=none, base=RBP
            object.text_section.data.push(sib_rbp_no_index());

            // displacement
            object
                .text_section
                .data
                .extend_from_slice(&offset.to_le_bytes());
        }
    }

    Ok(())
}

fn compile_instruction_statement(
    instruction_statement: &InstructionStatement,
    context: &mut FunctionContext,
    object: &mut ELFObject,
) -> Result<(), IRError> {
    use crate::ir::ast::local::instruction::InstructionStatement;
    match instruction_statement {
        InstructionStatement::Call(instruction) => {
            compile_call_instruction(instruction, context, object)?;
        }
        InstructionStatement::Add(_) => {
            return Err(IRError::new(
                IRErrorKind::AssignmentRequired,
                "Add instruction need assignment",
            ));
        }
        InstructionStatement::Return(instruction) => {
            compile_return_instruction(instruction, context, object)?;
        }
        InstructionStatement::Sub(_) => {
            return Err(IRError::new(
                IRErrorKind::AssignmentRequired,
                "Sub instruction need assignment",
            ));
        }
        InstructionStatement::Mul(_) => {
            return Err(IRError::new(
                IRErrorKind::AssignmentRequired,
                "Mul instruction need assignment",
            ));
        }
        InstructionStatement::Div(_) => {
            return Err(IRError::new(
                IRErrorKind::AssignmentRequired,
                "Div instruction need assignment",
            ));
        }
        InstructionStatement::Branch(instruction) => {
            compile_branch_instruction(instruction, context, object)?;
        }
        InstructionStatement::Jump(instruction) => {
            compile_jump_instruction(instruction, context, object)?;
        }
        InstructionStatement::Compare(_) => {
            return Err(IRError::new(
                IRErrorKind::AssignmentRequired,
                "Compare instruction need assignment",
            ));
        }
        InstructionStatement::Alloca(_) => {
            return Err(IRError::new(
                IRErrorKind::AssignmentRequired,
                "Alloca instruction need assignment",
            ));
        }
        InstructionStatement::Load(_instruction) => {
            return Err(IRError::new(
                IRErrorKind::AssignmentRequired,
                "Load instruction need assignment",
            ));
        }
        InstructionStatement::Store(_instruction) => todo!(),
    }

    Ok(())
}
