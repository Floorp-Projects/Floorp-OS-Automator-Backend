function searchFile(root_path, query) {
    return Deno.core.ops.op2_search_file(root_path, query);
}

globalThis.app = globalThis.app || {};
globalThis.app.sapphillon = globalThis.app.sapphillon || {};
globalThis.app.sapphillon.core = globalThis.app.sapphillon.core || {};
globalThis.app.sapphillon.core.search = globalThis.app.sapphillon.core.search || {};

globalThis.app.sapphillon.core.search.file = searchFile;
