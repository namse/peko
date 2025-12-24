import app from "./index";
import { fire } from "@bytecodealliance/jco-std/wasi/0.2.6/http/adapters/hono/server";

fire(app);

export { incomingHandler } from "@bytecodealliance/jco-std/wasi/0.2.6/http/adapters/hono/server";
