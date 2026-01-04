# Forte CLI 구현 진행 상황

## 완료된 작업

### 1. 프로젝트 구조 리팩토링 ✅

CLI와 서버 로직 분리 완료.

```
forte/src/
├── main.rs           # CLI 진입점 (clap)
├── cli/
│   ├── mod.rs        # 명령어 정의 (dev, init, add, build)
│   ├── dev.rs        # forte dev 구현
│   └── init.rs       # forte init 구현
└── server/
    ├── mod.rs        # SSR 서버 로직
    └── cache.rs      # SimpleCache (WASM/JS 캐싱)
```

### 2. forte init ✅

- 프로젝트 구조 생성 (rs/, fe/)
- Forte.toml, Cargo.toml, package.json 등 설정 파일 생성
- 기본 페이지 템플릿 생성 (index 페이지)
- E2E 테스트 통과 (3개)

### 3. forte dev ✅

- 포트 자동 선택 (3000부터 시작, 사용 중이면 다음 포트)
- `--port` 옵션 지정 시 해당 포트만 시도
- `forte-rs-to-ts` 호출 (Props 타입 생성)
- `cargo build --release` 호출 (백엔드 WASM 빌드)
- `npm run build` 호출 (프론트엔드 빌드)
- SSR 서버 시작 및 요청 처리
- E2E 테스트 통과 (2개)

### 4. 프론트엔드 라우터 자동 생성 ✅

- `rs/src/pages/` 디렉토리 스캔
- `pub async fn handler` 함수가 있는 파일만 라우트로 등록
- `fe/src/routes.generated.ts` 자동 생성
- 동적 라우트 지원 (`[id]` → `:id`)
- `server.tsx`에서 `matchRoute()` 함수로 라우팅 처리

### 5. forte add page ✅

- 현재 디렉토리가 Forte 프로젝트인지 확인 (Forte.toml 체크)
- 백엔드 페이지 생성 (`rs/src/pages/<path>/mod.rs`)
- 프론트엔드 페이지 생성 (`fe/src/pages/<path>/page.tsx`)
- 중첩 경로 지원 (`product/detail` → `product/detail/mod.rs`)
- 동적 라우트 지원 (`product/[id]` → `Params` 구조체 포함)
- E2E 테스트 통과 (5개)

### 6. Watch 모드 + 핫 스왑 ✅

- `notify` + `notify-debouncer-mini` 크레이트 사용
- `rs/src` 및 `fe/src` 디렉토리 감시
- 파일 변경 시 자동 재빌드 (500ms 디바운스)
- 캐시 무효화를 통한 핫 스왑 (서버 재시작 없음)
- 백엔드/프론트엔드 개별 재빌드 지원

### 7. forte add action ✅

- 백엔드 액션 파일 생성 (`rs/src/actions/<path>.rs`)
- 프론트엔드 클라이언트 생성 (`fe/src/actions/<path>.ts`)
- Input/Output 타입 자동 생성
- `/_action/<path>` 엔드포인트로 fetch 호출
- E2E 테스트 통과 (4개)

### 8. 정적 에셋 서빙 ✅

- `/public/*` 경로로 정적 파일 서빙
- `/favicon.ico` 자동 처리
- MIME 타입 자동 감지 (이미지, CSS, JS, 폰트 등)
- 캐시 헤더 설정 (`Cache-Control: public, max-age=3600`)
- 경로 탈출 공격 방지 (path traversal protection)
- `fe/public/` 디렉토리 자동 생성 (init)

### 9. forte build ✅

- 프로덕션 빌드 명령어
- codegen 실행 (forte-rs-to-ts, routes.generated.ts)
- 백엔드 WASM 빌드 (release 모드)
- 프론트엔드 빌드 (npm run build)
- `dist/` 디렉토리에 배포용 파일 생성
  - `backend.wasm`
  - `server.js`
  - `public/` (정적 파일 복사)
- E2E 테스트 통과 (2개)

### 10. Hydration 지원 ✅

클라이언트 사이드 React 마운팅 구현 완료.

- `fe/src/client.tsx` 템플릿 추가 (`hydrateRoot` 사용)
- `rolldown.config.ts` 배열로 변경 (server.js + client.js 이중 빌드)
- `server.tsx`에서 `window.__FORTE_PROPS__` 직렬화 + `<script>` 태그 추가
- XSS 방지를 위한 `escapeJsonForScript()` 함수 추가
- `forte dev`: 빌드 후 `fe/dist/client.js` → `fe/public/client.js` 복사
- `forte build`: `dist/public/client.js` 복사
- E2E 테스트 통과

## 다음 단계 (2차)

---

### 11. 프로덕션 에셋 해싱 ✅

캐시 무효화를 위한 파일명 해싱 구현 완료.

- `std::hash::DefaultHasher` 사용 (외부 의존성 없음)
- `client.js` → `client.{hash}.js` 변환
- `server.js` 내 경로 자동 치환
- `dist/public/manifest.json` 생성
- E2E 테스트 통과

---

### 12. 클라이언트 HMR (Hot Module Replacement) ✅

개발 중 파일 변경 시 브라우저 자동 새로고침 (LiveReload) 구현 완료.

**1단계: LiveReload 구현 완료**

- `tokio-tungstenite`를 사용한 WebSocket 서버
- `/__hmr` 엔드포인트에서 WebSocket 업그레이드
- `HmrBroadcaster`로 다중 클라이언트 지원 (broadcast channel)
- 파일 변경 → 빌드 완료 → WebSocket으로 "reload" 메시지 전송
- 클라이언트에서 `location.reload()` 호출
- 자동 재연결 (exponential backoff)

**변경된 파일:**
- `Cargo.toml`: tokio-tungstenite, futures-util 추가
- `src/server/hmr.rs`: HmrBroadcaster 구현
- `src/server/mod.rs`: WebSocket 업그레이드 핸들러
- `src/cli/dev.rs`: 빌드 완료 시 reload 신호 전송
- `src/cli/init.rs`: client.tsx에 HMR 클라이언트 코드 추가

**2단계: React Fast Refresh (향후)**
- `react-refresh` 패키지 추가
- Rolldown 플러그인으로 Fast Refresh 변환 주입
- 컴포넌트 상태 유지하면서 교체

---

## 구현 우선순위

| 순서 | 기능 | 상태 | 난이도 |
|------|------|------|--------|
| 1 | Hydration 지원 | ✅ 완료 | 중 |
| 2 | 에셋 해싱 | ✅ 완료 | 하 |
| 3 | 클라이언트 HMR (LiveReload) | ✅ 완료 | 중 |
| 4 | React Fast Refresh | ⏳ 향후 | 상 |

## 기술적 결정사항

### 의존성 경로
- `forte-rs-to-ts`: `env!("CARGO_MANIFEST_DIR")` 기반 상대 경로
- `forte-json`: path 의존성 (개발 환경)
- `RUSTUP_TOOLCHAIN` 환경변수 제거하여 rust-toolchain.toml 사용

### 프론트엔드 번들링
- rolldown 사용
- `globalThis.handler` 패턴으로 전역 핸들러 노출
- `inlineDynamicImports: true`로 단일 번들 생성

### 백엔드 패키지
- 패키지 이름: `backend` (고정)
- WASM 파일: `rs/target/wasm32-wasip2/release/backend.wasm`
