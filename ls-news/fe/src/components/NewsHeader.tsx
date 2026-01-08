import { Link } from "@/components/ui/link";

export function NewsHeader() {
  return (
    <header className="border-b bg-background">
      <nav className="container mx-auto px-4 py-3">
        <div className="flex items-center justify-between gap-4">
          <div className="flex items-center gap-4 flex-wrap">
            <a href="/" className="text-lg font-mono font-bold">
              <span className="text-emerald-500">&gt;</span>
              <span className="text-sky-500"> ls</span>
              <span className="text-amber-500"> news</span>
              <span className="text-purple-500">/*</span>
            </a>
            <div className="flex items-center gap-3 text-sm">
              <Link href="/best">Best</Link>
              <Link href="/showls">Show ls</Link>
              <Link href="/write">New Post</Link>
            </div>
          </div>

          <div className="flex items-center gap-4">
            <span className="text-sm text-muted-foreground">Guest</span>
          </div>
        </div>
      </nav>
    </header>
  );
}
