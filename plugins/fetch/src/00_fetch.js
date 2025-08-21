function fetch_str(url) {
    return Deno.core.ops.op2_fetch(url);
}

globalThis.fetch = fetch;