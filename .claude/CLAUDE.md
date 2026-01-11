# 프로젝트 가이드라인

## 기본 개요

- 자체 컴파일러를 0 dependencies로 구축하는 프로젝트입니다.
- 현재는 linux-amd64 환경만을 목표로 하고 구현하고 있습니다.
- 컴파일러 프론트엔드와 백엔드 2 계층으로 구성됩니다. 백엔드는 자체 구현 IR(Intermediate Representation)을 구현해서 사용합니다.
- 한국어로 대답하세요.

## IR 컴파일 관련 개요

- IR에 대한 전반적인 구조는 src/ir/ast 경로 아래에 정의되어있습니다.
- 개별 플랫폼의 컴파일 로직은 src/ir/compile/{platform} 경로 아래에서 관리됩니다.
- 개별 플랫폼 컴파일에 대한 테스트코드는 src/ir/compile/{platform}/mod.rs 파일 안에서 관리됩니다.
- 특정 OS, 특정 CPU에 종속적인 공통 로직/상수 등은 src/platforms 경로 아래에서 정리하고 관리됩니다.
