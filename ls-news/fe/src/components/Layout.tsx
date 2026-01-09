import { NewsHeader } from "./NewsHeader";
import { Footer } from "./Footer";

export function Layout({
  children,
  me,
}: {
  children: React.ReactNode;
  me?: { id: string; username: string; avatarUrl: string };
}) {
  return (
    <>
      <NewsHeader me={me} />
      <main className="container mx-auto px-4 py-6">{children}</main>
      <Footer />
    </>
  );
}
