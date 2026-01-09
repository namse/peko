import { jsx, jsxs } from "react/jsx-runtime";
import { L as Link, a as Layout } from "./Layout-doJt0_af.js";
import "clsx";
import "tailwind-merge";
function NewsItem({
  item
}) {
  return /* @__PURE__ */ jsx("div", { className: "py-3 border-b", children: /* @__PURE__ */ jsxs("div", { className: "flex items-baseline gap-2 flex-wrap", children: [
    /* @__PURE__ */ jsx(Link, { href: `/post/${item.id}`, size: "base", weight: "medium", children: item.title }),
    /* @__PURE__ */ jsx(Link, { href: item.url, size: "sm", muted: true, popup: true, children: item.domain })
  ] }) });
}
function NewsPagination({
  lastKey,
  hasMore
}) {
  return /* @__PURE__ */ jsx("div", { className: "py-6 border-b", children: hasMore ? /* @__PURE__ */ jsx(Link, { href: `/?after=${lastKey}`, children: "Load more" }) : /* @__PURE__ */ jsx("p", { className: "text-center text-muted-foreground", children: "End of list" }) });
}
function IndexPage(props) {
  if (props.t === "DbErr") {
    return /* @__PURE__ */ jsx(Layout, { children: /* @__PURE__ */ jsxs("div", { className: "text-center text-red-500", children: [
      "Db Error: ",
      props.message
    ] }) });
  }
  const displayedItems = props.rows.map((row) => ({
    id: row.post.id,
    title: row.post.title,
    url: row.post.url,
    domain: getDomain(row.post.url),
    author: row.author,
    deletedAt: row.deletedAt
  }));
  const hasMore = displayedItems.length === 10;
  return /* @__PURE__ */ jsxs(Layout, { children: [
    displayedItems.map((item) => /* @__PURE__ */ jsx(NewsItem, { item }, item.id)),
    displayedItems.length > 0 && /* @__PURE__ */ jsx(
      NewsPagination,
      {
        lastKey: displayedItems[displayedItems.length - 1].id,
        hasMore
      }
    )
  ] });
}
function getDomain(url) {
  try {
    return new URL(url).hostname;
  } catch {
    return url;
  }
}
export {
  IndexPage as default
};
