// VSCode Plugin for Sapphillon

globalThis.vscode = globalThis.vscode || {};

globalThis.vscode.open_folder = (path) => {
  return Deno.core.ops.op2_vscode_open_folder(path);
};

globalThis.vscode.open_file = (path) => {
  return Deno.core.ops.op2_vscode_open_file(path);
};

globalThis.vscode.write_file = (path, content) => {
  return Deno.core.ops.op2_vscode_write_file(path, content);
};

globalThis.vscode.close_workspace = () => {
  return Deno.core.ops.op2_vscode_close_workspace();
};

globalThis.vscode.get_active_file_content = () => {
  return Deno.core.ops.op2_vscode_get_active_file_content();
};

globalThis.vscode.get_workspace_path = () => {
  return Deno.core.ops.op2_vscode_get_workspace_path();
};
