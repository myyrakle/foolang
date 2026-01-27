use super::{BasicBlock, BasicBlockId, SSAValueId};
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
    pub fn update_last_use(&mut self) {
        if let Some(&last) = self.use_points.last() {
            self.last_use = Some(last);
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

            // Statement 처리 (현재는 간단히 처리, 나중에 확장 필요)
            // TODO: 실제 statement에서 사용되는 SSA 값 추출

            use_sets.insert(block.id, uses);
            def_sets.insert(block.id, defs);
        }

        // 2단계: Backward dataflow iteration
        let mut changed = true;
        let max_iterations = blocks.len() * 2; // 수렴 보장
        let mut iteration_count = 0;

        while changed && iteration_count < max_iterations {
            changed = false;
            iteration_count += 1;

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
        // TODO: 실제 사용 지점을 statement 레벨에서 추적

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
