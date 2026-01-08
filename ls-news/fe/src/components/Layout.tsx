import { NewsHeader } from "./NewsHeader";
import { Footer } from "./Footer";

type LayoutProps = {
  children: React.ReactNode;
};

export function Layout({ children }: LayoutProps) {
  return (
    <>
      <NewsHeader />
      <main className="container mx-auto px-4 py-6">{children}</main>
      <Footer />
    </>
  );
}
