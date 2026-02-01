use super::{BasicBlock, BasicBlockId, SSAValueId};
use crate::ir::ast::local::LocalStatement;
use std::collections::HashMap;

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
            renamer.rename_block(BasicBlockId::new(0), blocks);
        }
    }

    /// 특정 블록과 그 지배당하는 블록들을 재귀적으로 rename
    fn rename_block(&mut self, block_id: BasicBlockId, blocks: &mut [BasicBlock]) {
        let block_idx = block_id.as_usize();
        if block_idx >= blocks.len() {
            return;
        }

        // 1. 현재 블록의 Phi 노드 처리 (새 버전 생성)
        let phi_results: Vec<(SSAValueId, String)> = blocks[block_idx]
            .phi_nodes
            .iter()
            .map(|phi| {
                // Phi 결과에 대한 변수명 추출 (임시: 첫 입력의 변수명 사용)
                // TODO: 실제로는 모든 입력이 같은 변수명이어야 함
                let var_name = format!("phi_{}", phi.result.as_usize());
                let new_ssa_id = self.new_version(&var_name);
                (phi.result, var_name)
            })
            .collect();

        // Phi 결과를 스택에 push (이미 new_version에서 수행됨)
        let _ = phi_results.len(); // 컴파일 경고 방지

        // 2. 현재 블록의 statement 처리
        let statements_clone = blocks[block_idx].statements.clone();
        for stmt in &statements_clone {
            match stmt {
                LocalStatement::Assignment(assignment) => {
                    let var_name = assignment.name.name.clone();

                    // 우측(값)의 변수 참조 rename
                    // TODO: assignment.value 내부의 Identifier를 SSAValueId로 변환

                    // 좌측(정의): 새 버전 생성
                    let new_ssa_id = self.new_version(&var_name);

                    // defined_variables 업데이트
                    blocks[block_idx]
                        .defined_variables
                        .insert(var_name, new_ssa_id);
                }
                _ => {
                    // Label, Instruction은 변수 정의 없음
                }
            }
        }

        // 3. Successor 블록의 Phi 노드 operand 업데이트
        let successors: Vec<BasicBlockId> = blocks[block_idx].successors.clone();
        for succ_id in successors {
            let succ_idx = succ_id.as_usize();
            if succ_idx >= blocks.len() {
                continue;
            }

            // Successor의 각 Phi 노드에 대해
            for _phi_idx in 0..blocks[succ_idx].phi_nodes.len() {
                // 현재 블록에서 사용 가능한 버전 찾기
                // TODO: Phi 노드의 변수명 추출 및 매칭

                // 임시: phi.inputs에서 (block_id, ssa_id) 쌍 업데이트
                // 실제로는 변수명을 알아야 현재 버전을 찾을 수 있음
            }
        }

        // 4. 지배당하는 자식 블록들 재귀 처리
        // 간단한 구현: 직접 successor들을 처리 (실제로는 dominator tree 필요)
        let children: Vec<BasicBlockId> = blocks[block_idx].successors.clone();
        for child_id in children {
            if child_id.as_usize() > block_idx {
                // 앞으로만 진행 (back-edge 방지)
                self.rename_block(child_id, blocks);
            }
        }

        // 5. 블록을 나갈 때 스택 복원 (push했던 버전들 pop)
        for (_old_id, var_name) in phi_results {
            if let Some(stack) = self.version_stacks.get_mut(&var_name) {
                stack.pop();
            }
        }

        for stmt in &statements_clone {
            if let LocalStatement::Assignment(assignment) = stmt {
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
