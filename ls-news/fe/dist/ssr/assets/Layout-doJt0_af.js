import { jsx, jsxs, Fragment } from "react/jsx-runtime";
import { clsx } from "clsx";
import { twMerge } from "tailwind-merge";
function cn(...inputs) {
  return twMerge(clsx(inputs));
}
const fontSizes = {
  sm: "text-sm",
  base: "text-base"
};
const fontWeights = {
  thin: "font-thin",
  extralight: "font-extralight",
  light: "font-light",
  normal: "font-normal",
  medium: "font-medium",
  semibold: "font-semibold",
  bold: "font-bold",
  extrabold: "font-extrabold",
  black: "font-black"
};
function Link({
  href,
  popup,
  size,
  weight,
  muted,
  children,
  ...props
}) {
  const popupProps = popup ? { target: "_blank", rel: "noopener noreferrer" } : {};
  return /* @__PURE__ */ jsx(
    "a",
    {
      href,
      className: cn(
        "hover:underline",
        size ? fontSizes[size] : "",
        weight ? fontWeights[weight] : "",
        muted ? "text-muted-foreground" : ""
      ),
      ...popupProps,
      ...props,
      children
    }
  );
}
function NewsHeader() {
  return /* @__PURE__ */ jsx("header", { className: "border-b bg-background", children: /* @__PURE__ */ jsx("nav", { className: "container mx-auto px-4 py-3", children: /* @__PURE__ */ jsxs("div", { className: "flex items-center justify-between gap-4", children: [
    /* @__PURE__ */ jsxs("div", { className: "flex items-center gap-4 flex-wrap", children: [
      /* @__PURE__ */ jsxs("a", { href: "/", className: "text-lg font-mono font-bold", children: [
        /* @__PURE__ */ jsx("span", { className: "text-emerald-500", children: ">" }),
        /* @__PURE__ */ jsx("span", { className: "text-sky-500", children: " ls" }),
        /* @__PURE__ */ jsx("span", { className: "text-amber-500", children: " news" }),
        /* @__PURE__ */ jsx("span", { className: "text-purple-500", children: "/*" })
      ] }),
      /* @__PURE__ */ jsxs("div", { className: "flex items-center gap-3 text-sm", children: [
        /* @__PURE__ */ jsx(Link, { href: "/best", children: "Best" }),
        /* @__PURE__ */ jsx(Link, { href: "/showls", children: "Show ls" }),
        /* @__PURE__ */ jsx(Link, { href: "/write", children: "New Post" })
      ] })
    ] }),
    /* @__PURE__ */ jsx("div", { className: "flex items-center gap-4", children: /* @__PURE__ */ jsx("span", { className: "text-sm text-muted-foreground", children: "Guest" }) })
  ] }) }) });
}
function Footer() {
  return /* @__PURE__ */ jsx("footer", { className: "border-t bg-background py-6", children: /* @__PURE__ */ jsx("div", { className: "container mx-auto px-4", children: /* @__PURE__ */ jsx("div", { className: "flex flex-col gap-4", children: /* @__PURE__ */ jsx("div", { className: "flex items-center gap-2 text-sm flex-wrap" }) }) }) });
}
function Layout({ children }) {
  return /* @__PURE__ */ jsxs(Fragment, { children: [
    /* @__PURE__ */ jsx(NewsHeader, {}),
    /* @__PURE__ */ jsx("main", { className: "container mx-auto px-4 py-6", children }),
    /* @__PURE__ */ jsx(Footer, {})
  ] });
}
export {
  Link as L,
  Layout as a,
  cn as c
};
