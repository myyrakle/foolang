# SSA (Static Single Assignment) 구현

이 모듈은 foolang 컴파일러를 위한 SSA 인프라를 제공합니다.

## 개요

SSA는 각 변수가 정확히 한 번만 할당되는 중간 표현(IR) 형식입니다. 이를 통해:
- 데이터 흐름 분석이 단순해집니다
- 최적화가 용이해집니다
- 레지스터 할당이 효율적으로 이뤄집니다

## 구현된 컴포넌트

### 1. 핵심 데이터 구조 (`mod.rs`)

#### SSAValueId
- SSA 값의 고유 식별자
- 각 assignment마다 새로운 ID 생성

#### SSAValue
- SSA 값의 메타데이터 저장
- 원본 변수명, 타입, 정의 위치 포함

#### BasicBlock
- Control flow의 기본 단위
- Phi 노드, statements, predecessor/successor 관리

#### PhiNode
- 여러 control flow path의 값 병합
- Join point에서 자동 삽입

### 2. CFG 구축 (`cfg_builder.rs`)

**기능:**
- Label, Branch, Jump를 기준으로 basic block 자동 분할
- Predecessor/Successor 관계 구축
- Control flow edge 생성

**사용 예:**
```rust
use crate::ir::ssa::cfg_builder::CFGBuilder;

let blocks = CFGBuilder::build(&statements);
```

### 3. Phi 노드 삽입 (`phi_insertion.rs`)

**기능:**
- Dominance frontier 기반 Phi 노드 자동 삽입
- SSA renaming (변수 → SSA 값 변환)
- Join point 자동 감지

**알고리즘:**
1. 변수별 정의 위치 수집
2. Dominance frontier 계산
3. Join point에 Phi 노드 삽입
4. SSA renaming

**사용 예:**
```rust
use crate::ir::ssa::phi_insertion::PhiInserter;

let phi_locations = PhiInserter::insert_phi_nodes(&mut blocks, &statements);
```

### 4. Liveness Analysis (`liveness.rs`)

**기능:**
- SSA 값의 생명 주기 추적
- 정의 지점 및 마지막 사용 지점 계산
- Live-in/Live-out 집합 관리

**알고리즘:**
- Backward dataflow analysis
- Block 단위 liveness 전파

**사용 예:**
```rust
use crate::ir::ssa::liveness::LivenessAnalysis;

let analysis = LivenessAnalysis::analyze(&blocks);
if analysis.is_last_use(value_id, block_id, stmt_index) {
    // 레지스터 해제
}
```

### 5. 레지스터 할당 (`register_allocator.rs`)

**기능:**
- Linear Scan 기반 레지스터 할당
- 생명 주기 기반 레지스터 재사용
- Spill (스택으로 저장) 자동 처리

**할당 전략:**
- Callee-saved 레지스터 사용 (RBX, R12-R15)
- Live interval 만료 시 레지스터 회수
- 레지스터 부족 시 스택 사용

**사용 예:**
```rust
use crate::ir::ssa::register_allocator::RegisterAllocator;

let mut allocator = RegisterAllocator::new();
let location = allocator.allocate(value_id, block_id, stmt_index, &liveness)?;

// 사용 완료 후
allocator.free_if_last_use(value_id, block_id, stmt_index, &liveness);
```

## FunctionContext 통합

`FunctionContext`에 SSA 관련 필드와 메서드가 추가되었습니다:

### 새 필드
- `ssa_values`: SSA 값 관리
- `next_ssa_id`: 다음 SSA ID
- `variable_versions`: 변수명 → SSA 값 매핑
- `basic_blocks`: CFG의 basic blocks
- `liveness`: Liveness 분석 결과
- `register_allocator`: 레지스터 할당자

### 새 메서드
- `new_ssa_value()`: 새 SSA 값 생성
- `get_current_version()`: 변수의 현재 SSA 값 조회
- `allocate_ssa_value()`: SSA 값에 레지스터/스택 할당
- `free_ssa_value_if_last_use()`: 마지막 사용 후 레지스터 해제

## 사용 시나리오

### 1. SSA 값 생성 (Assignment 컴파일)

```rust
// Assignment: x = add 1, 2
let ssa_id = context.new_ssa_value(Some("x".to_string()), IRType::I32);

// 인스트럭션 컴파일 → RAX에 결과
compile_add_instruction(&instr, context, object)?;

// SSA 값에 레지스터 할당
let location = context.allocate_ssa_value(ssa_id)?;

// RAX → 할당된 위치로 이동
match location {
    ValueLocation::Register(reg) => {
        // mov reg, rax
    }
    ValueLocation::Spilled(offset) => {
        // mov [rbp + offset], rax
    }
}
```

### 2. SSA 값 사용 (Operand 로딩)

```rust
// Operand: 변수 참조
if let Operand::Identifier(id) = operand {
    // 현재 SSA 값 조회
    let ssa_id = context.get_current_version(&id.name)?;

    // SSA 값의 위치 조회
    let location = context.get_ssa_value_location(ssa_id)?;

    // 위치에서 값 로드
    match location {
        ValueLocation::Register(reg) => {
            // 이미 레지스터에 있음
        }
        ValueLocation::Spilled(offset) => {
            // mov target_reg, [rbp + offset]
        }
    }

    // 마지막 사용이면 레지스터 해제
    context.free_ssa_value_if_last_use(ssa_id);
}
```

### 3. CFG 구축 및 Phi 노드 삽입

```rust
// 1. CFG 구축
let mut blocks = CFGBuilder::build(&statements);

// 2. Phi 노드 삽입
let phi_locations = PhiInserter::insert_phi_nodes(&mut blocks, &statements);

// 3. Liveness 분석
let liveness = LivenessAnalysis::analyze(&blocks);

// 4. FunctionContext에 저장
context.basic_blocks = blocks;
context.liveness = Some(liveness);
```

## 성능 최적화

### 레지스터 재사용
- 5개의 callee-saved 레지스터만 사용
- 생명 주기가 끝난 값의 레지스터 즉시 회수
- 동시에 live한 값이 5개 이하면 스택 사용 없음

### Linear Scan 할당
- O(n log n) 복잡도 (n = SSA 값 개수)
- 간단하고 빠른 레지스터 할당
- 대부분의 경우 충분히 좋은 결과

## 테스트

모든 컴포넌트는 포괄적인 단위 테스트를 포함합니다:

```bash
# SSA 모듈 테스트
cargo test ssa::

# 개별 컴포넌트 테스트
cargo test cfg_builder
cargo test phi_insertion
cargo test liveness
cargo test register_allocator
```

## 향후 작업

현재 구현된 것은 SSA 인프라이며, 실제 컴파일러와의 통합이 필요합니다:

1. **Liveness 분석 완성**: Statement 레벨 use/def 추적
2. **Assignment 컴파일 재작성**: SSA 기반으로 전환
3. **Operand 로딩 개선**: SSA 값 직접 지원
4. **Phi 노드 코드 생성**: Predecessor 블록별 mov 생성
5. **최적화**: Dead code elimination, copy propagation

## 참고 자료

- Cytron et al. "Efficiently Computing Static Single Assignment Form"
- Cooper & Torczon, "Engineering a Compiler" (Chapter 9: SSA)
- LLVM SSA 구현
- Appel, "Modern Compiler Implementation" (Chapter 19)

## 라이선스

이 코드는 foolang 프로젝트의 일부이며, 프로젝트의 라이선스를 따릅니다.
