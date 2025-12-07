function searchFile(root_path, query) {
    return Deno.core.ops.op2_search_file(root_path, query);
}

globalThis.search = globalThis.search || {};
globalThis.search.file = searchFile;
