import { createHonoApp } from "./server/hono-app";
import { App } from "astro/app";
import { fire } from "@bytecodealliance/jco-std/wasi/0.2.6/http/adapters/hono/server";
import { incomingHandler } from "@bytecodealliance/jco-std/wasi/0.2.6/http/adapters/hono/server";

export function createExports(manifest: any) {
  const app = createHonoApp(new App(manifest));
  fire(app);

  return { incomingHandler };
}
