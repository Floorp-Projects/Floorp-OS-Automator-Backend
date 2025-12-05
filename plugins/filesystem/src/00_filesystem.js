function readFile(path) {
    return Deno.core.ops.op2_filesystem_read(path);
}

function writeFile(path, content) {
    return Deno.core.ops.op2_filesystem_write(path, content);
}

function listFiles(path) {
    return Deno.core.ops.op2_filesystem_list_files(path);
}

globalThis.fs = globalThis.fs || {};
globalThis.fs.read = readFile;
globalThis.fs.write = writeFile;
globalThis.fs.list = listFiles;
