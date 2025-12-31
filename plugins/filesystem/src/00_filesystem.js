function readFile(path) {
    return Deno.core.ops.op2_filesystem_read(path);
}

function writeFile(path, content) {
    return Deno.core.ops.op2_filesystem_write(path, content);
}

function listFiles(path) {
    return Deno.core.ops.op2_filesystem_list_files(path);
}

globalThis.app = globalThis.app || {};
globalThis.app.sapphillon = globalThis.app.sapphillon || {};
globalThis.app.sapphillon.core = globalThis.app.sapphillon.core || {};
globalThis.app.sapphillon.core.filesystem = globalThis.app.sapphillon.core.filesystem || {};

globalThis.app.sapphillon.core.filesystem.read = readFile;
globalThis.app.sapphillon.core.filesystem.write = writeFile;
globalThis.app.sapphillon.core.filesystem.listFiles = listFiles;
