use super::{BasicBlock, BasicBlockId, PhiNode, SSAValueId};
use crate::ir::ast::{
    local::{assignment::AssignmentStatement, LocalStatement},
    types::IRType,
};
use std::collections::{HashMap, HashSet};

/// Phi 노드 삽입기
///
/// SSA 구축을 위해 control flow join point에 Phi 노드를 자동으로 삽입합니다.
pub struct PhiInserter {
    /// 변수별로 정의되는 블록들
    var_defs: HashMap<String, HashSet<BasicBlockId>>,

    /// 각 블록의 dominance frontier
    dominance_frontiers: HashMap<BasicBlockId, HashSet<BasicBlockId>>,
}

impl PhiInserter {
    pub fn new() -> Self {
        Self {
            var_defs: HashMap::new(),
            dominance_frontiers: HashMap::new(),
        }
    }

    /// Phi 노드 삽입 (Cytron et al. 알고리즘 간소화 버전)
    ///
    /// # 알고리즘:
    /// 1. 각 변수가 어느 블록에서 정의되는지 수집
    /// 2. Dominance frontier 계산 (간소화: 여러 predecessor가 있는 블록)
    /// 3. Join point에 Phi 노드 삽입
    pub fn insert_phi_nodes(
        blocks: &mut [BasicBlock],
        statements: &[LocalStatement],
    ) -> HashMap<BasicBlockId, Vec<String>> {
        let mut inserter = PhiInserter::new();

        // 1단계: 변수 정의 위치 수집
        inserter.collect_variable_definitions(statements);

        // 2단계: Dominance frontier 계산 (간소화)
        inserter.compute_dominance_frontiers(blocks);

        // 3단계: Phi 노드 삽입
        inserter.insert_phi_nodes_for_variables(blocks)
    }

    /// 각 변수가 어느 블록에서 정의되는지 수집
    fn collect_variable_definitions(&mut self, statements: &[LocalStatement]) {
        // 간소화: 모든 statement를 첫 번째 블록에 있다고 가정
        // 실제로는 CFG의 블록 경계를 고려해야 함
        let block_id = BasicBlockId::new(0);

        for stmt in statements {
            if let LocalStatement::Assignment(assignment) = stmt {
                let var_name = assignment.name.name.clone();
                self.var_defs
                    .entry(var_name)
                    .or_insert_with(HashSet::new)
                    .insert(block_id);
            }
        }
    }

    /// Dominance frontier 계산 (간소화 버전)
    ///
    /// 실제 dominance frontier 계산은 복잡하므로,
    /// 여기서는 간단히 "여러 predecessor가 있는 블록"을 join point로 간주
    fn compute_dominance_frontiers(&mut self, blocks: &[BasicBlock]) {
        for block in blocks {
            if block.predecessors.len() > 1 {
                // 이 블록은 join point
                // 모든 predecessor의 dominance frontier에 이 블록 추가
                for &pred_id in &block.predecessors {
                    self.dominance_frontiers
                        .entry(pred_id)
                        .or_insert_with(HashSet::new)
                        .insert(block.id);
                }
            }
        }
    }

    /// 변수별로 Phi 노드 삽입
    fn insert_phi_nodes_for_variables(
        &self,
        blocks: &mut [BasicBlock],
    ) -> HashMap<BasicBlockId, Vec<String>> {
        let mut phi_locations: HashMap<BasicBlockId, Vec<String>> = HashMap::new();

        for (var_name, def_blocks) in &self.var_defs {
            let mut phi_inserted = HashSet::new();
            let mut worklist: Vec<BasicBlockId> = def_blocks.iter().copied().collect();

            while let Some(block_id) = worklist.pop() {
                // 이 블록의 dominance frontier에 Phi 노드 삽입
                if let Some(df_blocks) = self.dominance_frontiers.get(&block_id) {
                    for &df_block_id in df_blocks {
                        if !phi_inserted.contains(&df_block_id) {
                            // Phi 노드 삽입
                            phi_locations
                                .entry(df_block_id)
                                .or_insert_with(Vec::new)
                                .push(var_name.clone());

                            phi_inserted.insert(df_block_id);

                            // Phi 노드도 정의로 간주하여 worklist에 추가
                            if !def_blocks.contains(&df_block_id) {
                                worklist.push(df_block_id);
                            }
                        }
                    }
                }
            }
        }

        // 실제 Phi 노드 생성 및 블록에 추가
        for (block_id, var_names) in &phi_locations {
            let block_idx = block_id.as_usize();
            if block_idx < blocks.len() {
                for var_name in var_names {
                    // 임시 SSA value ID 생성 (나중에 renaming 단계에서 재할당)
                    let result_id = SSAValueId::new(0);

                    let phi = PhiNode::new(
                        result_id,
                        IRType::None, // 타입은 나중에 추론
                        var_name.clone(),
                    );

                    blocks[block_idx].add_phi_node(phi);
                }
            }
        }

        phi_locations
    }
}

/// SSA Renaming - 변수를 SSA 값으로 변환
///
/// Phi 노드 삽입 후, 각 변수 사용을 적절한 SSA 값 ID로 변환
pub struct SSARenamer {
    /// 변수별 SSA 값 스택 (shadowing 지원)
    var_stacks: HashMap<String, Vec<SSAValueId>>,

    /// 다음 SSA 값 ID
    next_ssa_id: usize,
}

impl SSARenamer {
    pub fn new() -> Self {
        Self {
            var_stacks: HashMap::new(),
            next_ssa_id: 1, // 0은 undefined용으로 예약
        }
    }

    /// 변수의 현재 SSA 값 조회
    pub fn get_current_value(&self, var_name: &str) -> Option<SSAValueId> {
        self.var_stacks
            .get(var_name)
            .and_then(|stack| stack.last())
            .copied()
    }

    /// 새 SSA 값 생성 및 스택에 푸시
    pub fn push_new_value(&mut self, var_name: String) -> SSAValueId {
        let new_id = SSAValueId::new(self.next_ssa_id);
        self.next_ssa_id += 1;

        self.var_stacks
            .entry(var_name)
            .or_insert_with(Vec::new)
            .push(new_id);

        new_id
    }

    /// 스택에서 값 제거 (블록 종료 시)
    pub fn pop_value(&mut self, var_name: &str) {
        if let Some(stack) = self.var_stacks.get_mut(var_name) {
            stack.pop();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::ast::common::Identifier;

    #[test]
    fn test_phi_inserter_creation() {
        let inserter = PhiInserter::new();
        assert!(inserter.var_defs.is_empty());
        assert!(inserter.dominance_frontiers.is_empty());
    }

    #[test]
    fn test_collect_variable_definitions() {
        let mut inserter = PhiInserter::new();

        let statements = vec![LocalStatement::Assignment(AssignmentStatement {
            name: Identifier {
                type_: IRType::None,
                name: "x".to_string(),
            },
            value: crate::ir::ast::local::assignment::AssignmentStatementValue::Literal(
                crate::ir::ast::common::literal::LiteralValue::Int32(42),
            ),
        })];

        inserter.collect_variable_definitions(&statements);

        assert!(inserter.var_defs.contains_key("x"));
        assert_eq!(inserter.var_defs.get("x").unwrap().len(), 1);
    }

    #[test]
    fn test_dominance_frontiers_simple() {
        let mut inserter = PhiInserter::new();

        // 두 블록이 하나의 join point로 합류
        let mut block0 = BasicBlock::new(BasicBlockId::new(0));
        let mut block1 = BasicBlock::new(BasicBlockId::new(1));
        let mut block2 = BasicBlock::new(BasicBlockId::new(2));

        block0.add_successor(BasicBlockId::new(2));
        block1.add_successor(BasicBlockId::new(2));
        block2.add_predecessor(BasicBlockId::new(0));
        block2.add_predecessor(BasicBlockId::new(1));

        let blocks = vec![block0, block1, block2];

        inserter.compute_dominance_frontiers(&blocks);

        // block2는 join point이므로 block0, block1의 dominance frontier
        assert!(inserter
            .dominance_frontiers
            .get(&BasicBlockId::new(0))
            .unwrap()
            .contains(&BasicBlockId::new(2)));
        assert!(inserter
            .dominance_frontiers
            .get(&BasicBlockId::new(1))
            .unwrap()
            .contains(&BasicBlockId::new(2)));
    }

    #[test]
    fn test_ssa_renamer() {
        let mut renamer = SSARenamer::new();

        // x의 첫 번째 버전
        let x1 = renamer.push_new_value("x".to_string());
        assert_eq!(renamer.get_current_value("x"), Some(x1));

        // x의 두 번째 버전 (shadowing)
        let x2 = renamer.push_new_value("x".to_string());
        assert_eq!(renamer.get_current_value("x"), Some(x2));
        assert_ne!(x1, x2);

        // pop 후 이전 버전으로 복귀
        renamer.pop_value("x");
        assert_eq!(renamer.get_current_value("x"), Some(x1));

        renamer.pop_value("x");
        assert_eq!(renamer.get_current_value("x"), None);
    }
}
