use super::{BasicBlock, BasicBlockId};
use crate::ir::ast::local::{instruction::InstructionStatement, LocalStatement};
use std::collections::HashMap;

/// CFG (Control Flow Graph) 구축기
pub struct CFGBuilder {
    /// 현재까지 생성된 basic blocks
    blocks: Vec<BasicBlock>,

    /// 라벨 이름 -> basic block ID 매핑
    label_to_block: HashMap<String, BasicBlockId>,

    /// 다음 basic block ID
    next_block_id: usize,

    /// 다음 SSA 값 ID (임시: renaming 전에 고유 ID 부여용)
    next_ssa_id: usize,
}

impl CFGBuilder {
    pub fn new() -> Self {
        Self {
            blocks: Vec::new(),
            label_to_block: HashMap::new(),
            next_block_id: 0,
            next_ssa_id: 0,
        }
    }

    /// Statement 리스트로부터 CFG 구축
    ///
    /// # 알고리즘:
    /// 1. Label과 Branch/Jump를 기준으로 basic block 분할
    /// 2. 각 블록에 statement들 할당
    /// 3. Predecessor/Successor 관계 구축
    pub fn build(statements: &[LocalStatement]) -> Vec<BasicBlock> {
        let mut builder = CFGBuilder::new();

        if statements.is_empty() {
            // 빈 함수면 단일 블록 반환
            return vec![BasicBlock::new(BasicBlockId::new(0))];
        }

        // 1단계: 블록 경계 지점 찾기
        let block_boundaries = builder.find_block_boundaries(statements);

        // 2단계: Basic blocks 생성 및 statement 할당
        builder.create_blocks(statements, &block_boundaries);

        // 3단계: Control flow edges 구축
        builder.build_edges(statements, &block_boundaries);

        builder.blocks
    }

    /// 블록 경계 지점 찾기
    ///
    /// 다음 위치가 새 블록의 시작점:
    /// - 함수 시작 (index 0)
    /// - Label 정의
    /// - Branch/Jump 다음 statement
    fn find_block_boundaries(&mut self, statements: &[LocalStatement]) -> Vec<usize> {
        let mut boundaries = vec![0]; // 함수 시작은 항상 블록 시작

        // 1단계: 블록 경계 수집
        for (i, stmt) in statements.iter().enumerate() {
            match stmt {
                // Label은 새 블록의 시작
                LocalStatement::Label(_) => {
                    if !boundaries.contains(&i) {
                        boundaries.push(i);
                    }
                }
                // Branch/Jump 다음은 새 블록의 시작
                LocalStatement::Instruction(InstructionStatement::Branch(_))
                | LocalStatement::Instruction(InstructionStatement::Jump(_))
                | LocalStatement::Instruction(InstructionStatement::Return(_)) => {
                    if i + 1 < statements.len() && !boundaries.contains(&(i + 1)) {
                        boundaries.push(i + 1);
                    }
                }
                _ => {}
            }
        }

        // 2단계: 정렬
        boundaries.sort_unstable();

        // 3단계: 라벨 매핑 (정렬 후)
        for (i, stmt) in statements.iter().enumerate() {
            if let LocalStatement::Label(label_def) = stmt {
                // 정렬된 boundaries에서 이 라벨의 블록 인덱스 찾기
                if let Some(block_idx) = boundaries.iter().position(|&b| b == i) {
                    let block_id = BasicBlockId::new(block_idx);
                    self.label_to_block
                        .insert(label_def.name.name.clone(), block_id);
                }
            }
        }

        boundaries
    }

    /// Basic blocks 생성 및 statement 인덱스 범위 저장
    fn create_blocks(&mut self, statements: &[LocalStatement], boundaries: &[usize]) {
        for (block_idx, &start) in boundaries.iter().enumerate() {
            let end = boundaries
                .get(block_idx + 1)
                .copied()
                .unwrap_or(statements.len());

            let block_id = BasicBlockId::new(block_idx);
            let mut block = BasicBlock::new(block_id);

            // 블록의 statement 저장 (Phase 15: SSA/liveness 분석을 위해 필수)
            block
                .statements
                .extend(statements[start..end].iter().cloned());

            // Phi 노드 삽입을 위해 이 블록의 변수 정의를 수집
            // TODO: SSA renaming 단계 미구현 (블록 간 변수 버전 관리 필요)
            // 문제: liveness 분석 시 이전 블록 변수 참조 불가
            // 해결책: SSA renaming 구현 (변수 버전 관리, 전역 SSA 값 매핑)
            //
            // 현재: 각 assignment에 고유 SSA ID 부여 (블록 내에서만 유효)
            for stmt in &statements[start..end] {
                if let LocalStatement::Assignment(assignment) = stmt {
                    let var_name = assignment.name.name.clone();
                    // 고유한 SSAValueId 생성
                    let ssa_id = crate::ir::ssa::SSAValueId::new(self.next_ssa_id);
                    self.next_ssa_id += 1;
                    block.defined_variables.insert(var_name, ssa_id);
                }
            }

            self.blocks.push(block);
        }

        self.next_block_id = self.blocks.len();
    }

    /// Control flow edges 구축
    fn build_edges(&mut self, statements: &[LocalStatement], boundaries: &[usize]) {
        for (block_idx, &start) in boundaries.iter().enumerate() {
            let end = boundaries
                .get(block_idx + 1)
                .copied()
                .unwrap_or(statements.len());

            let block_id = BasicBlockId::new(block_idx);

            // 블록의 마지막 statement 확인
            let last_stmt = statements[start..end].iter().rev().find(|stmt| {
                !matches!(stmt, LocalStatement::Label(_))
            });

            match last_stmt {
                Some(LocalStatement::Instruction(InstructionStatement::Branch(branch))) => {
                    // Conditional branch: 두 개의 successor
                    if let Some(&true_block) =
                        self.label_to_block.get(&branch.true_label.name)
                    {
                        self.add_edge(block_id, true_block);
                    }
                    if let Some(&false_block) =
                        self.label_to_block.get(&branch.false_label.name)
                    {
                        self.add_edge(block_id, false_block);
                    }
                }
                Some(LocalStatement::Instruction(InstructionStatement::Jump(jump))) => {
                    // Unconditional jump: 한 개의 successor
                    if let Some(&target_block) = self.label_to_block.get(&jump.label.name) {
                        self.add_edge(block_id, target_block);
                    }
                }
                Some(LocalStatement::Instruction(InstructionStatement::Return(_))) => {
                    // Return: successor 없음
                }
                _ => {
                    // Fall-through: 다음 블록이 successor
                    if block_idx + 1 < self.blocks.len() {
                        let next_block = BasicBlockId::new(block_idx + 1);
                        self.add_edge(block_id, next_block);
                    }
                }
            }
        }
    }

    /// Edge 추가 (from -> to)
    fn add_edge(&mut self, from: BasicBlockId, to: BasicBlockId) {
        let from_idx = from.as_usize();
        let to_idx = to.as_usize();

        if from_idx < self.blocks.len() && to_idx < self.blocks.len() {
            self.blocks[from_idx].add_successor(to);
            self.blocks[to_idx].add_predecessor(from);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::ast::{
        common::{Identifier, Label},
        local::{instruction::branch::JumpInstruction, label::LabelDefinition},
    };

    #[test]
    fn test_single_block() {
        // 빈 함수
        let statements = vec![];
        let blocks = CFGBuilder::build(&statements);

        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].id.as_usize(), 0);
        assert!(blocks[0].predecessors.is_empty());
        assert!(blocks[0].successors.is_empty());
    }

    #[test]
    fn test_linear_blocks() {
        // Label 없는 직선 코드: 여러 assignment가 하나의 블록으로 병합
        let statements = vec![
            LocalStatement::Assignment(
                crate::ir::ast::local::assignment::AssignmentStatement {
                    name: Identifier {
                        type_: crate::ir::ast::types::IRType::None,
                        name: "x".to_string(),
                    },
                    value: crate::ir::ast::local::assignment::AssignmentStatementValue::Literal(
                        crate::ir::ast::common::literal::LiteralValue::Int32(1),
                    ),
                },
            ),
            LocalStatement::Assignment(
                crate::ir::ast::local::assignment::AssignmentStatement {
                    name: Identifier {
                        type_: crate::ir::ast::types::IRType::None,
                        name: "y".to_string(),
                    },
                    value: crate::ir::ast::local::assignment::AssignmentStatementValue::Literal(
                        crate::ir::ast::common::literal::LiteralValue::Int32(2),
                    ),
                },
            ),
            LocalStatement::Assignment(
                crate::ir::ast::local::assignment::AssignmentStatement {
                    name: Identifier {
                        type_: crate::ir::ast::types::IRType::None,
                        name: "z".to_string(),
                    },
                    value: crate::ir::ast::local::assignment::AssignmentStatementValue::Literal(
                        crate::ir::ast::common::literal::LiteralValue::Int32(3),
                    ),
                },
            ),
        ];

        let blocks = CFGBuilder::build(&statements);

        // Label이나 분기가 없으므로 단일 블록으로 병합되어야 함
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].id.as_usize(), 0);

        // Predecessor/Successor 없음 (직선 코드, 분기 없음)
        assert!(blocks[0].predecessors.is_empty());
        assert!(blocks[0].successors.is_empty());
    }

    #[test]
    fn test_jump_creates_blocks() {
        // jump target
        // ...
        // target:
        let statements = vec![
            LocalStatement::Instruction(InstructionStatement::Jump(JumpInstruction {
                label: Label {
                    name: "target".to_string(),
                },
            })),
            LocalStatement::Label(LabelDefinition {
                name: Identifier {
                    type_: crate::ir::ast::types::IRType::None,
                    name: "target".to_string(),
                },
            }),
        ];

        let blocks = CFGBuilder::build(&statements);

        // 2개 블록: jump 전과 target 라벨
        assert_eq!(blocks.len(), 2);

        // 첫 번째 블록 -> 두 번째 블록
        assert_eq!(blocks[0].successors.len(), 1);
        assert_eq!(blocks[0].successors[0], BasicBlockId::new(1));
        assert_eq!(blocks[1].predecessors.len(), 1);
        assert_eq!(blocks[1].predecessors[0], BasicBlockId::new(0));
    }

    #[test]
    fn test_label_creates_block() {
        // start:
        // label1:
        let statements = vec![
            LocalStatement::Label(LabelDefinition {
                name: Identifier {
                    type_: crate::ir::ast::types::IRType::None,
                    name: "start".to_string(),
                },
            }),
            LocalStatement::Label(LabelDefinition {
                name: Identifier {
                    type_: crate::ir::ast::types::IRType::None,
                    name: "label1".to_string(),
                },
            }),
        ];

        let blocks = CFGBuilder::build(&statements);

        // 각 라벨이 새 블록 시작
        assert_eq!(blocks.len(), 2);
    }
}
