import { renderToString } from "react-dom/server";
import { routes } from "./routes.generated";

function matchRoute(pathname: string): { route: typeof routes[0]; params: Record<string, string> } | null {
    for (const route of routes) {
        const routeParts = route.path.split("/");
        const pathParts = pathname.split("/");

        if (routeParts.length !== pathParts.length) continue;

        const params: Record<string, string> = {};
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

function escapeJsonForScript(json: string): string {
    return json.replace(/</g, "\\u003c").replace(/>/g, "\\u003e");
}

const isDev = import.meta.env?.DEV ?? false;

export async function render(url: string, props: any): Promise<string> {
    const urlObj = new URL(url, "http://localhost");
    const matched = matchRoute(urlObj.pathname);

    if (!matched) {
        return "Not Found";
    }

    const allProps = { ...props, params: matched.params };
    const pageModule = await matched.route.component();
    const html = renderToString(pageModule.default(allProps));
    const propsJson = escapeJsonForScript(JSON.stringify(allProps));

    const viteScripts = `<script type="module" src="/@vite/client"></script>`;
    const clientScript = `/src/client.tsx`;

    return `<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8" />
    <title>Forte App</title>
    ${viteScripts}
</head>
<body>
    <div id="root">${html}</div>
    <script>window.__FORTE_PROPS__ = ${propsJson};</script>
    <script type="module" src="${clientScript}"></script>
</body>
</html>`;
}

(globalThis as any).handler = async function handler(request: Request): Promise<Response> {
    const props = await request.json();
    const url = new URL(request.url);

    const matched = matchRoute(url.pathname);
    if (matched) {
        const allProps = { ...props, params: matched.params };
        const pageModule = await matched.route.component();
        const html = renderToString(pageModule.default(allProps));
        const propsJson = escapeJsonForScript(JSON.stringify(allProps));

        const viteScripts = isDev
            ? `<script type="module" src="/@vite/client"></script>`
            : "";
        const clientScript = isDev
            ? `/src/client.tsx`
            : `/public/client.js`;

        return new Response(
            `<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8" />
    <title>Forte App</title>
    ${viteScripts}
</head>
<body>
    <div id="root">${html}</div>
    <script>window.__FORTE_PROPS__ = ${propsJson};</script>
    <script type="module" src="${clientScript}"></script>
</body>
</html>`,
            {
                headers: { "Content-Type": "text/html" },
            }
        );
    }

    return new Response("Not Found", { status: 404 });
};
