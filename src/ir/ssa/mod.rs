use crate::ir::ast::{local::LocalStatement, types::IRType};
use std::collections::HashMap;

pub mod liveness;
pub mod register_allocator;

/// SSA 값의 고유 식별자
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SSAValueId(pub usize);

/// SSA 값 정의
#[derive(Debug, Clone)]
pub struct SSAValue {
    /// 고유 ID
    pub id: SSAValueId,

    /// 원본 변수 이름 (디버깅용, optional)
    pub original_name: Option<String>,

    /// 값의 타입
    pub type_: IRType,

    /// 정의된 위치 (basic block ID)
    pub def_block: BasicBlockId,
}

/// Basic Block 식별자
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BasicBlockId(pub usize);

/// Basic Block 정의
#[derive(Debug)]
pub struct BasicBlock {
    /// Block ID
    pub id: BasicBlockId,

    /// Phi 노드들 (블록 시작 부분)
    pub phi_nodes: Vec<PhiNode>,

    /// 일반 statement들
    pub statements: Vec<LocalStatement>,

    /// Predecessor blocks
    pub predecessors: Vec<BasicBlockId>,

    /// Successor blocks
    pub successors: Vec<BasicBlockId>,

    /// 이 블록에서 정의된 변수들 (변수명 -> SSA value ID)
    pub defined_variables: HashMap<String, SSAValueId>,
}

/// Phi 노드: 여러 predecessor에서 오는 값 병합
#[derive(Debug, Clone)]
pub struct PhiNode {
    /// 결과 SSA 값
    pub result: SSAValueId,

    /// 결과 타입
    pub result_type: IRType,

    /// 원본 변수 이름
    pub original_name: String,

    /// 입력: (predecessor block ID, SSA value ID) 쌍들
    pub inputs: Vec<(BasicBlockId, SSAValueId)>,
}

impl SSAValueId {
    pub fn new(id: usize) -> Self {
        SSAValueId(id)
    }

    pub fn as_usize(&self) -> usize {
        self.0
    }
}

impl BasicBlockId {
    pub fn new(id: usize) -> Self {
        BasicBlockId(id)
    }

    pub fn as_usize(&self) -> usize {
        self.0
    }
}

impl BasicBlock {
    pub fn new(id: BasicBlockId) -> Self {
        Self {
            id,
            phi_nodes: Vec::new(),
            statements: Vec::new(),
            predecessors: Vec::new(),
            successors: Vec::new(),
            defined_variables: HashMap::new(),
        }
    }

    /// 이 블록에 phi 노드 추가
    pub fn add_phi_node(&mut self, phi: PhiNode) {
        self.phi_nodes.push(phi);
    }

    /// 이 블록에 statement 추가
    pub fn add_statement(&mut self, stmt: LocalStatement) {
        self.statements.push(stmt);
    }

    /// Predecessor 추가
    pub fn add_predecessor(&mut self, pred: BasicBlockId) {
        if !self.predecessors.contains(&pred) {
            self.predecessors.push(pred);
        }
    }

    /// Successor 추가
    pub fn add_successor(&mut self, succ: BasicBlockId) {
        if !self.successors.contains(&succ) {
            self.successors.push(succ);
        }
    }
}

impl PhiNode {
    pub fn new(result: SSAValueId, result_type: IRType, original_name: String) -> Self {
        Self {
            result,
            result_type,
            original_name,
            inputs: Vec::new(),
        }
    }

    /// 입력 추가
    pub fn add_input(&mut self, from_block: BasicBlockId, value: SSAValueId) {
        self.inputs.push((from_block, value));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::ast::types::{IRPrimitiveType, IRType};

    #[test]
    fn test_ssa_value_id_creation() {
        let id1 = SSAValueId::new(0);
        let id2 = SSAValueId::new(1);

        assert_eq!(id1.as_usize(), 0);
        assert_eq!(id2.as_usize(), 1);
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_basic_block_creation() {
        let block_id = BasicBlockId::new(0);
        let block = BasicBlock::new(block_id);

        assert_eq!(block.id.as_usize(), 0);
        assert!(block.phi_nodes.is_empty());
        assert!(block.statements.is_empty());
        assert!(block.predecessors.is_empty());
        assert!(block.successors.is_empty());
    }

    #[test]
    fn test_basic_block_add_predecessor_successor() {
        let mut block = BasicBlock::new(BasicBlockId::new(1));
        let pred = BasicBlockId::new(0);
        let succ = BasicBlockId::new(2);

        block.add_predecessor(pred);
        block.add_successor(succ);

        assert_eq!(block.predecessors.len(), 1);
        assert_eq!(block.successors.len(), 1);
        assert_eq!(block.predecessors[0], pred);
        assert_eq!(block.successors[0], succ);

        // 중복 추가는 무시됨
        block.add_predecessor(pred);
        assert_eq!(block.predecessors.len(), 1);
    }

    #[test]
    fn test_phi_node_creation() {
        let result_id = SSAValueId::new(10);
        let phi = PhiNode::new(
            result_id,
            IRType::Primitive(IRPrimitiveType::Int32),
            "x".to_string(),
        );

        assert_eq!(phi.result, result_id);
        assert_eq!(phi.original_name, "x");
        assert!(phi.inputs.is_empty());
    }

    #[test]
    fn test_phi_node_add_inputs() {
        let result_id = SSAValueId::new(10);
        let mut phi = PhiNode::new(
            result_id,
            IRType::Primitive(IRPrimitiveType::Int32),
            "x".to_string(),
        );

        let block1 = BasicBlockId::new(0);
        let block2 = BasicBlockId::new(1);
        let value1 = SSAValueId::new(5);
        let value2 = SSAValueId::new(7);

        phi.add_input(block1, value1);
        phi.add_input(block2, value2);

        assert_eq!(phi.inputs.len(), 2);
        assert_eq!(phi.inputs[0], (block1, value1));
        assert_eq!(phi.inputs[1], (block2, value2));
    }
}
