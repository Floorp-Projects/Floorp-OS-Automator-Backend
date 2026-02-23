// @ts-nocheck
// JavaScript glue exposing Git operations to workflows

// --- Basic Operations ---

function gitGetDiff(repoPath) {
  return Deno.core.ops.op_git_get_diff(repoPath);
}

function gitGetStatus(repoPath) {
  return Deno.core.ops.op_git_get_status(repoPath);
}

function gitGetBranch(repoPath) {
  return Deno.core.ops.op_git_get_branch(repoPath);
}

function gitGetCommitLog(repoPath, count) {
  return Deno.core.ops.op_git_get_commit_log(repoPath, count?.toString());
}

function gitAdd(repoPath, files) {
  return Deno.core.ops.op_git_add(repoPath, files);
}

function gitCommit(repoPath, message) {
  return Deno.core.ops.op_git_commit(repoPath, message);
}

function gitPush(repoPath) {
  return Deno.core.ops.op_git_push(repoPath);
}

// --- Branch Operations ---

function gitCheckout(repoPath, branch) {
  return Deno.core.ops.op_git_checkout(repoPath, branch);
}

function gitCreateBranch(repoPath, branchName) {
  return Deno.core.ops.op_git_create_branch(repoPath, branchName);
}

function gitDeleteBranch(repoPath, branchName) {
  return Deno.core.ops.op_git_delete_branch(repoPath, branchName);
}

function gitListBranches(repoPath) {
  return Deno.core.ops.op_git_list_branches(repoPath);
}

function gitMerge(repoPath, branch) {
  return Deno.core.ops.op_git_merge(repoPath, branch);
}

// --- Remote Operations ---

function gitPull(repoPath) {
  return Deno.core.ops.op_git_pull(repoPath);
}

function gitFetch(repoPath) {
  return Deno.core.ops.op_git_fetch(repoPath);
}

function gitGetRemotes(repoPath) {
  return Deno.core.ops.op_git_get_remotes(repoPath);
}

function gitSetRemote(repoPath, name, url) {
  return Deno.core.ops.op_git_set_remote(repoPath, name, url);
}

// --- Information Retrieval ---

function gitGetLastCommit(repoPath) {
  return Deno.core.ops.op_git_get_last_commit(repoPath);
}

function gitGetFileHistory(repoPath, filePath) {
  return Deno.core.ops.op_git_get_file_history(repoPath, filePath);
}

function gitBlame(repoPath, filePath) {
  return Deno.core.ops.op_git_blame(repoPath, filePath);
}

function gitShow(repoPath, commitHash) {
  return Deno.core.ops.op_git_show(repoPath, commitHash);
}

function gitGetTags(repoPath) {
  return Deno.core.ops.op_git_get_tags(repoPath);
}

// --- Workflow Utilities ---

function gitStash(repoPath) {
  return Deno.core.ops.op_git_stash(repoPath);
}

function gitStashPop(repoPath) {
  return Deno.core.ops.op_git_stash_pop(repoPath);
}

function gitReset(repoPath, mode, ref) {
  return Deno.core.ops.op_git_reset(repoPath, mode, ref);
}

function gitRevert(repoPath, commitHash) {
  return Deno.core.ops.op_git_revert(repoPath, commitHash);
}

function gitCherryPick(repoPath, commitHash) {
  return Deno.core.ops.op_git_cherry_pick(repoPath, commitHash);
}

// --- Export ---

globalThis.git = {
  // Basic operations
  getDiff: gitGetDiff,
  getStatus: gitGetStatus,
  getBranch: gitGetBranch,
  getCommitLog: gitGetCommitLog,
  add: gitAdd,
  commit: gitCommit,
  push: gitPush,
  // Branch operations
  checkout: gitCheckout,
  createBranch: gitCreateBranch,
  deleteBranch: gitDeleteBranch,
  listBranches: gitListBranches,
  merge: gitMerge,
  // Remote operations
  pull: gitPull,
  fetch: gitFetch,
  getRemotes: gitGetRemotes,
  setRemote: gitSetRemote,
  // Information retrieval
  getLastCommit: gitGetLastCommit,
  getFileHistory: gitGetFileHistory,
  blame: gitBlame,
  show: gitShow,
  getTags: gitGetTags,
  // Workflow utilities
  stash: gitStash,
  stashPop: gitStashPop,
  reset: gitReset,
  revert: gitRevert,
  cherryPick: gitCherryPick,
};
