use super::{liveness::LivenessAnalysis, BasicBlockId, SSAValueId};
use crate::{ir::error::IRError, platforms::amd64::register::Register};
use std::collections::{HashMap, HashSet};

/// SSA 값의 저장 위치
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueLocation {
    /// 레지스터에 할당됨
    Register(Register),

    /// 스택에 spill됨 (RBP 기준 오프셋, 음수)
    Spilled(i32),

    /// 아직 할당되지 않음
    Unassigned,
}

/// Live interval: SSA 값이 살아있는 구간
#[derive(Debug, Clone)]
pub struct LiveInterval {
    /// SSA 값 ID
    pub value_id: SSAValueId,

    /// 시작 지점 (block_id, statement_index)
    pub start: (BasicBlockId, usize),

    /// 종료 지점 (block_id, statement_index)
    pub end: (BasicBlockId, usize),

    /// 할당된 레지스터 (있다면)
    pub assigned_register: Option<Register>,
}

/// Linear Scan 기반 레지스터 할당자
#[derive(Debug)]
pub struct RegisterAllocator {
    /// 사용 가능한 레지스터 풀 (callee-saved: RBX, R12-R15)
    pub available_registers: Vec<Register>,

    /// 현재 활성화된 live interval들
    pub active_intervals: Vec<LiveInterval>,

    /// SSA value -> 할당된 위치
    pub allocation_map: HashMap<SSAValueId, ValueLocation>,

    /// 스택 오프셋 (spill용, RBP 기준 음수)
    pub stack_offset: i32,

    /// spill된 값들의 목록 (복원용)
    pub spilled_values: Vec<SSAValueId>,

    /// 한 번이라도 사용된 callee-saved 레지스터들
    /// prologue/epilogue에서 저장/복원해야 하는 레지스터 추적
    pub ever_used_callee_saved: HashSet<Register>,
}

impl LiveInterval {
    pub fn new(
        value_id: SSAValueId,
        start: (BasicBlockId, usize),
        end: (BasicBlockId, usize),
    ) -> Self {
        Self {
            value_id,
            start,
            end,
            assigned_register: None,
        }
    }

    /// 이 interval이 주어진 지점에서 active한지 확인
    /// CFG 순서를 고려한 비교를 위해 LivenessAnalysis를 사용
    pub fn is_active_at(
        &self,
        block: BasicBlockId,
        statement_index: usize,
        liveness: &LivenessAnalysis,
    ) -> bool {
        let point = (block, statement_index);

        // CFG 순서를 고려한 비교
        let after_start = liveness.compare_points(point, self.start) >= std::cmp::Ordering::Equal;
        let before_end = liveness.compare_points(point, self.end) <= std::cmp::Ordering::Equal;

        after_start && before_end
    }
}

impl RegisterAllocator {
    /// 새 레지스터 할당자 생성
    ///
    /// # Arguments
    /// * `initial_stack_offset` - 스택 오프셋의 시작 지점 (FunctionContext의 현재 stack_offset)
    ///                            이미 사용된 스택 공간을 피하기 위해 필요
    pub fn new(initial_stack_offset: i32) -> Self {
        Self {
            available_registers: vec![
                Register::RBX,
                Register::R12,
                Register::R13,
                Register::R14,
                Register::R15,
            ],
            active_intervals: Vec::new(),
            allocation_map: HashMap::new(),
            stack_offset: initial_stack_offset,
            spilled_values: Vec::new(),
            ever_used_callee_saved: HashSet::new(),
        }
    }

    /// 만료된 interval들의 레지스터 회수
    ///
    /// 주어진 시점에서 더 이상 활성화되지 않은 interval들을 제거하고
    /// 그들의 레지스터를 available_registers에 반환
    pub fn expire_old_intervals(
        &mut self,
        block: BasicBlockId,
        statement_index: usize,
        liveness: &LivenessAnalysis,
    ) {
        // 종료된 interval들 찾기
        let mut expired = Vec::new();

        for (idx, interval) in self.active_intervals.iter().enumerate() {
            if !interval.is_active_at(block, statement_index, liveness) {
                expired.push(idx);
            }
        }

        // 역순으로 제거 (인덱스 유지)
        for &idx in expired.iter().rev() {
            let interval = self.active_intervals.remove(idx);

            // 레지스터가 할당되어 있었다면 회수
            if let Some(reg) = interval.assigned_register {
                self.available_registers.push(reg);
            }
        }
    }

    /// 레지스터 할당
    ///
    /// 주어진 SSA 값에 대해 레지스터를 할당하거나 spill 수행
    pub fn allocate(
        &mut self,
        value_id: SSAValueId,
        block: BasicBlockId,
        statement_index: usize,
        liveness: &LivenessAnalysis,
    ) -> Result<ValueLocation, IRError> {
        // 이미 할당되어 있으면 반환
        if let Some(existing) = self.allocation_map.get(&value_id) {
            return Ok(existing.clone());
        }

        // 만료된 interval 회수
        self.expire_old_intervals(block, statement_index, liveness);

        // 사용 가능한 레지스터가 있으면 할당
        if let Some(reg) = self.available_registers.pop() {
            let location = ValueLocation::Register(reg);
            self.allocation_map.insert(value_id, location.clone());

            // 한 번이라도 사용된 레지스터로 기록 (prologue/epilogue용)
            self.ever_used_callee_saved.insert(reg);

            // Live interval 생성 및 추가
            if let Some(liveness_info) = liveness.value_liveness.get(&value_id) {
                let start = liveness_info.def_point;
                let end = liveness_info.last_use.unwrap_or(start);

                let mut interval = LiveInterval::new(value_id, start, end);
                interval.assigned_register = Some(reg);
                self.active_intervals.push(interval);
            }

            return Ok(location);
        }

        // 레지스터가 없으면 spill
        self.spill(value_id, liveness)
    }

    /// Spill: 레지스터 부족 시 스택에 값 저장
    ///
    /// 가장 먼 미래에 사용될 값을 spill하거나,
    /// 현재 할당하려는 값을 spill
    fn spill(
        &mut self,
        value_id: SSAValueId,
        _liveness: &LivenessAnalysis,
    ) -> Result<ValueLocation, IRError> {
        // 간단한 전략: 현재 값을 스택에 할당
        self.stack_offset -= 8; // 8바이트 (64비트 값)
        let location = ValueLocation::Spilled(self.stack_offset);

        self.allocation_map.insert(value_id, location.clone());
        self.spilled_values.push(value_id);

        Ok(location)
    }

    /// 마지막 사용 후 레지스터 해제
    pub fn free_if_last_use(
        &mut self,
        value_id: SSAValueId,
        block: BasicBlockId,
        statement_index: usize,
        liveness: &LivenessAnalysis,
    ) {
        if liveness.is_last_use(value_id, block, statement_index) {
            // allocation_map에서 제거
            if let Some(ValueLocation::Register(reg)) = self.allocation_map.remove(&value_id) {
                // 레지스터를 available pool에 반환
                self.available_registers.push(reg);

                // active_intervals에서도 제거
                self.active_intervals
                    .retain(|interval| interval.value_id != value_id);
            }
        }
    }

    /// SSA 값의 위치 조회
    pub fn get_location(&self, value_id: SSAValueId) -> Option<&ValueLocation> {
        self.allocation_map.get(&value_id)
    }

    /// 사용된 callee-saved 레지스터 목록 반환
    ///
    /// 한 번이라도 할당된 적이 있는 레지스터를 반환합니다.
    /// prologue/epilogue에서 저장/복원할 레지스터를 결정하는 데 사용됩니다.
    pub fn used_callee_saved_registers(&self) -> Vec<Register> {
        self.ever_used_callee_saved.iter().copied().collect()
    }

    /// spill된 값들이 사용하는 총 스택 크기 반환
    /// stack_offset이 음수이므로 절댓값을 반환
    pub fn get_spill_stack_size(&self) -> i32 {
        if self.stack_offset >= 0 {
            0
        } else {
            -self.stack_offset
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::ssa::liveness::LivenessInfo;

    #[test]
    fn test_register_allocator_creation() {
        let allocator = RegisterAllocator::new(0);

        assert_eq!(allocator.available_registers.len(), 5);
        assert!(allocator.active_intervals.is_empty());
        assert!(allocator.allocation_map.is_empty());
        assert_eq!(allocator.stack_offset, 0);
    }

    #[test]
    fn test_allocate_register() {
        let mut allocator = RegisterAllocator::new(0);
        let mut liveness = LivenessAnalysis::new();

        let value_id = SSAValueId::new(1);
        let block = BasicBlockId::new(0);

        // Liveness 정보 추가
        let mut info = LivenessInfo::new((block, 0));
        info.add_use((block, 1));
        info.update_last_use();
        liveness.add_value_liveness(value_id, info);

        let location = allocator.allocate(value_id, block, 0, &liveness).unwrap();

        // 레지스터가 할당되어야 함
        assert!(matches!(location, ValueLocation::Register(_)));
        assert_eq!(allocator.available_registers.len(), 4); // 하나 사용됨
    }

    #[test]
    fn test_allocate_multiple_values() {
        let mut allocator = RegisterAllocator::new(0);
        let mut liveness = LivenessAnalysis::new();

        let block = BasicBlockId::new(0);

        // 5개 값 할당 (사용 가능한 레지스터 수)
        // 모든 값이 오래 사용되도록 설정
        for i in 0..5 {
            let value_id = SSAValueId::new(i);
            let mut info = LivenessInfo::new((block, i));
            info.add_use((block, i + 100)); // 오래 사용됨
            info.update_last_use();
            liveness.add_value_liveness(value_id, info);

            let location = allocator.allocate(value_id, block, i, &liveness).unwrap();
            assert!(matches!(location, ValueLocation::Register(_)));
        }

        assert_eq!(allocator.available_registers.len(), 0); // 모두 사용됨
    }

    #[test]
    fn test_spill_when_no_registers() {
        let mut allocator = RegisterAllocator::new(0);
        let mut liveness = LivenessAnalysis::new();

        let block = BasicBlockId::new(0);

        // 5개 레지스터 모두 소진
        for i in 0..5 {
            let value_id = SSAValueId::new(i);
            let mut info = LivenessInfo::new((block, i));
            info.add_use((block, i + 10)); // 오래 사용됨
            info.update_last_use();
            liveness.add_value_liveness(value_id, info);

            allocator.allocate(value_id, block, i, &liveness).unwrap();
        }

        // 6번째 값 할당 시도 -> spill되어야 함
        let value_id = SSAValueId::new(100);
        let mut info = LivenessInfo::new((block, 5));
        info.add_use((block, 6));
        info.update_last_use();
        liveness.add_value_liveness(value_id, info);

        let location = allocator.allocate(value_id, block, 5, &liveness).unwrap();

        assert!(matches!(location, ValueLocation::Spilled(_)));
        assert_eq!(allocator.spilled_values.len(), 1);
    }

    #[test]
    fn test_free_if_last_use() {
        let mut allocator = RegisterAllocator::new(0);
        let mut liveness = LivenessAnalysis::new();

        let value_id = SSAValueId::new(1);
        let block = BasicBlockId::new(0);

        // Liveness 정보: 0에서 정의, 2에서 마지막 사용
        let mut info = LivenessInfo::new((block, 0));
        info.add_use((block, 1));
        info.add_use((block, 2));
        info.update_last_use();
        liveness.add_value_liveness(value_id, info);

        // 레지스터 할당
        allocator.allocate(value_id, block, 0, &liveness).unwrap();
        assert_eq!(allocator.available_registers.len(), 4);

        // 마지막 사용 시점에 해제
        allocator.free_if_last_use(value_id, block, 2, &liveness);
        assert_eq!(allocator.available_registers.len(), 5); // 반환됨
    }

    #[test]
    fn test_live_interval_is_active() {
        use crate::ir::ssa::liveness::LivenessAnalysis;
        use crate::ir::ssa::BasicBlock;

        // 간단한 liveness 분석 객체 생성
        let blocks = vec![BasicBlock::new(BasicBlockId::new(0))];
        let liveness = LivenessAnalysis::analyze(&blocks);

        let interval = LiveInterval::new(
            SSAValueId::new(1),
            (BasicBlockId::new(0), 5),
            (BasicBlockId::new(0), 10),
        );

        assert!(!interval.is_active_at(BasicBlockId::new(0), 4, &liveness)); // 시작 전
        assert!(interval.is_active_at(BasicBlockId::new(0), 5, &liveness)); // 시작 지점
        assert!(interval.is_active_at(BasicBlockId::new(0), 7, &liveness)); // 중간
        assert!(interval.is_active_at(BasicBlockId::new(0), 10, &liveness)); // 종료 지점
        assert!(!interval.is_active_at(BasicBlockId::new(0), 11, &liveness)); // 종료 후
    }
}
