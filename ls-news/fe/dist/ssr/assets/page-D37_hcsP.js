import { jsx, jsxs, Fragment } from "react/jsx-runtime";
import { c as cn, a as Layout, L as Link } from "./Layout-doJt0_af.js";
import { ThumbsUp, ThumbsDown } from "lucide-react";
import Markdown from "react-markdown";
import "clsx";
import "tailwind-merge";
function IconButton({
  children,
  className,
  ...props
}) {
  return /* @__PURE__ */ jsx(
    "button",
    {
      className: cn("p-2 rounded-full cursor-pointer", className),
      ...props,
      children
    }
  );
}
function PostPage(props) {
  if (props.t === "NotFound") {
    return /* @__PURE__ */ jsx(Layout, { children: /* @__PURE__ */ jsxs("div", { className: "text-center py-20", children: [
      /* @__PURE__ */ jsx("h1", { className: "text-2xl font-bold mb-4", children: "Post not found" }),
      /* @__PURE__ */ jsx(Link, { href: "/", children: "Go back to home" })
    ] }) });
  }
  if (props.t === "DbErr") {
    return /* @__PURE__ */ jsx(Layout, { children: /* @__PURE__ */ jsxs("div", { className: "text-center text-red-500", children: [
      "Db Error: ",
      props.message
    ] }) });
  }
  const { post, comments, users } = props;
  const author = users[post.authorId];
  return /* @__PURE__ */ jsxs(Layout, { children: [
    /* @__PURE__ */ jsx(
      PostHeader,
      {
        post,
        author,
        commentsCount: comments.length
      }
    ),
    /* @__PURE__ */ jsxs("div", { className: "mb-8", children: [
      /* @__PURE__ */ jsx(PostContent, { content: post.content }),
      /* @__PURE__ */ jsx(PostInteractions, { likes: post.likes })
    ] }),
    /* @__PURE__ */ jsx(CommentList, { comments, users })
  ] });
}
function PostHeader({
  post,
  author,
  commentsCount
}) {
  const timeAgo = getRelativeTimeText(post.createdAt);
  return /* @__PURE__ */ jsxs("div", { className: "mb-8", children: [
    /* @__PURE__ */ jsx("div", { className: "mb-2 flex items-start justify-between gap-4", children: /* @__PURE__ */ jsxs("div", { className: "flex-1", children: [
      /* @__PURE__ */ jsx("h1", { className: "text-3xl font-bold", children: post.title }),
      /* @__PURE__ */ jsx(Link, { href: post.url, size: "sm", muted: true, popup: true, children: post.url })
    ] }) }),
    /* @__PURE__ */ jsxs("div", { className: "flex items-center gap-2 text-sm text-muted-foreground", children: [
      author && /* @__PURE__ */ jsxs(Fragment, { children: [
        /* @__PURE__ */ jsx(
          "img",
          {
            src: author.avatarUrl,
            alt: author.username,
            className: "w-6 h-6 rounded-full"
          }
        ),
        /* @__PURE__ */ jsx(Link, { href: `https://github.com/${author.username}`, popup: true, children: author.username }),
        " | "
      ] }),
      /* @__PURE__ */ jsx("span", { children: timeAgo }),
      " | ",
      /* @__PURE__ */ jsxs("span", { children: [
        commentsCount,
        " comments"
      ] })
    ] })
  ] });
}
function PostContent({ content }) {
  return /* @__PURE__ */ jsx("div", { className: "prose prose-lg prose-slate max-w-none mb-4 p-6 bg-muted/30 rounded-lg leading-relaxed", children: /* @__PURE__ */ jsx(Markdown, { children: content }) });
}
function PostInteractions({ likes }) {
  return /* @__PURE__ */ jsxs("div", { className: "flex items-center gap-4 px-4", children: [
    /* @__PURE__ */ jsxs(IconButton, { "aria-label": "Like", className: "flex items-center gap-2 px-2", children: [
      /* @__PURE__ */ jsx(ThumbsUp, { size: 20 }),
      /* @__PURE__ */ jsx("span", { className: "text-sm font-medium", children: likes })
    ] }),
    /* @__PURE__ */ jsx(IconButton, { "aria-label": "Dislike", children: /* @__PURE__ */ jsx(ThumbsDown, { size: 20 }) })
  ] });
}
function CommentList({
  comments,
  users
}) {
  const commentsMap = /* @__PURE__ */ new Map();
  comments.forEach((comment) => {
    commentsMap.set(comment.id, comment);
  });
  function calculateIndent(comment) {
    let indent = 0;
    let currentComment = comment;
    while (currentComment?.parentCommentId) {
      currentComment = commentsMap.get(currentComment.parentCommentId);
      indent++;
    }
    return indent;
  }
  return /* @__PURE__ */ jsx("div", { children: comments.map((comment) => /* @__PURE__ */ jsx(
    CommentItem,
    {
      comment,
      author: users[comment.authorId],
      indent: calculateIndent(comment)
    },
    comment.id
  )) });
}
function CommentItem({
  comment,
  author,
  indent
}) {
  const timeAgo = getRelativeTimeText(comment.createdAt);
  return /* @__PURE__ */ jsxs(
    "div",
    {
      id: `comment-${comment.id}`,
      className: "border-l-2 border-muted pl-4 py-4 mb-2",
      style: { marginLeft: `${indent * 24}px` },
      children: [
        /* @__PURE__ */ jsxs("div", { className: "flex items-center gap-2 text-sm text-muted-foreground mb-2", children: [
          author && /* @__PURE__ */ jsxs(Fragment, { children: [
            /* @__PURE__ */ jsx(
              "img",
              {
                src: author.avatarUrl,
                alt: author.username,
                className: "w-6 h-6 rounded-full"
              }
            ),
            /* @__PURE__ */ jsx(Link, { href: `https://github.com/${author.username}`, popup: true, children: author.username })
          ] }),
          /* @__PURE__ */ jsx("span", { children: timeAgo })
        ] }),
        /* @__PURE__ */ jsx("div", { className: "prose prose-base max-w-none leading-relaxed mb-3", children: /* @__PURE__ */ jsx(Markdown, { children: comment.content }) }),
        /* @__PURE__ */ jsxs("div", { className: "flex items-center gap-4", children: [
          /* @__PURE__ */ jsxs(IconButton, { "aria-label": "Like", className: "flex items-center gap-2 px-2", children: [
            /* @__PURE__ */ jsx(ThumbsUp, { size: 16 }),
            /* @__PURE__ */ jsx("span", { className: "text-sm", children: comment.likes })
          ] }),
          /* @__PURE__ */ jsx(IconButton, { "aria-label": "Dislike", children: /* @__PURE__ */ jsx(ThumbsDown, { size: 16 }) })
        ] })
      ]
    }
  );
}
function getRelativeTimeText(date) {
  const formatter = new Intl.RelativeTimeFormat("en");
  const ranges = {
    years: 3600 * 24 * 365,
    months: 3600 * 24 * 30,
    weeks: 3600 * 24 * 7,
    days: 3600 * 24,
    hours: 3600,
    minutes: 60,
    seconds: 1
  };
  const secondsElapsed = (date.getTime() - Date.now()) / 1e3;
  for (const [key, value] of Object.entries(ranges)) {
    if (value < Math.abs(secondsElapsed)) {
      const delta = secondsElapsed / value;
      return formatter.format(
        Math.round(delta),
        key
      );
    }
  }
  return "just now";
}
export {
  PostPage as default
};
