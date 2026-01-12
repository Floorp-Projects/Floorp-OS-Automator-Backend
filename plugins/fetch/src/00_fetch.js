function fetch(url) {
    return Deno.core.ops.op2_fetch(url);
}

function post(url, body) {
    return Deno.core.ops.op2_post(url, body);
}

globalThis.app = globalThis.app || {};
globalThis.app.sapphillon = globalThis.app.sapphillon || {};
globalThis.app.sapphillon.core = globalThis.app.sapphillon.core || {};
globalThis.app.sapphillon.core.fetch = globalThis.app.sapphillon.core.fetch || {};

globalThis.app.sapphillon.core.fetch.fetch = fetch;
globalThis.app.sapphillon.core.fetch.post = post;