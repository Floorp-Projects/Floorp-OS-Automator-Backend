function exec(command) {
    return Deno.core.ops.op2_exec(command);
}

globalThis.exec = exec;
