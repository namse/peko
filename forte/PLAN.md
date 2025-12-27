# Co-Router Master Plan v1.0 (Final)

---

## 1. 아키텍처 개요

### 1.1 철학

**"물리적 분리, 논리적 통합"**

- 코드는 백엔드(Rust)와 프론트엔드(React)로 나뉘어 있지만, 라우팅과 데이터 타입은 CLI에 의해 강력하게 결합됩니다.
- 개발자는 "설정(Config)"을 건드리지 않습니다. 오직 "규칙(Convention)"에 맞춰 파일만 만듭니다.
- 프레임워크는 최소한의 기능만 제공하고, 확장은 사용자의 몫입니다.

### 1.2 Core Stack

| 레이어        | 기술                   | 역할                                           |
| ------------- | ---------------------- | ---------------------------------------------- |
| CLI           | Rust                   | 파일 감지, 코드 파싱, 코드 생성, 프로세스 관리 |
| Backend       | Rust + Axum            | API 서버 (사용자에게 Axum은 숨겨짐)            |
| Frontend      | React + Vite + Node.js | SSR 렌더링 서버                                |
| Communication | Internal HTTP          | Node → Rust 통신                               |

### 1.3 요청 흐름

```
Browser ──GET /product/123──▶ Node.js (3000)
                                  │
                                  ▼
                         Internal Fetch
                                  │
                                  ▼
                            Rust (8080)
                                  │
                                  ▼
                         PageProps (JSON)
                                  │
                                  ▼
                         Node.js SSR
                                  │
                                  ▼
Browser ◀──────────────── HTML + Hydration Script
```

---

## 2. 디렉토리 구조

### 2.1 전체 구조

```
my-project/
├── backend/
│   ├── Cargo.toml
│   └── src/
│       └── routes/
│           ├── mod.rs              [Generated] CLI가 자동 관리
│           ├── index/
│           │   └── props.rs        [User Code]
│           └── product/
│               └── _id_/           ← Rust는 _id_ 형식
│                   └── props.rs    [User Code]
│
├── frontend/
│   ├── package.json
│   └── src/
│       └── app/
│           ├── layout.tsx          [User Code] 루트 레이아웃
│           ├── index/
│           │   └── page.tsx        [User Code]
│           └── product/
│               └── [id]/           ← Frontend는 [id] 형식
│                   ├── page.tsx    [User Code]
│                   └── props.gen.ts [Generated]
│
├── .generated/                     [Hidden] CLI 생성 코드
│   ├── backend/
│   │   ├── main.rs
│   │   ├── router.rs
│   │   └── env.rs
│   └── frontend/
│       ├── server.js
│       ├── client.tsx
│       └── routes.ts
│
├── .env
├── .env.development
├── .env.production
├── Co-Router.toml
└── README.md
```

### 2.2 경로 매핑 규칙

| Backend (Rust)                        | Frontend (React)                   | URL                          |
| ------------------------------------- | ---------------------------------- | ---------------------------- |
| `routes/index/`                       | `app/index/`                       | `/`                          |
| `routes/about/`                       | `app/about/`                       | `/about`                     |
| `routes/product/_id_/`                | `app/product/[id]/`                | `/product/:id`               |
| `routes/user/_userId_/post/_postId_/` | `app/user/[userId]/post/[postId]/` | `/user/:userId/post/:postId` |

### 2.3 라우트 그룹 (URL 미반영)

```
app/
└── (marketing)/        ← 괄호로 감싸면 URL에 미포함
    ├── about/page.tsx      → /about
    └── contact/page.tsx    → /contact
```

---

## 3. CLI 엔진 상세

### 3.1 명령어 체계

```bash
co-router init <project-name>   # 프로젝트 생성
co-router dev                   # 개발 서버 실행
co-router build                 # 프로덕션 빌드
co-router test                  # 테스트 실행
```

### 3.2 Watcher (감시자)

- **라이브러리:** `notify`
- **감시 대상:** `backend/src/routes/**/props.rs`
- **Debounce:** 300ms (연속 저장 시 불필요한 트리거 방지)
- **동작:** 파일 변경 감지 시 Parser → Generator 파이프라인 실행

### 3.3 Parser (분석기)

- **라이브러리:** `syn`
- **추출 정보:**

| 대상                       | 추출 내용                |
| -------------------------- | ------------------------ |
| `*Path` 구조체             | URL Path Parameter 필드  |
| `*Query` 구조체            | Query String 필드        |
| `*Header` 구조체           | HTTP Header 필드         |
| `PageProps` 구조체         | 응답 데이터 필드 및 타입 |
| `ActionInput` 구조체       | POST 요청 바디 필드      |
| `get_props` 함수           | 인자 목록 및 반환 타입   |
| `post_action` 함수         | 인자 목록 및 반환 타입   |
| `#[serde(...)]` 어트리뷰트 | rename, skip 처리        |

### 3.4 Generator (생성기)

**TypeScript 생성 (`props.gen.ts`):**

```typescript
// [Generated] Do not edit manually
export interface PageProps {
  id: number;
  name: string;
  description: string | null;
  tags: string[];
}

export interface ActionInput {
  title: string;
  content: string;
}
```

**Rust Router 생성 (`router.rs`):**

```rust
// [Generated] Do not edit manually
use axum::{Router, routing::get};

pub fn create_router() -> Router {
    Router::new()
        .route("/", get(wrapper_index))
        .route("/product/:id", get(wrapper_product_id).post(wrapper_product_id_action))
}
```

---

## 4. 타입 시스템

### 4.1 기본 타입 매핑

| Rust                      | TypeScript          |
| ------------------------- | ------------------- |
| `String`, `&str`          | `string`            |
| `i8`, `i16`, `i32`, `i64` | `number`            |
| `u8`, `u16`, `u32`, `u64` | `number`            |
| `f32`, `f64`              | `number`            |
| `bool`                    | `boolean`           |
| `Option<T>`               | `T \| null`         |
| `Vec<T>`                  | `T[]`               |
| `HashMap<String, T>`      | `Record<string, T>` |

### 4.2 외부 크레이트 타입 매핑

`Co-Router.toml`에서 설정:

```toml
[type_mappings]
"chrono::DateTime<Utc>" = "string"
"chrono::NaiveDate" = "string"
"uuid::Uuid" = "string"
"rust_decimal::Decimal" = "string"
```

### 4.3 Enum 변환

```rust
// Rust
#[derive(Serialize)]
#[serde(tag = "type")]
pub enum Status {
    Pending,
    Active { since: String },
    Inactive,
}
```

```typescript
// Generated TypeScript
export type Status =
  | { type: "Pending" }
  | { type: "Active"; since: string }
  | { type: "Inactive" };
```

### 4.4 중첩 구조체

같은 파일 내 정의된 구조체는 자동으로 함께 변환:

```rust
pub struct Author {
    pub name: String,
}

pub struct PageProps {
    pub title: String,
    pub author: Author,
}
```

```typescript
export interface Author {
  name: string;
}

export interface PageProps {
  title: string;
  author: Author;
}
```

### 4.5 Serde 어트리뷰트 지원

```rust
pub struct PageProps {
    #[serde(rename = "userName")]
    pub user_name: String,

    #[serde(skip)]
    pub internal_id: i32,  // TypeScript에 포함되지 않음
}
```

---

## 5. 에러 핸들링

### 5.1 AppError 타입

프레임워크가 제공하는 에러 enum:

```rust
// co_router::error (프레임워크 제공)
pub enum AppError {
    NotFound(String),
    BadRequest(String),
    Unauthorized(String),
    Forbidden(String),
    Internal(String),
}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
```

### 5.2 사용자 코드에서의 사용

```rust
use co_router::error::AppError;

pub async fn get_props(path: ProductPath) -> Result<PageProps, AppError> {
    let product = db::find_product(path.id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Product {} not found", path.id)))?;

    Ok(PageProps {
        id: product.id,
        name: product.name,
    })
}
```

### 5.3 에러 UI 컴포넌트

```
frontend/src/app/
├── error.tsx           ← 전역 에러 페이지
└── product/
    └── [id]/
        ├── page.tsx
        └── error.tsx   ← 라우트별 에러 페이지 (선택적)
```

```tsx
// error.tsx
interface ErrorProps {
  status: number;
  message: string;
}

export default function Error({ status, message }: ErrorProps) {
  return (
    <div>
      <h1>{status}</h1>
      <p>{message}</p>
    </div>
  );
}
```

에러 발생 시 탐색 순서: 현재 경로 → 상위 경로 → 루트 `error.tsx`

### 5.4 개발 모드 에러 오버레이

- Rust 컴파일 에러 시 브라우저에 오버레이 표시
- 스택트레이스 및 소스 위치 표시
- Node → Rust 통신 실패 시 재시도 안내

---

## 6. 레이아웃 시스템

### 6.1 기본 구조

```
frontend/src/app/
├── layout.tsx              ← 모든 페이지에 적용
├── page.tsx
└── dashboard/
    ├── layout.tsx          ← /dashboard/* 에 적용
    └── settings/
        └── page.tsx        ← 두 레이아웃 모두 적용
```

### 6.2 레이아웃 컴포넌트

```tsx
// layout.tsx
interface LayoutProps {
  children: React.ReactNode;
}

export default function Layout({ children }: LayoutProps) {
  return (
    <html>
      <body>
        <nav>...</nav>
        <main>{children}</main>
      </body>
    </html>
  );
}
```

### 6.3 특징

- 레이아웃은 `props`를 받지 않음 (서버 데이터 페칭 없음)
- 레이아웃에서 데이터가 필요하면 클라이언트 컴포넌트로 자체 fetch
- 중첩 시 외부 → 내부 순서로 래핑

---

## 7. Post Action

### 7.1 백엔드 정의

```rust
// backend/src/routes/login/props.rs

pub struct PageProps {
    pub error: Option<String>,
}

pub struct ActionInput {
    pub email: String,
    pub password: String,
}

pub enum ActionResult {
    Redirect(String),
    Render(PageProps),
}

pub async fn get_props() -> PageProps {
    PageProps { error: None }
}

pub async fn post_action(input: ActionInput) -> ActionResult {
    match authenticate(&input.email, &input.password).await {
        Ok(user) => {
            // 세션/쿠키 설정 로직
            ActionResult::Redirect("/dashboard".into())
        }
        Err(_) => {
            ActionResult::Render(PageProps {
                error: Some("Invalid credentials".into()),
            })
        }
    }
}
```

### 7.2 프론트엔드 사용

```tsx
// frontend/src/app/login/page.tsx
import type { PageProps, ActionInput } from "./props.gen";

export default function LoginPage({ error }: PageProps) {
  return (
    <form method="POST">
      <input name="email" type="email" required />
      <input name="password" type="password" required />
      {error && <p className="error">{error}</p>}
      <button type="submit">Login</button>
    </form>
  );
}
```

### 7.3 Validation

`validator` 크레이트 derive 매크로 지원:

```rust
use validator::Validate;

#[derive(Validate)]
pub struct ActionInput {
    #[validate(email)]
    pub email: String,

    #[validate(length(min = 8))]
    pub password: String,
}
```

Validation 실패 시 자동으로 400 응답과 에러 메시지 반환.

---

## 8. 클라이언트 내비게이션

### 8.1 기본 동작

기본은 MPA (풀 페이지 리로드). 단순하고 예측 가능.

### 8.2 옵트인 SPA 내비게이션

```tsx
import { Link } from "co-router/client";

export default function Navigation() {
  return (
    <nav>
      <Link href="/">Home</Link>
      <Link href="/product/1" prefetch>
        Product
      </Link>
    </nav>
  );
}
```

### 8.3 Link 컴포넌트 동작

1. 클릭 시 `e.preventDefault()`
2. `/api/product/1` (Rust 서버)로 직접 fetch
3. 해당 `page.tsx`를 동적 import
4. 클라이언트에서 렌더링
5. `history.pushState`로 URL 업데이트

### 8.4 Prefetch

`prefetch` 속성이 있으면 viewport에 들어올 때 미리 데이터와 JS 청크를 로드.

---

## 9. 환경 설정

### 9.1 파일 구조

```
my-project/
├── .env                    ← 공통 (gitignore 권장)
├── .env.development        ← 개발 환경
├── .env.production         ← 프로덕션 환경
└── .env.example            ← 템플릿 (git 포함)
```

### 9.2 Co-Router.toml 스키마

```toml
[env]
required = ["DATABASE_URL", "JWT_SECRET"]
optional = ["REDIS_URL", "LOG_LEVEL", "SENTRY_DSN"]

[env.defaults]
LOG_LEVEL = "info"
PORT = "3000"
RUST_PORT = "8080"

[type_mappings]
"chrono::DateTime<Utc>" = "string"
"uuid::Uuid" = "string"

[proxy]
forward_headers = ["Cookie", "Authorization", "Accept-Language"]
timeout_ms = 5000

[build]
output_dir = "dist"
```

### 9.3 타입 안전한 환경 변수 접근

CLI가 생성하는 코드:

```rust
// .generated/backend/env.rs
pub struct Env {
    pub database_url: String,
    pub jwt_secret: String,
    pub redis_url: Option<String>,
    pub log_level: String,
}

lazy_static! {
    pub static ref ENV: Env = Env::load();
}
```

사용:

```rust
use crate::generated::env::ENV;

pub async fn get_props() -> PageProps {
    let conn = db::connect(&ENV.database_url).await;
    // ...
}
```

### 9.4 환경 검증

`co-router dev` 또는 `co-router build` 실행 시 required 변수 체크:

```
Error: Missing required environment variables:
  - DATABASE_URL
  - JWT_SECRET

Please set them in .env or .env.development
```

---

## 10. 프로덕션 빌드

### 10.1 빌드 명령어

```bash
co-router build
```

### 10.2 빌드 프로세스

1. 환경 변수 검증 (`.env.production`)
2. `cargo build --release` 실행
3. `vite build` (클라이언트 에셋)
4. `vite build --ssr` (SSR 번들)
5. 결과물 조립

### 10.3 빌드 결과물

```
dist/
├── server                  ← Rust 바이너리 (메인 실행 파일)
├── node/
│   └── server.js           ← SSR 번들
├── client/
│   ├── assets/
│   │   ├── index-[hash].js
│   │   └── index-[hash].css
│   └── .vite/
│       └── manifest.json
└── static/                 ← public 폴더 복사본
```

### 10.4 실행

```bash
# 프로덕션 실행
./dist/server

# 내부적으로:
# 1. Rust 서버가 8080 포트에서 API 서빙
# 2. Node 프로세스가 자식으로 실행되어 SSR 담당
# 3. Rust가 정적 파일도 서빙 (/client/*)
```

### 10.5 Docker 지원

```dockerfile
# Dockerfile (자동 생성 템플릿)
FROM node:20-slim AS node-base
FROM rust:1.75-slim AS builder

# 빌드 단계...

FROM debian:bookworm-slim
COPY --from=builder /app/dist /app
COPY --from=node-base /usr/local/bin/node /usr/local/bin/

WORKDIR /app
CMD ["./server"]
```

---

## 11. 개발 경험 최적화

### 11.1 Hot Reload 전략

**Rust (백엔드):**

- 파일 변경 감지 시 증분 컴파일
- `.generated/backend`를 별도 크레이트로 분리하여 컴파일 범위 최소화
- 컴파일 중 상태를 프론트엔드에 전달

**React (프론트엔드):**

- Vite HMR 활용
- `props.gen.ts` 변경 시 자동 리로드

### 11.2 동기화 메커니즘

```
CLI 내부 상태:
- backend_compiling: bool
- backend_ready: bool
- last_error: Option<String>
```

백엔드 컴파일 중 Node 서버 동작:

1. 새 요청 수신
2. `backend_compiling == true` 확인
3. 503 응답 + "Compiling..." 메시지
4. 컴파일 완료 시 WebSocket으로 클라이언트에 새로고침 신호

### 11.3 에러 표시

컴파일 에러 발생 시 브라우저 오버레이:

```
┌─────────────────────────────────────┐
│  ❌ Rust Compilation Error          │
│                                     │
│  backend/src/routes/product/        │
│  _id_/props.rs:15:9                 │
│                                     │
│  error[E0308]: mismatched types     │
│    expected `String`                │
│    found `i32`                      │
│                                     │
│  [Waiting for file changes...]      │
└─────────────────────────────────────┘
```

---

## 12. 테스트 전략

### 12.1 CLI 테스트

```rust
// CLI 내부 테스트
#[test]
fn test_rust_to_ts_conversion() {
    let rust = r#"pub struct PageProps { pub name: String }"#;
    let ts = parser::convert_to_typescript(rust);
    assert_eq!(ts, "export interface PageProps {\n  name: string;\n}");
}

#[test]
fn test_route_path_parsing() {
    let path = "routes/user/_userId_/post/_postId_";
    let url = router::path_to_url(path);
    assert_eq!(url, "/user/:userId/post/:postId");
}
```

### 12.2 사용자 프로젝트 테스트

```bash
co-router test
# 내부적으로 실행:
# 1. cargo test (backend)
# 2. vitest run (frontend)
```

**백엔드 테스트:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_props() {
        let path = ProductPath { id: 1 };
        let result = get_props(path).await;
        assert!(result.is_ok());
    }
}
```

**프론트엔드 테스트:**

```tsx
import { render, screen } from "@testing-library/react";
import Page from "./page";

test("renders product name", () => {
  render(<Page id={1} name="Test Product" />);
  expect(screen.getByText("Test Product")).toBeInTheDocument();
});
```

---

## 13. 개발 로드맵

### Phase 0: 프로젝트 스캐폴딩 (1주)

- [ ] `co-router init` 명령어 구현
- [ ] 템플릿 파일 임베딩 (`include_str!`)
- [ ] 기본 디렉토리 구조 생성
- [ ] `Co-Router.toml` 기본 설정

**완료 기준:** `co-router init my-app`으로 빈 프로젝트 생성 가능

### Phase 1: 타입 생성기 (2주)

- [ ] `syn`으로 Rust 구조체 파싱
- [ ] TypeScript 인터페이스 변환
- [ ] `notify`로 파일 감시
- [ ] Snake_case → camelCase 변환
- [ ] `Option`, `Vec` 타입 처리
- [ ] `#[serde(rename)]`, `#[serde(skip)]` 지원

**완료 기준:** `props.rs` 저장 시 `props.gen.ts` 자동 생성

### Phase 2: 백엔드 서버 (2주)

- [ ] 라우트 스캔 (`_id_` → `:id` 변환)
- [ ] Wrapper 핸들러 코드 생성
- [ ] `router.rs`, `main.rs` 생성
- [ ] 자식 프로세스로 Rust 서버 실행
- [ ] `AppError` 타입 및 에러 처리
- [ ] 환경 변수 로딩 및 검증

**완료 기준:** `co-router dev`로 API 서버 실행, JSON 응답 확인

### Phase 3: 프론트엔드 SSR (3주)

- [ ] Vite 설정 템플릿
- [ ] Node.js 렌더링 서버
- [ ] Internal Fetch (Node → Rust)
- [ ] Header 전달 로직
- [ ] `renderToString` 연동
- [ ] Hydration 스크립트
- [ ] `layout.tsx` 지원
- [ ] `error.tsx` 지원

**완료 기준:** 브라우저에서 SSR 페이지 렌더링 및 하이드레이션

### Phase 4: 인터랙션 (2주)

- [ ] Post Action 지원 (`post_action` 함수)
- [ ] `ActionInput` 타입 생성
- [ ] `ActionResult` (Redirect/Render) 처리
- [ ] Validation 연동
- [ ] `<Link>` 컴포넌트 (클라이언트 내비게이션)
- [ ] Prefetch 기능

**완료 기준:** 폼 제출 및 클라이언트 내비게이션 동작

### Phase 5: 프로덕션 (2주)

- [ ] `co-router build` 명령어
- [ ] Release 빌드 자동화
- [ ] Vite 빌드 통합
- [ ] Manifest 기반 에셋 주입
- [ ] 정적 파일 서빙
- [ ] Dockerfile 템플릿
- [ ] `co-router test` 명령어

**완료 기준:** 프로덕션 배포 가능한 바이너리 생성

---

## 14. 사용자 워크플로우 예시

### 14.1 프로젝트 시작

```bash
# 1. 프로젝트 생성
co-router init my-blog
cd my-blog

# 2. 개발 서버 실행
co-router dev
```

### 14.2 새 페이지 추가

```bash
# 3. 백엔드 로직 작성
mkdir -p backend/src/routes/post/_id_
```

```rust
// backend/src/routes/post/_id_/props.rs
use co_router::error::AppError;

pub struct PostPath {
    pub id: i32,
}

pub struct PageProps {
    pub id: i32,
    pub title: String,
    pub content: String,
    pub author: String,
}

pub async fn get_props(path: PostPath) -> Result<PageProps, AppError> {
    // 실제로는 DB 조회
    Ok(PageProps {
        id: path.id,
        title: "Hello World".into(),
        content: "This is my first post.".into(),
        author: "Alice".into(),
    })
}
```

```bash
# 4. 저장하면 자동 생성됨
# frontend/src/app/post/[id]/props.gen.ts
```

```tsx
// 5. UI 작성
// frontend/src/app/post/[id]/page.tsx
import type { PageProps } from "./props.gen";

export default function PostPage({ title, content, author }: PageProps) {
  return (
    <article>
      <h1>{title}</h1>
      <p>By {author}</p>
      <div>{content}</div>
    </article>
  );
}
```

```
# 6. 브라우저에서 확인
http://localhost:3000/post/1
```

### 14.3 폼 추가

```rust
// backend/src/routes/post/new/props.rs
pub struct PageProps {
    pub error: Option<String>,
}

pub struct ActionInput {
    pub title: String,
    pub content: String,
}

pub enum ActionResult {
    Redirect(String),
    Render(PageProps),
}

pub async fn get_props() -> PageProps {
    PageProps { error: None }
}

pub async fn post_action(input: ActionInput) -> ActionResult {
    match create_post(&input.title, &input.content).await {
        Ok(post) => ActionResult::Redirect(format!("/post/{}", post.id)),
        Err(e) => ActionResult::Render(PageProps {
            error: Some(e.to_string()),
        }),
    }
}
```

```tsx
// frontend/src/app/post/new/page.tsx
import type { PageProps } from "./props.gen";

export default function NewPostPage({ error }: PageProps) {
  return (
    <form method="POST">
      <input name="title" placeholder="Title" required />
      <textarea name="content" placeholder="Content" required />
      {error && <p className="error">{error}</p>}
      <button type="submit">Create Post</button>
    </form>
  );
}
```

### 14.4 프로덕션 배포

```bash
# 빌드
co-router build

# 실행
./dist/server

# 또는 Docker
docker build -t my-blog .
docker run -p 3000:3000 my-blog
```

---

## 15. 제약 사항 및 명시적 비지원

다음 기능은 의도적으로 지원하지 않습니다:

| 기능                           | 이유                                    |
| ------------------------------ | --------------------------------------- |
| Catch-all 라우팅 (`[...slug]`) | 복잡도 증가, 대부분의 케이스에서 불필요 |
| 내장 미들웨어 시스템           | 사용자가 Rust 레벨에서 직접 구현        |
| GraphQL                        | REST JSON API에 집중                    |
| 다국어(i18n) 내장              | 사용자 구현 영역                        |
| 내장 ORM                       | 사용자가 선호하는 라이브러리 사용       |

---

## 16. 기술 스택 요약

| 구성 요소          | 기술      | 버전  |
| ------------------ | --------- | ----- |
| CLI                | Rust      | 1.75+ |
| Parser             | syn       | 2.x   |
| File Watcher       | notify    | 6.x   |
| Backend Framework  | Axum      | 0.7+  |
| Frontend Framework | React     | 18+   |
| Bundler            | Vite      | 5.x   |
| SSR Runtime        | Node.js   | 20+   |
| Validation         | validator | 0.16+ |

---
