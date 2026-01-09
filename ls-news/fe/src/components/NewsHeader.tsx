import { Link } from "@/components/ui/link";

export function NewsHeader({
  me,
}: {
  me?: { id: string; username: string; avatarUrl: string };
}) {
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
              <Link href="/write">新規投稿</Link>
            </div>
          </div>

          <div className="flex items-center gap-4">
            {me ? (
              <>
                <div className="flex items-center gap-2">
                  {me.avatarUrl && (
                    <img
                      src={me.avatarUrl}
                      alt={me.username}
                      className="w-6 h-6 rounded-full"
                    />
                  )}
                  <span className="text-sm">{me.username}</span>
                </div>
                <button
                  type="button"
                  className="text-sm hover:underline cursor-pointer"
                  onClick={async () => {
                    await fetch("/api/auth/signout", { method: "POST" });
                    window.location.reload();
                  }}
                >
                  ログアウト
                </button>
              </>
            ) : (
              <Link href="/api/auth/login">GitHubでログイン</Link>
            )}
          </div>
        </div>
      </nav>
    </header>
  );
}
