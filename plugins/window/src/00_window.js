function getActiveWindowTitle() {
    return Deno.core.ops.op2_get_active_window_title();
}

function getInactiveWindowTitles() {
    return Deno.core.ops.op2_get_inactive_window_titles();
}

globalThis.app = globalThis.app || {};
globalThis.app.sapphillon = globalThis.app.sapphillon || {};
globalThis.app.sapphillon.core = globalThis.app.sapphillon.core || {};
globalThis.app.sapphillon.core.window = globalThis.app.sapphillon.core.window || {};

globalThis.app.sapphillon.core.window.getActiveWindowTitle = getActiveWindowTitle;
globalThis.app.sapphillon.core.window.getInactiveWindowTitles = getInactiveWindowTitles;
