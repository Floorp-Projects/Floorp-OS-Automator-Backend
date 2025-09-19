function readFile(path) {
    return Deno.core.ops.op2_read_file(path);
}

globalThis.readFile = readFile;
