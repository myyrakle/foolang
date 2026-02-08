use super::{BasicBlock, BasicBlockId, SSAValueId};
use crate::ir::ast::local::LocalStatement;
use std::collections::{HashMap, HashSet};

/// SSA Renaming: 변수명을 SSA 값으로 변환
///
/// 알고리즘:
/// 1. 각 변수마다 버전 스택 유지 (shadowing 지원)
/// 2. DFS로 지배자 트리 순회하며 renaming
/// 3. Phi 노드의 operand 업데이트
pub struct SSARenamer {
    /// 변수명 -> SSA 값 스택
    /// 스택의 top이 현재 블록에서 유효한 버전
    version_stacks: HashMap<String, Vec<SSAValueId>>,

    /// 변수명 -> 다음 버전 번호
    version_counters: HashMap<String, usize>,

    /// 다음 SSA 값 ID
    next_ssa_id: usize,
}

impl SSARenamer {
    pub fn new() -> Self {
        Self {
            version_stacks: HashMap::new(),
            version_counters: HashMap::new(),
            next_ssa_id: 0,
        }
    }

    /// SSA renaming 수행
    pub fn rename(blocks: &mut [BasicBlock]) {
        let mut renamer = SSARenamer::new();

        // 루트 블록(entry block)부터 시작
        if !blocks.is_empty() {
            let mut visited = HashSet::new();
            renamer.rename_block(BasicBlockId::new(0), blocks, &mut visited);
        }
    }

    /// 특정 블록과 그 지배당하는 블록들을 재귀적으로 rename
    fn rename_block(
        &mut self,
        block_id: BasicBlockId,
        blocks: &mut [BasicBlock],
        visited: &mut HashSet<BasicBlockId>,
    ) {
        // 이미 방문한 블록은 건너뜀 (순환 방지)
        if !visited.insert(block_id) {
            return;
        }

        let block_idx = block_id.as_usize();
        if block_idx >= blocks.len() {
            return;
        }

        // 1. 현재 블록의 Phi 노드 처리 (새 버전 생성 및 반영)
        let mut phi_results: Vec<(SSAValueId, String)> = Vec::new();
        for phi in &mut blocks[block_idx].phi_nodes {
            // Phi 노드의 원래 변수명 사용 (모든 입력이 같은 변수를 나타냄)
            let var_name = phi.original_name.clone();

            // 새 SSA ID 생성 및 버전 스택에 push
            let new_ssa_id = self.new_version(&var_name);

            // Phi 노드의 result를 새 SSA ID로 업데이트
            phi.result = new_ssa_id;

            // 스택 복원을 위해 기록
            phi_results.push((new_ssa_id, var_name));
        }

        // 2. 현재 블록의 statement 처리
        let num_statements = blocks[block_idx].statements.len();
        for stmt_idx in 0..num_statements {
            if let LocalStatement::Assignment(ref mut assignment) =
                blocks[block_idx].statements[stmt_idx]
            {
                let var_name = assignment.name.name.clone();

                // 우측(값)의 변수 참조 rename
                // RHS의 Identifier를 현재 버전의 SSAValue로 변환
                self.rename_operands_in_assignment(&mut assignment.value);

                // 좌측(정의): 새 버전 생성
                let new_ssa_id = self.new_version(&var_name);

                // defined_variables 업데이트
                blocks[block_idx]
                    .defined_variables
                    .insert(var_name, new_ssa_id);
            }
            // Label, Instruction은 변수 정의 없음
        }

        // 3. Successor 블록의 Phi 노드 operand 업데이트
        // 현재 블록에서 successor로 전달되는 SSA 값들을 Phi 노드에 추가
        let successors: Vec<BasicBlockId> = blocks[block_idx].successors.clone();
        for succ_id in successors {
            let succ_idx = succ_id.as_usize();
            if succ_idx >= blocks.len() {
                continue;
            }

            // Successor의 각 Phi 노드에 현재 블록의 버전 추가
            for phi in &mut blocks[succ_idx].phi_nodes {
                let var_name = &phi.original_name;

                // 현재 블록에서 이 변수의 최신 버전 조회
                if let Some(ssa_id) = self.current_version(var_name) {
                    // Phi 노드의 inputs에 (현재 블록 ID, SSA 값) 추가
                    phi.inputs.push((block_id, ssa_id));
                }
            }
        }

        // 4. 지배당하는 자식 블록들 재귀 처리
        // visited set으로 순환 방지 (back-edge 자동 차단)
        let children: Vec<BasicBlockId> = blocks[block_idx].successors.clone();
        for child_id in children {
            self.rename_block(child_id, blocks, visited);
        }

        // 5. 블록을 나갈 때 스택 복원 (push했던 버전들 pop)
        // Phi 노드로 생성한 버전들 pop
        for (_old_id, var_name) in phi_results {
            if let Some(stack) = self.version_stacks.get_mut(&var_name) {
                stack.pop();
            }
        }

        // Statement로 생성한 버전들 pop
        for stmt_idx in 0..num_statements {
            if let LocalStatement::Assignment(assignment) = &blocks[block_idx].statements[stmt_idx]
            {
                let var_name = &assignment.name.name;
                if let Some(stack) = self.version_stacks.get_mut(var_name) {
                    stack.pop();
                }
            }
        }
    }

    /// 변수의 새 버전 생성 및 스택에 push
    fn new_version(&mut self, var_name: &str) -> SSAValueId {
        let new_id = SSAValueId::new(self.next_ssa_id);
        self.next_ssa_id += 1;

        // 버전 스택에 push
        self.version_stacks
            .entry(var_name.to_string())
            .or_insert_with(Vec::new)
            .push(new_id);

        // 버전 카운터 증가
        *self.version_counters
            .entry(var_name.to_string())
            .or_insert(0) += 1;

        new_id
    }

    /// 변수의 현재 버전 조회 (스택 top)
    fn current_version(&self, var_name: &str) -> Option<SSAValueId> {
        self.version_stacks
            .get(var_name)
            .and_then(|stack| stack.last().copied())
    }

    /// AssignmentStatementValue 내부의 Operand들을 rename
    fn rename_operands_in_assignment(
        &self,
        value: &mut crate::ir::ast::local::assignment::AssignmentStatementValue,
    ) {
        use crate::ir::ast::local::assignment::AssignmentStatementValue;

        match value {
            AssignmentStatementValue::Literal(_) => {
                // Literal은 renaming 불필요
            }
            AssignmentStatementValue::Instruction(instr) => {
                self.rename_operands_in_instruction(instr);
            }
        }
    }

    /// InstructionStatement 내부의 Operand들을 rename
    fn rename_operands_in_instruction(
        &self,
        instr: &mut crate::ir::ast::local::instruction::InstructionStatement,
    ) {
        use crate::ir::ast::common::Operand;
        use crate::ir::ast::local::instruction::InstructionStatement;

        // 각 Operand를 rename하는 헬퍼 클로저
        let rename_operand = |operand: &mut Operand, renamer: &SSARenamer| {
            if let Operand::Identifier(ref ident) = operand {
                if let Some(ssa_id) = renamer.current_version(&ident.name) {
                    // Identifier를 SSAValue로 변환
                    *operand = Operand::SSAValue(ssa_id);
                }
            }
        };

        match instr {
            InstructionStatement::Add(add) => {
                rename_operand(&mut add.left, self);
                rename_operand(&mut add.right, self);
            }
            InstructionStatement::Sub(sub) => {
                rename_operand(&mut sub.left, self);
                rename_operand(&mut sub.right, self);
            }
            InstructionStatement::Mul(mul) => {
                rename_operand(&mut mul.left, self);
                rename_operand(&mut mul.right, self);
            }
            InstructionStatement::Div(div) => {
                rename_operand(&mut div.left, self);
                rename_operand(&mut div.right, self);
            }
            InstructionStatement::Rem(rem) => {
                rename_operand(&mut rem.left, self);
                rename_operand(&mut rem.right, self);
            }
            InstructionStatement::Compare(cmp) => {
                rename_operand(&mut cmp.left, self);
                rename_operand(&mut cmp.right, self);
            }
            InstructionStatement::Return(ret) => {
                if let Some(ref mut operand) = ret.return_value {
                    rename_operand(operand, self);
                }
            }
            InstructionStatement::Call(call) => {
                for arg in &mut call.parameters {
                    rename_operand(arg, self);
                }
            }
            InstructionStatement::Load(_) => {
                // Load는 Operand를 포함하지 않음
            }
            InstructionStatement::Store(store) => {
                rename_operand(&mut store.value, self);
            }
            InstructionStatement::Jump(_) => {
                // Jump는 Operand를 포함하지 않음
            }
            InstructionStatement::Branch(_branch) => {
                // Branch는 Identifier를 직접 사용하므로, 현재 구조에서는 renaming 어려움
                // TODO: Branch가 Operand를 사용하도록 AST 구조 변경 필요
            }
            InstructionStatement::Alloca(_) => {
                // Alloca는 Operand를 포함하지 않음
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ssa_renamer_basic() {
        let mut renamer = SSARenamer::new();

        // 같은 변수에 대해 여러 버전 생성
        let v1 = renamer.new_version("x");
        let v2 = renamer.new_version("x");
        let v3 = renamer.new_version("x");

        // 현재 버전은 마지막
        assert_eq!(renamer.current_version("x"), Some(v3));

        // ID는 증가
        assert!(v2.as_usize() > v1.as_usize());
        assert!(v3.as_usize() > v2.as_usize());
    }
}
