import { core, internals } from "ext:core/mod.js";

globalThis.console = new internals.Console(core.ops.op_custom_print);
