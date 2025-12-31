function exec(command) {
    return Deno.core.ops.op2_exec(command);
}

globalThis.app = globalThis.app || {};
globalThis.app.sapphillon = globalThis.app.sapphillon || {};
globalThis.app.sapphillon.core = globalThis.app.sapphillon.core || {};
globalThis.app.sapphillon.core.exec = globalThis.app.sapphillon.core.exec || {};

globalThis.app.sapphillon.core.exec.exec = exec;
