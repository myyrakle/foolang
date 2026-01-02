use crate::{
    ir::{ast::global::function::FunctionDefinition, error::IRError},
    platforms::{
        amd64::{
            instruction::Instruction,
            register::{modrm_reg_reg, Register},
            rex::RexPrefix,
        },
        linux::elf::{
            object::ELFObject,
            section::SectionType,
            symbol::{Symbol, SymbolBinding, SymbolType},
        },
    },
};

use super::instruction;
use std::collections::HashMap;

/// 변수 저장 위치
#[derive(Debug, Clone)]
pub enum VariableLocation {
    /// 레지스터에 저장
    Register(Register),
    /// 스택에 저장 (RBP 기준 오프셋, 음수)
    Stack(i32),
}

/// 함수 컴파일 컨텍스트
#[derive(Debug)]
pub struct FunctionContext {
    /// 로컬 변수 이름 -> 저장 위치
    pub variables: HashMap<String, VariableLocation>,

    /// 사용 가능한 callee-saved 레지스터 (RBX, R12-R15)
    /// x86-64 System V ABI에서 callee-saved 레지스터는 함수가 보존해야 함
    pub available_registers: Vec<Register>,

    /// 현재 스택 오프셋 (RBP 기준, 음수로 증가)
    pub stack_offset: i32,

    /// 사용된 callee-saved 레지스터 목록 (epilogue에서 복원용)
    pub used_callee_saved: Vec<Register>,
}

impl FunctionContext {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            // callee-saved 레지스터: RBX, R12, R13, R14, R15
            available_registers: vec![
                Register::RBX,
                Register::R12,
                Register::R13,
                Register::R14,
                Register::R15,
            ],
            stack_offset: 0,
            used_callee_saved: Vec::new(),
        }
    }

    /// 변수를 할당하고 위치 반환
    pub fn allocate_variable(&mut self, name: String) -> VariableLocation {
        // 이미 할당된 변수면 기존 위치 반환
        if let Some(loc) = self.variables.get(&name) {
            return loc.clone();
        }

        // 사용 가능한 레지스터가 있으면 레지스터 사용
        if let Some(reg) = self.available_registers.pop() {
            let loc = VariableLocation::Register(reg);
            self.variables.insert(name, loc.clone());
            self.used_callee_saved.push(reg);
            loc
        } else {
            // 레지스터가 없으면 스택 사용
            self.stack_offset -= 8; // 8바이트 (64비트 포인터)
            let loc = VariableLocation::Stack(self.stack_offset);
            self.variables.insert(name, loc.clone());
            loc
        }
    }

    /// 변수의 위치 조회
    pub fn get_variable(&self, name: &str) -> Option<&VariableLocation> {
        self.variables.get(name)
    }

    /// 필요한 스택 크기 계산 (16바이트 정렬)
    ///
    /// x86-64 System V ABI 요구사항:
    /// - CALL 명령어 실행 직전: RSP % 16 == 0
    /// - 함수 진입 시점: RSP % 16 == 8 (return address)
    ///
    /// Prologue 실행 후 스택 상태:
    /// 1. push rbp          -> RSP % 16 == 8
    /// 2. sub rsp, X        -> RSP % 16 == 8 (X는 16의 배수)
    /// 3. push N개 레지스터 -> RSP % 16 == (8 - N*8) % 16
    ///
    /// CALL 전 RSP를 16바이트 정렬하려면:
    /// - N이 홀수: (8 - N*8) % 16 == 0 (정렬됨)
    /// - N이 짝수: (8 - N*8) % 16 == 8 (8바이트 추가 필요)
    pub fn required_stack_size(&self) -> i32 {
        let local_size = if self.stack_offset == 0 {
            0
        } else {
            -self.stack_offset
        };

        // callee-saved 레지스터 개수
        let callee_saved_count = self.used_callee_saved.len();

        // 정렬 보정: callee-saved 레지스터가 짝수 개면 8바이트 추가
        let alignment_padding = if callee_saved_count % 2 == 0 { 8 } else { 0 };

        let total_size = local_size + alignment_padding;

        // 16바이트 정렬
        if total_size == 0 {
            0
        } else {
            ((total_size + 15) / 16) * 16
        }
    }
}

pub fn compile_function(
    function: &FunctionDefinition,
    object: &mut ELFObject,
) -> Result<(), IRError> {
    let function_start_offset = object.text_section.data.len();

    // 1단계: statement를 스캔하여 필요한 변수 목록 추출
    let mut context = FunctionContext::new();
    prescan_statements(&function.function_body.statements, &mut context);

    // Function prologue 생성
    // push rbp (스택 프레임 저장)
    object
        .text_section
        .data
        .push(Instruction::Push as u8 + Register::RBP.number());

    // mov rbp, rsp (새로운 스택 프레임 설정)
    object.text_section.data.push(RexPrefix::RexW as u8);
    object.text_section.data.push(Instruction::Mov as u8);
    object
        .text_section
        .data
        .push(modrm_reg_reg(Register::RSP, Register::RBP));

    // 스택 공간 할당: sub rsp, stack_size
    let stack_size = context.required_stack_size();
    if stack_size > 0 {
        object.text_section.data.push(RexPrefix::RexW as u8);
        object.text_section.data.push(0x81); // SUB r/m64, imm32
        object.text_section.data.push(0xEC); // ModR/M: 11 101 100 (RSP)
        object
            .text_section
            .data
            .extend_from_slice(&(stack_size as u32).to_le_bytes());
    }

    // callee-saved 레지스터 저장: push rbx, push r12, ...
    for reg in &context.used_callee_saved {
        if reg.requires_rex() {
            object.text_section.data.push(0x41); // REX.B
        }
        object
            .text_section
            .data
            .push(Instruction::Push as u8 + (reg.number() & 0x7));
    }

    // 2단계: LocalStatements 컴파일 (변수 할당은 이미 결정됨)
    instruction::compile_statements(&function.function_body.statements, &mut context, object)?;

    // 마지막 statement가 return instruction인지 확인
    let has_explicit_return = function
        .function_body
        .statements
        .last()
        .map(|stmt| {
            matches!(
                stmt,
                crate::ir::ast::local::LocalStatement::Instruction(
                    crate::ir::ast::local::instruction::InstructionStatement::Return(_)
                )
            )
        })
        .unwrap_or(false);

    // 명시적인 return이 없으면 기본 epilogue 생성
    if !has_explicit_return {
        // xor eax, eax (return 0)
        object.text_section.data.push(Instruction::Xor as u8);
        object
            .text_section
            .data
            .push(modrm_reg_reg(Register::RAX, Register::RAX));

        // epilogue 생성 (컨텍스트 전달)
        generate_epilogue(&context, object);
    }

    let function_end_offset = object.text_section.data.len();
    let function_size = function_end_offset - function_start_offset;

    // Function symbol을 symbol table에 추가
    object.symbol_table.add_symbol(Symbol {
        name: function.function_name.clone(),
        section: SectionType::Text,
        offset: function_start_offset,
        size: function_size,
        symbol_type: SymbolType::Function,
        binding: SymbolBinding::Global,
    });

    Ok(())
}

/// Statement를 미리 스캔하여 필요한 변수를 할당
fn prescan_statements(
    statements: &[crate::ir::ast::local::LocalStatement],
    context: &mut FunctionContext,
) {
    use crate::ir::ast::local::LocalStatement;

    for stmt in statements {
        if let LocalStatement::Assignment(assignment) = stmt {
            // assignment의 변수 이름을 미리 할당
            let var_name = assignment.name.name.clone();
            context.allocate_variable(var_name);
        }
    }
}

/// Function epilogue 생성 (callee-saved 레지스터 복원, 스택 해제, return)
pub fn generate_epilogue(context: &FunctionContext, object: &mut ELFObject) {
    // callee-saved 레지스터 복원 (역순으로 pop)
    for reg in context.used_callee_saved.iter().rev() {
        if reg.requires_rex() {
            object.text_section.data.push(0x41); // REX.B
        }
        object
            .text_section
            .data
            .push(Instruction::Pop as u8 + (reg.number() & 0x7));
    }

    // 스택 해제: add rsp, stack_size
    let stack_size = context.required_stack_size();
    if stack_size > 0 {
        object.text_section.data.push(RexPrefix::RexW as u8);
        object.text_section.data.push(0x81); // ADD r/m64, imm32
        object.text_section.data.push(0xC4); // ModR/M: 11 000 100 (RSP)
        object
            .text_section
            .data
            .extend_from_slice(&(stack_size as u32).to_le_bytes());
    }

    // pop rbp (스택 프레임 복원)
    object
        .text_section
        .data
        .push(Instruction::Pop as u8 + Register::RBP.number());

    // ret
    object.text_section.data.push(Instruction::Ret as u8);
}
