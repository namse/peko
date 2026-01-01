# Forte CLI 구현 진행 상황

## 완료된 작업

### 1. 프로젝트 구조 리팩토링 ✅

CLI와 서버 로직 분리 완료.

```
forte/src/
├── main.rs           # CLI 진입점 (clap)
├── cli/
│   ├── mod.rs        # 명령어 정의 (dev, init, add, build)
│   └── dev.rs        # forte dev 구현
└── server/
    ├── mod.rs        # SSR 서버 로직
    └── cache.rs      # SimpleCache (WASM/JS 캐싱)
```

**구현된 CLI 명령어:**
- `forte dev` - 개발 서버 시작
- `forte init` - (스텁만)
- `forte add page/action` - (스텁만)
- `forte build` - (스텁만)

### 2. forte dev 기본 구현 (진행 중)

**완료:**
- 포트 자동 선택 (3000부터 시작, 사용 중이면 다음 포트)
- `--port` 옵션 지정 시 해당 포트만 시도, 실패하면 exit(1)
- SSR 서버 시작 및 요청 처리

**구현됨 (테스트 필요):**
- `forte-rs-to-ts` 호출 로직 (codegen)
- `cargo build --release` 호출 (백엔드 빌드)
- `npm run build` 호출 (프론트엔드 빌드)

**중단 지점:**
- `forte-rs-to-ts`가 PATH에 없어서 실행 불가
- 경로 해결 방법 결정 필요:
  1. 상대 경로로 찾기
  2. 환경 변수
  3. cargo install 안내

## 미완료 작업

PLAN.md의 구현 순서 참고:

3. 프론트엔드 라우터 자동 생성 (`routes.generated.ts`)
4. Watch 모드 + 핫 스왑
5. `forte init`
6. `forte add page`
7. `forte add action` + Action codegen
8. 정적 에셋 서빙
9. `forte build`

## 다음 단계

1. `forte-rs-to-ts` 경로 문제 해결
2. 전체 `forte dev` 플로우 테스트
3. 프론트엔드 라우터 자동 생성 구현
