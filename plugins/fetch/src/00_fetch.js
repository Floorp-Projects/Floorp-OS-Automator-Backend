function fetch(url) {
    return Deno.core.ops.op2_fetch(url);
}

function post(url, body) {
    return Deno.core.ops.op2_post(url, body);
}

globalThis.fetch = fetch;
globalThis.post = post;