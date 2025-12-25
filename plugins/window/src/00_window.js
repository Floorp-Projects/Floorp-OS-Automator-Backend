// Window Plugin JavaScript Bindings
console.log("Window Plugin script loading...");

globalThis.window = globalThis.window || {};

globalThis.window.get_active_title = () => {
  console.log("window.get_active_title called");
  return Deno.core.ops.op2_get_active_window_title();
};

globalThis.window.get_inactive_titles = () => {
  console.log("window.get_inactive_titles called");
  return Deno.core.ops.op2_get_inactive_window_titles();
};

globalThis.window.close = (titlePattern) => {
  console.log("window.close called with pattern: " + titlePattern);
  return Deno.core.ops.op2_close_window(titlePattern);
};

// Also expose as standalone functions for backwards compatibility
globalThis.get_active_window_title = globalThis.window.get_active_title;
globalThis.get_inactive_window_titles = globalThis.window.get_inactive_titles;
globalThis.close_window = globalThis.window.close;

console.log("Window Plugin initialized.");
