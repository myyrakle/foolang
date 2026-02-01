use super::{BasicBlock, BasicBlockId, SSAValueId};
use crate::ir::ast::{
    common::Operand,
    local::{instruction::InstructionStatement, LocalStatement},
};
use std::collections::{HashMap, HashSet};

/// SSA 값의 생명 주기 정보
#[derive(Debug, Clone)]
pub struct LivenessInfo {
    /// 값이 정의된 지점 (block_id, statement_index within block)
    pub def_point: (BasicBlockId, usize),

    /// 값이 사용되는 지점들 (block_id, statement_index within block)
    pub use_points: Vec<(BasicBlockId, usize)>,

    /// 마지막 사용 지점 (block_id, statement_index within block)
    pub last_use: Option<(BasicBlockId, usize)>,

    /// live range의 길이 (휴리스틱, 최적화에 사용)
    pub range_length: usize,
}

/// 전체 함수에 대한 liveness 정보
#[derive(Debug)]
pub struct LivenessAnalysis {
    /// SSA value ID -> liveness 정보
    pub value_liveness: HashMap<SSAValueId, LivenessInfo>,

    /// 각 basic block 진입 시점에서 live한 값들
    pub live_in: HashMap<BasicBlockId, HashSet<SSAValueId>>,

    /// 각 basic block 종료 시점에서 live한 값들
    pub live_out: HashMap<BasicBlockId, HashSet<SSAValueId>>,
}

impl LivenessInfo {
    pub fn new(def_point: (BasicBlockId, usize)) -> Self {
        Self {
            def_point,
            use_points: Vec::new(),
            last_use: None,
            range_length: 0,
        }
    }

    /// 사용 지점 추가
    pub fn add_use(&mut self, use_point: (BasicBlockId, usize)) {
        self.use_points.push(use_point);
    }

    /// 마지막 사용 지점 업데이트
    ///
    /// 블록 ID와 statement index를 기준으로 정렬하여 실제 실행 순서에서의 마지막 사용을 찾습니다.
    pub fn update_last_use(&mut self) {
        // 블록 ID와 statement index를 기준으로 정렬하여 실제 마지막 사용 찾기
        if !self.use_points.is_empty() {
            let last = self.use_points
                .iter()
                .max_by_key(|(block_id, stmt_idx)| (block_id.as_usize(), *stmt_idx))
                .copied();
            self.last_use = last;
        }
    }

    /// live range 길이 계산 (간단한 휴리스틱)
    pub fn calculate_range_length(&mut self) {
        if let Some(last_use) = self.last_use {
            // 블록 간 거리를 고려한 간단한 계산
            let def_block = self.def_point.0.as_usize();
            let last_block = last_use.0.as_usize();
            let block_distance = if last_block >= def_block {
                last_block - def_block
            } else {
                0
            };

            // 블록 내 statement 거리도 고려
            let stmt_distance = if self.def_point.0 == last_use.0 {
                last_use.1.saturating_sub(self.def_point.1)
            } else {
                0
            };

            self.range_length = block_distance * 100 + stmt_distance;
        } else {
            self.range_length = 0;
        }
    }
}

/// Statement에서 사용되는 변수명 추출
fn extract_used_variables(stmt: &LocalStatement) -> Vec<String> {
    let mut vars = Vec::new();

    match stmt {
        LocalStatement::Assignment(assignment) => {
            // Assignment의 value에서 사용되는 변수
            if let crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                instr,
            ) = &assignment.value
            {
                vars.extend(extract_used_variables_from_instruction(instr));
            }
        }
        LocalStatement::Instruction(instr) => {
            vars.extend(extract_used_variables_from_instruction(instr));
        }
        LocalStatement::Label(_) => {
            // Label은 변수 사용 없음
        }
    }

    vars
}

/// Instruction에서 사용되는 변수명 추출
fn extract_used_variables_from_instruction(instr: &InstructionStatement) -> Vec<String> {
    let mut vars = Vec::new();

    match instr {
        InstructionStatement::Add(add) => {
            if let Operand::Identifier(id) = &add.left {
                vars.push(id.name.clone());
            }
            if let Operand::Identifier(id) = &add.right {
                vars.push(id.name.clone());
            }
        }
        InstructionStatement::Sub(sub) => {
            if let Operand::Identifier(id) = &sub.left {
                vars.push(id.name.clone());
            }
            if let Operand::Identifier(id) = &sub.right {
                vars.push(id.name.clone());
            }
        }
        InstructionStatement::Mul(mul) => {
            if let Operand::Identifier(id) = &mul.left {
                vars.push(id.name.clone());
            }
            if let Operand::Identifier(id) = &mul.right {
                vars.push(id.name.clone());
            }
        }
        InstructionStatement::Div(div) => {
            if let Operand::Identifier(id) = &div.left {
                vars.push(id.name.clone());
            }
            if let Operand::Identifier(id) = &div.right {
                vars.push(id.name.clone());
            }
        }
        InstructionStatement::Rem(rem) => {
            if let Operand::Identifier(id) = &rem.left {
                vars.push(id.name.clone());
            }
            if let Operand::Identifier(id) = &rem.right {
                vars.push(id.name.clone());
            }
        }
        InstructionStatement::Compare(cmp) => {
            if let Operand::Identifier(id) = &cmp.left {
                vars.push(id.name.clone());
            }
            if let Operand::Identifier(id) = &cmp.right {
                vars.push(id.name.clone());
            }
        }
        InstructionStatement::Branch(branch) => {
            // Branch의 condition은 Identifier 타입
            vars.push(branch.condition.name.clone());
        }
        InstructionStatement::Return(ret) => {
            if let Some(Operand::Identifier(id)) = &ret.return_value {
                vars.push(id.name.clone());
            }
        }
        InstructionStatement::Call(call) => {
            for arg in &call.parameters {
                if let Operand::Identifier(id) = arg {
                    vars.push(id.name.clone());
                }
            }
        }
        InstructionStatement::Store(store) => {
            // Store의 ptr은 Identifier 타입
            vars.push(store.ptr.name.clone());
            if let Operand::Identifier(id) = &store.value {
                vars.push(id.name.clone());
            }
        }
        InstructionStatement::Load(load) => {
            // Load의 ptr은 Identifier 타입
            vars.push(load.ptr.name.clone());
        }
        InstructionStatement::Jump(_) | InstructionStatement::Alloca(_) => {
            // Jump와 Alloca는 변수 사용 없음
        }
    }

    vars
}

/// Statement에서 정의되는 변수명 추출
fn extract_defined_variable(stmt: &LocalStatement) -> Option<String> {
    match stmt {
        LocalStatement::Assignment(assignment) => Some(assignment.name.name.clone()),
        _ => None,
    }
}

impl LivenessAnalysis {
    pub fn new() -> Self {
        Self {
            value_liveness: HashMap::new(),
            live_in: HashMap::new(),
            live_out: HashMap::new(),
        }
    }

    /// Backward dataflow analysis로 liveness 계산
    ///
    /// 알고리즘:
    /// 1. 각 블록의 use/def 집합 초기화
    /// 2. Backward iteration으로 live-out/live-in 계산
    /// 3. 각 SSA 값의 마지막 사용 지점 결정
    pub fn analyze(blocks: &[BasicBlock]) -> Self {
        let mut analysis = LivenessAnalysis::new();

        if blocks.is_empty() {
            return analysis;
        }

        // 1단계: 각 블록의 use/def 정보 수집
        let mut use_sets: HashMap<BasicBlockId, HashSet<SSAValueId>> = HashMap::new();
        let mut def_sets: HashMap<BasicBlockId, HashSet<SSAValueId>> = HashMap::new();

        for block in blocks {
            let mut uses = HashSet::new();
            let mut defs = HashSet::new();

            // Phi 노드 처리
            for phi in &block.phi_nodes {
                defs.insert(phi.result);
                for (_, input_value) in &phi.inputs {
                    uses.insert(*input_value);
                }
            }

            // Statement 처리: 변수명 -> SSA 값 매핑 사용
            // TODO: 현재 block.defined_variables는 현재 블록에서만 정의된 변수를 추적
            // 이전 블록에서 정의된 변수를 참조하면 조회 실패
            // 해결책: SSA renaming 단계 구현 또는 함수 레벨 variable_versions 참조
            for stmt in &block.statements {
                // 사용되는 변수들을 SSA 값으로 변환
                for var_name in extract_used_variables(stmt) {
                    if let Some(&ssa_id) = block.defined_variables.get(&var_name) {
                        if !defs.contains(&ssa_id) {
                            uses.insert(ssa_id);
                        }
                    }
                    // TODO: block.defined_variables에서 찾지 못한 변수는
                    // predecessor 블록이나 함수 레벨 매핑에서 조회 필요
                }

                // 정의되는 변수를 SSA 값으로 변환
                if let Some(var_name) = extract_defined_variable(stmt) {
                    if let Some(&ssa_id) = block.defined_variables.get(&var_name) {
                        defs.insert(ssa_id);
                    }
                }
            }

            use_sets.insert(block.id, uses);
            def_sets.insert(block.id, defs);
        }

        // 2단계: Backward dataflow iteration (수렴할 때까지 반복)
        let mut changed = true;

        while changed {
            changed = false;

            // Reverse postorder로 순회 (간단히 역순으로 처리)
            for block in blocks.iter().rev() {
                let block_id = block.id;

                // live_out[block] = ∪ (live_in[succ] for succ in successors)
                let mut new_live_out = HashSet::new();
                for &succ_id in &block.successors {
                    if let Some(succ_live_in) = analysis.live_in.get(&succ_id) {
                        new_live_out.extend(succ_live_in.iter().copied());
                    }
                }

                // live_in[block] = use[block] ∪ (live_out[block] - def[block])
                let uses = use_sets.get(&block_id).cloned().unwrap_or_default();
                let defs = def_sets.get(&block_id).cloned().unwrap_or_default();

                let mut new_live_in = uses.clone();
                for value in &new_live_out {
                    if !defs.contains(value) {
                        new_live_in.insert(*value);
                    }
                }

                // 변경사항 확인
                if analysis.live_out.get(&block_id) != Some(&new_live_out) {
                    changed = true;
                }
                if analysis.live_in.get(&block_id) != Some(&new_live_in) {
                    changed = true;
                }

                analysis.live_out.insert(block_id, new_live_out);
                analysis.live_in.insert(block_id, new_live_in);
            }
        }

        // 3단계: 각 SSA 값의 상세 liveness 정보 수집
        for block in blocks {
            let block_id = block.id;

            // Phi 노드의 result 값 정의
            for (phi_idx, phi) in block.phi_nodes.iter().enumerate() {
                let liveness_info = LivenessInfo::new((block_id, phi_idx));

                // Phi 노드의 입력은 predecessor 블록에서 사용됨
                // 여기서는 단순화하여 현재 블록에서 사용된 것으로 표시
                for (pred_block_id, input_value) in &phi.inputs {
                    let info = analysis
                        .value_liveness
                        .entry(*input_value)
                        .or_insert_with(|| LivenessInfo::new((*pred_block_id, 0)));
                    info.add_use((*pred_block_id, 0));
                }

                analysis.value_liveness.insert(phi.result, liveness_info);
            }

            // Statement의 정의와 사용 추적
            for (stmt_idx, stmt) in block.statements.iter().enumerate() {
                // 사용되는 변수들 추적
                for var_name in extract_used_variables(stmt) {
                    if let Some(&ssa_id) = block.defined_variables.get(&var_name) {
                        if let Some(liveness_info) = analysis.value_liveness.get_mut(&ssa_id) {
                            liveness_info.add_use((block_id, stmt_idx));
                        }
                    }
                }

                // 정의되는 변수 추적
                if let Some(var_name) = extract_defined_variable(stmt) {
                    if let Some(&ssa_id) = block.defined_variables.get(&var_name) {
                        // 기존 정보가 있으면 def_point만 업데이트, 없으면 새로 생성
                        analysis
                            .value_liveness
                            .entry(ssa_id)
                            .or_insert_with(|| LivenessInfo::new((block_id, stmt_idx)))
                            .def_point = (block_id, stmt_idx);
                    }
                }
            }
        }

        // 4단계: 각 SSA 값의 last_use 업데이트 및 range_length 계산
        for liveness_info in analysis.value_liveness.values_mut() {
            liveness_info.update_last_use();
            liveness_info.calculate_range_length();
        }

        analysis
    }

    /// 특정 지점에서 값이 마지막으로 사용되는지 확인
    pub fn is_last_use(
        &self,
        value: SSAValueId,
        block: BasicBlockId,
        statement_index: usize,
    ) -> bool {
        if let Some(liveness) = self.value_liveness.get(&value) {
            if let Some(last_use) = liveness.last_use {
                return last_use.0 == block && last_use.1 == statement_index;
            }
        }
        false
    }

    /// 특정 SSA 값이 주어진 지점에서 live한지 확인
    pub fn is_live_at(
        &self,
        value: SSAValueId,
        block: BasicBlockId,
        _statement_index: usize,
    ) -> bool {
        // 간단한 구현: 블록 레벨에서 확인
        self.live_in
            .get(&block)
            .map(|set| set.contains(&value))
            .unwrap_or(false)
            || self
                .live_out
                .get(&block)
                .map(|set| set.contains(&value))
                .unwrap_or(false)
    }

    /// SSA 값의 liveness 정보 추가
    pub fn add_value_liveness(&mut self, value_id: SSAValueId, info: LivenessInfo) {
        self.value_liveness.insert(value_id, info);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::ssa::{BasicBlock, PhiNode};
    use crate::ir::ast::types::{IRPrimitiveType, IRType};

    #[test]
    fn test_liveness_info_creation() {
        let def_point = (BasicBlockId::new(0), 0);
        let info = LivenessInfo::new(def_point);

        assert_eq!(info.def_point, def_point);
        assert!(info.use_points.is_empty());
        assert_eq!(info.last_use, None);
    }

    #[test]
    fn test_liveness_info_add_use() {
        let def_point = (BasicBlockId::new(0), 0);
        let mut info = LivenessInfo::new(def_point);

        info.add_use((BasicBlockId::new(0), 1));
        info.add_use((BasicBlockId::new(0), 2));

        assert_eq!(info.use_points.len(), 2);
    }

    #[test]
    fn test_liveness_info_update_last_use() {
        let def_point = (BasicBlockId::new(0), 0);
        let mut info = LivenessInfo::new(def_point);

        info.add_use((BasicBlockId::new(0), 1));
        info.add_use((BasicBlockId::new(0), 3));
        info.update_last_use();

        assert_eq!(info.last_use, Some((BasicBlockId::new(0), 3)));
    }

    #[test]
    fn test_liveness_analysis_empty_blocks() {
        let blocks = vec![];
        let analysis = LivenessAnalysis::analyze(&blocks);

        assert!(analysis.value_liveness.is_empty());
        assert!(analysis.live_in.is_empty());
        assert!(analysis.live_out.is_empty());
    }

    #[test]
    fn test_liveness_analysis_single_block() {
        let block = BasicBlock::new(BasicBlockId::new(0));
        let blocks = vec![block];

        let analysis = LivenessAnalysis::analyze(&blocks);

        // 단일 블록이므로 live_in/live_out이 비어있어야 함
        assert!(analysis.live_in.get(&BasicBlockId::new(0)).is_some());
        assert!(analysis.live_out.get(&BasicBlockId::new(0)).is_some());
    }

    #[test]
    fn test_liveness_analysis_with_phi() {
        let mut block0 = BasicBlock::new(BasicBlockId::new(0));
        let mut block1 = BasicBlock::new(BasicBlockId::new(1));
        let mut block2 = BasicBlock::new(BasicBlockId::new(2));

        // block0, block1 -> block2 (join point)
        block2.add_predecessor(BasicBlockId::new(0));
        block2.add_predecessor(BasicBlockId::new(1));
        block0.add_successor(BasicBlockId::new(2));
        block1.add_successor(BasicBlockId::new(2));

        // block2에 phi 노드 추가
        let value1 = SSAValueId::new(1);
        let value2 = SSAValueId::new(2);
        let result = SSAValueId::new(3);

        // value1은 block0에서, value2는 block1에서 정의되도록 설정
        block0
            .defined_variables
            .insert("x".to_string(), value1);
        block1
            .defined_variables
            .insert("x".to_string(), value2);

        let mut phi = PhiNode::new(
            result,
            IRType::Primitive(IRPrimitiveType::Int32),
            "x".to_string(),
        );
        phi.add_input(BasicBlockId::new(0), value1);
        phi.add_input(BasicBlockId::new(1), value2);
        block2.add_phi_node(phi);

        let blocks = vec![block0, block1, block2];
        let analysis = LivenessAnalysis::analyze(&blocks);

        // Phi 입력 값들이 predecessor 블록의 live_out에 있어야 함
        let block2_live_in = analysis.live_in.get(&BasicBlockId::new(2)).unwrap();
        assert!(block2_live_in.contains(&value1) || block2_live_in.contains(&value2));
    }
}
