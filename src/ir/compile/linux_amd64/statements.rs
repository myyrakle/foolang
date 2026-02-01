use crate::{
    ir::{
        ast::local::{
            assignment::{AssignmentStatement, AssignmentStatementValue},
            instruction::InstructionStatement,
            LocalStatement,
        },
        compile::linux_amd64::{
            add::compile_add_instruction,
            alloca::{
                compile_alloca_instruction, compile_load_instruction, compile_store_instruction,
            },
            branch::{
                compile_branch_instruction, compile_jump_instruction, compile_label_definition,
            },
            call::compile_call_instruction,
            compare::compile_compare_instruction,
            div::compile_div_instruction,
            function::FunctionContext,
            mul::compile_mul_instruction,
            rem::compile_rem_instruction,
            return_::compile_return_instruction,
            sub::compile_sub_instruction,
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
    // SSA 모드: basic block 단위로 순회하며 current_block도 갱신
    if context.liveness.is_some() && !context.basic_blocks.is_empty() {
        for block in context.basic_blocks.clone() {
            context.current_block = block.id;

            for (stmt_idx, statement) in block.statements.iter().enumerate() {
                context.current_statement_index = stmt_idx;
                compile_statement(statement, context, object)?;
            }
        }
    } else {
        // 기존 모드: 단순 statement 순회
        for (stmt_idx, statement) in statements.iter().enumerate() {
            context.current_statement_index = stmt_idx;
            compile_statement(statement, context, object)?;
        }
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
        AssignmentStatementValue::Instruction(InstructionStatement::Add(instruction)) => {
            compile_add_instruction(instruction, context, object)?;
        }
        AssignmentStatementValue::Instruction(InstructionStatement::Sub(instruction)) => {
            compile_sub_instruction(instruction, context, object)?;
        }
        AssignmentStatementValue::Instruction(InstructionStatement::Mul(instruction)) => {
            compile_mul_instruction(instruction, context, object)?;
        }
        AssignmentStatementValue::Instruction(InstructionStatement::Div(instruction)) => {
            compile_div_instruction(instruction, context, object)?;
        }
        AssignmentStatementValue::Instruction(InstructionStatement::Rem(instruction)) => {
            compile_rem_instruction(instruction, context, object)?;
        }
        AssignmentStatementValue::Instruction(InstructionStatement::Call(instruction)) => {
            // call instruction 컴파일 (결과는 RAX에)
            compile_call_instruction(instruction, context, object)?;
        }
        AssignmentStatementValue::Instruction(InstructionStatement::Compare(instruction)) => {
            compile_compare_instruction(instruction, context, object)?;
        }
        AssignmentStatementValue::Instruction(InstructionStatement::Alloca(instruction)) => {
            compile_alloca_instruction(instruction, context, object)?;
        }
        AssignmentStatementValue::Instruction(InstructionStatement::Load(instruction)) => {
            compile_load_instruction(instruction, context, object)?;
        }
        _ => {
            return Err(IRError::new(
                IRErrorKind::AssignmentRequired,
                "Not supported instruction in assignment",
            ));
        }
    }

    // Phase 11: SSA 기반 변수 할당
    // SSA가 활성화된 경우 새로운 할당 방식 사용, 아니면 기존 방식 사용
    let var_name = assignment_statement.name.name.clone();

    let var_loc = if context.liveness.is_some() {
        // SSA 기반 할당
        use crate::ir::ast::types::IRType;
        use crate::ir::ssa::register_allocator::ValueLocation as SSAValueLocation;

        // 1. 새 SSA 값 생성
        let ssa_id = context.new_ssa_value(Some(var_name.clone()), IRType::None);

        // 2. SSA 값에 레지스터/스택 할당 (liveness 기반)
        let ssa_loc = context.allocate_ssa_value(ssa_id)?;

        // 3. SSAValueLocation을 VariableLocation으로 변환
        match ssa_loc {
            SSAValueLocation::Register(reg) => VariableLocation::Register(reg),
            SSAValueLocation::Spilled(offset) => VariableLocation::Stack(offset),
            SSAValueLocation::Unassigned => {
                return Err(IRError::new(
                    IRErrorKind::NotImplemented,
                    "SSA value unassigned",
                ));
            }
        }
    } else {
        // 기존 방식: 변수명 기반 할당
        context.allocate_variable(var_name.clone())
    };

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

    // Phase 15: SSA 모드에서도 variables HashMap 업데이트 (legacy 변수 조회 지원)
    if context.liveness.is_some() {
        context.variables.insert(var_name, var_loc);
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
                "Sub instruction needs assignment",
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
        InstructionStatement::Rem(_) => {
            return Err(IRError::new(
                IRErrorKind::AssignmentRequired,
                "Rem instruction need assignment",
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
        InstructionStatement::Store(instruction) => {
            compile_store_instruction(instruction, context, object)?;
        }
    }

    Ok(())
}
