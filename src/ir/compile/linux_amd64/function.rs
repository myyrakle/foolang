use crate::{
    ir::{
        ast::global::function::FunctionDefinition,
        error::{IRError, IRErrorKind},
    },
    platforms::{
        amd64::{
            instruction::Instruction,
            register::{modrm_digit_rsp, modrm_reg_reg, Register},
            rex::RexPrefix,
        },
        linux::elf::{
            object::ELFObject,
            section::SectionType,
            symbol::{Symbol, SymbolBinding, SymbolType},
        },
    },
};

use super::statements;
use std::collections::HashMap;

/// 변수 저장 위치
#[derive(Debug, Clone)]
pub enum VariableLocation {
    /// 레지스터에 저장
    Register(Register),
    /// 스택에 저장 (RBP 기준 오프셋, 음수)
    Stack(i32),
}

/// 라벨 위치 정보
#[derive(Debug, Clone)]
pub enum LabelLocation {
    /// 라벨이 이미 정의됨 (텍스트 섹션 내 오프셋)
    Defined(usize),
    /// 라벨이 아직 정의되지 않음 (forward reference)
    /// Vec<usize>는 이 라벨을 참조하는 점프 명령의 오프셋 목록
    Undefined(Vec<usize>),
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

    /// 라벨 이름 -> 위치 정보
    pub labels: HashMap<String, LabelLocation>,

    /// prologue에서 할당한 실제 스택 크기
    /// epilogue에서 동일한 크기를 복원하기 위해 저장
    pub allocated_stack_size: i32,

    /// alloca 명령어들이 필요로 하는 총 스택 크기
    /// prescan 단계에서 미리 계산되며, prologue에서 스택을 할당할 때 포함됨
    pub pending_alloca_size: i32,
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
            labels: HashMap::new(),
            allocated_stack_size: 0,
            pending_alloca_size: 0,
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

    /// 라벨 정의 등록 (라벨이 있는 위치의 코드 오프셋 저장)
    pub fn define_label(&mut self, label_name: String, offset: usize) {
        self.labels
            .insert(label_name, LabelLocation::Defined(offset));
    }

    /// 라벨 참조 추가 (점프 명령에서 사용, forward reference 처리)
    pub fn add_label_reference(&mut self, label_name: String, jump_offset: usize) {
        let entry = self
            .labels
            .entry(label_name)
            .or_insert_with(|| LabelLocation::Undefined(Vec::new()));

        if let LabelLocation::Undefined(refs) = entry {
            refs.push(jump_offset);
        }
    }

    /// 라벨의 위치 조회
    pub fn get_label_location(&self, label_name: &str) -> Option<&LabelLocation> {
        self.labels.get(label_name)
    }

    /// 필요한 스택 크기 계산 (16바이트 정렬)
    ///
    /// x86-64 System V ABI 요구사항:
    /// - CALL 명령어 실행 직전: RSP % 16 == 0
    /// - 함수 진입 시점: RSP % 16 == 8 (return address)
    ///
    /// Prologue 실행 후 스택 상태:
    /// 1. push rbp          -> RSP % 16 == 0 (return address 8 + rbp 8 = 16)
    /// 2. push N개 레지스터 -> RSP % 16 == (N*8) % 16
    /// 3. sub rsp, X        -> RSP % 16 == (N*8 + X) % 16
    ///
    /// CALL 전 RSP를 16바이트 정렬하려면 (N*8 + X) % 16 == 0:
    /// - N이 짝수: N*8 % 16 == 0 → X는 16의 배수
    /// - N이 홀수: N*8 % 16 == 8 → X는 16k+8 형태 (8바이트 추가 필요)
    pub fn required_stack_size(&self) -> i32 {
        let local_size = if self.stack_offset == 0 {
            0
        } else {
            -self.stack_offset
        };

        // alloca 명령어들이 필요로 하는 크기 포함
        let total_local_size = local_size + self.pending_alloca_size;

        // callee-saved 레지스터 개수
        let callee_saved_count = self.used_callee_saved.len();

        // 정렬 보정: callee-saved 레지스터가 홀수 개면 8바이트 추가
        let alignment_padding = if callee_saved_count % 2 == 1 { 8 } else { 0 };

        let total_size = total_local_size + alignment_padding;

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

    // callee-saved 레지스터 저장: push rbx, push r12, ...
    // (epilogue에서 역순으로 복원하기 위해 sub rsp 전에 실행)
    for reg in &context.used_callee_saved {
        if reg.requires_rex() {
            object.text_section.data.push(RexPrefix::RexB as u8); // REX.B
        }
        object
            .text_section
            .data
            .push(Instruction::Push as u8 + (reg.number() & Instruction::REG_NUMBER_MASK));
    }

    // 스택 공간 할당: sub rsp, stack_size
    let stack_size = context.required_stack_size();
    context.allocated_stack_size = stack_size; // 할당한 크기 저장
    if stack_size > 0 {
        object.text_section.data.push(RexPrefix::RexW as u8);
        object.text_section.data.push(Instruction::ALU_RM64_IMM32); // SUB r/m64, imm32
        object
            .text_section
            .data
            .push(modrm_digit_rsp(Instruction::OPCODE_EXT_SUB)); // ModR/M: SUB rsp
        object
            .text_section
            .data
            .extend_from_slice(&(stack_size as u32).to_le_bytes());
    }

    // 2단계: LocalStatements 컴파일 (변수 할당은 이미 결정됨)
    statements::compile_statements(&function.function_body.statements, &mut context, object)?;

    // 3단계: 미정의 라벨 검증
    for (label_name, location) in &context.labels {
        if matches!(location, LabelLocation::Undefined(_)) {
            return Err(IRError::new(
                IRErrorKind::LabelNotFound,
                &format!("Label '{}' is referenced but never defined", label_name),
            ));
        }
    }

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

/// Statement를 미리 스캔하여 필요한 변수를 할당하고 alloca 크기 계산
fn prescan_statements(
    statements: &[crate::ir::ast::local::LocalStatement],
    context: &mut FunctionContext,
) {
    use crate::ir::ast::local::{
        assignment::AssignmentStatementValue, instruction::InstructionStatement, LocalStatement,
    };

    for stmt in statements {
        if let LocalStatement::Assignment(assignment) = stmt {
            // assignment의 변수 이름을 미리 할당
            let var_name = assignment.name.name.clone();
            context.allocate_variable(var_name);

            // alloca instruction이 있으면 필요한 크기 누적
            if let AssignmentStatementValue::Instruction(InstructionStatement::Alloca(
                alloca_inst,
            )) = &assignment.value
            {
                let type_size = alloca_inst.type_.size_in_bytes();
                context.pending_alloca_size += type_size as i32;
            }
        }
    }
}

/// Function epilogue 생성 (callee-saved 레지스터 복원, 스택 해제, return)
pub fn generate_epilogue(context: &FunctionContext, object: &mut ELFObject) {
    // 스택 해제: add rsp, allocated_stack_size
    // (prologue의 sub rsp 역순 - pop callee-saved 전에 실행)
    // prologue에서 할당한 크기와 동일한 크기를 복원
    let stack_size = context.allocated_stack_size;
    if stack_size > 0 {
        object.text_section.data.push(RexPrefix::RexW as u8);
        object.text_section.data.push(Instruction::ALU_RM64_IMM32); // ADD r/m64, imm32
        object
            .text_section
            .data
            .push(modrm_digit_rsp(Instruction::OPCODE_EXT_ADD)); // ModR/M: ADD rsp
        object
            .text_section
            .data
            .extend_from_slice(&(stack_size as u32).to_le_bytes());
    }

    // callee-saved 레지스터 복원 (역순으로 pop)
    for reg in context.used_callee_saved.iter().rev() {
        if reg.requires_rex() {
            object.text_section.data.push(RexPrefix::RexB as u8); // REX.B
        }
        object
            .text_section
            .data
            .push(Instruction::Pop as u8 + (reg.number() & Instruction::REG_NUMBER_MASK));
    }

    // pop rbp (스택 프레임 복원)
    object
        .text_section
        .data
        .push(Instruction::Pop as u8 + Register::RBP.number());

    // ret
    object.text_section.data.push(Instruction::Ret as u8);
}
