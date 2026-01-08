# Claude Code Guidelines for ls-news

## Frontend (React/TypeScript)

### Component Props

- **Page components**: Props는 `.props.ts` 파일에서 import하여 사용
  ```tsx
  import type { Props } from "./.props";
  export default function IndexPage(props: Props) { ... }
  ```

- **일반 컴포넌트**: Props는 항상 inline으로 정의
  ```tsx
  // Good
  export function NewsItem({
    item,
  }: {
    item: { id: string; title: string; };
  }) { ... }

  // Bad - 별도 type 정의 금지
  type NewsItemProps = { item: { id: string; title: string; } };
  export function NewsItem({ item }: NewsItemProps) { ... }
  ```

## Environment Variables

- `TURSO_URL`: libsql 데이터베이스 URL (예: `http://localhost:8080`)
- `TURSO_AUTH_TOKEN`: 인증 토큰 (로컬 개발시 빈 문자열)
