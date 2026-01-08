import { Link } from "@/components/ui/link";

type NewsPaginationProps = {
  lastKey: string;
  hasMore: boolean;
};

export function NewsPagination({ lastKey, hasMore }: NewsPaginationProps) {
  return (
    <div className="py-6 border-b">
      {hasMore ? (
        <Link href={`/?after=${lastKey}`}>Load more</Link>
      ) : (
        <p className="text-center text-muted-foreground">End of list</p>
      )}
    </div>
  );
}
