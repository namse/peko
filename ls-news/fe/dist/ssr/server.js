import { renderToString } from "react-dom/server";
const routes = [
  { path: "/", component: () => import("./assets/page-Di5j51Fj.js"), schema: () => import("./assets/.props-D_UHkcGk.js") },
  { path: "/post/:id", component: () => import("./assets/page-D37_hcsP.js"), schema: () => import("./assets/.props-DdBYkuvo.js") }
];
function matchRoute(pathname) {
  for (const route of routes) {
    const routeParts = route.path.split("/");
    const pathParts = pathname.split("/");
    if (routeParts.length !== pathParts.length) continue;
    const params = {};
    let match = true;
    for (let i = 0; i < routeParts.length; i++) {
      if (routeParts[i].startsWith(":")) {
        params[routeParts[i].slice(1)] = pathParts[i];
      } else if (routeParts[i] !== pathParts[i]) {
        match = false;
        break;
      }
    }
    if (match) {
      return { route, params };
    }
  }
  return null;
}
function escapeJsonForScript(json) {
  return json.replace(/</g, "\\u003c").replace(/>/g, "\\u003e");
}
async function render(url, rawProps) {
  const urlObj = new URL(url, "http://localhost");
  const matched = matchRoute(urlObj.pathname);
  if (!matched) {
    return "Not Found";
  }
  const [pageModule, schemaModule] = await Promise.all([
    matched.route.component(),
    matched.route.schema()
  ]);
  const props = schemaModule.PropsSchema.parse(rawProps);
  const allProps = { ...props, params: matched.params };
  const html = renderToString(pageModule.default(allProps));
  const propsJson = escapeJsonForScript(JSON.stringify(allProps));
  const viteScripts = `<script type="module" src="/@vite/client"><\/script>`;
  const clientScript = `/src/client.tsx`;
  return `<!DOCTYPE html>
<html lang="ja">
<head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width" />
    <title>ls-news</title>
    <link rel="stylesheet" href="/src/styles/globals.css" />
    ${viteScripts}
</head>
<body>
    <div id="root">${html}</div>
    <script>window.__FORTE_PROPS__ = ${propsJson};<\/script>
    <script type="module" src="${clientScript}"><\/script>
</body>
</html>`;
}
globalThis.handler = async function handler(request) {
  const rawProps = await request.json();
  const url = new URL(request.url);
  const matched = matchRoute(url.pathname);
  if (matched) {
    const [pageModule, schemaModule] = await Promise.all([
      matched.route.component(),
      matched.route.schema()
    ]);
    const props = schemaModule.PropsSchema.parse(rawProps);
    const allProps = { ...props, params: matched.params };
    const html = renderToString(pageModule.default(allProps));
    const propsJson = escapeJsonForScript(JSON.stringify(allProps));
    const viteScripts = "";
    const clientScript = `/public/client.js`;
    const cssLink = `<link rel="stylesheet" href="/public/globals.css" />`;
    return new Response(
      `<!DOCTYPE html>
<html lang="ja">
<head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width" />
    <title>ls-news</title>
    ${cssLink}
    ${viteScripts}
</head>
<body>
    <div id="root">${html}</div>
    <script>window.__FORTE_PROPS__ = ${propsJson};<\/script>
    <script type="module" src="${clientScript}"><\/script>
</body>
</html>`,
      {
        headers: { "Content-Type": "text/html" }
      }
    );
  }
  return new Response("Not Found", { status: 404 });
};
export {
  render
};
