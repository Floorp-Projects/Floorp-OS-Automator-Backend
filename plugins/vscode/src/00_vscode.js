console.log("VSCode Plugin script loading...");

globalThis.vscode = globalThis.vscode || {};

globalThis.vscode.open_folder = (path) => {
  console.log(`vscode.open_folder called with path: ${path}`);
  return Deno.core.ops.op2_vscode_open_folder(path);
};
globalThis.vscode.open_file = (path) => {
  console.log(`vscode.open_file called with path: ${path}`);
  return Deno.core.ops.op2_vscode_open_file(path);
};
globalThis.vscode.write_file = (path, content) => {
  console.log(
    `vscode.write_file called with path: ${path}, content length: ${
      content ? content.length : "null"
    }`
  );
  return Deno.core.ops.op2_vscode_write_file(path, content);
};
globalThis.vscode.close_workspace = () => {
  console.log("vscode.close_workspace called");
  return Deno.core.ops.op2_vscode_close_workspace();
};
globalThis.vscode.get_active_file_content = () => {
  console.log("vscode.get_active_file_content called");
  return Deno.core.ops.op2_vscode_get_active_file_content();
};
globalThis.vscode.get_workspace_path = () => {
  console.log("vscode.get_workspace_path called");
  return Deno.core.ops.op2_vscode_get_workspace_path();
};

console.log("VSCode Plugin initialized (flat style + debug).");
