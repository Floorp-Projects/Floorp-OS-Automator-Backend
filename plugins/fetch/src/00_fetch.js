function fetch(url) {
    return Deno.core.ops.op2_fetch(url);
}

globalThis.fetch = fetch;