// @ts-nocheck
// JavaScript glue exposing Git operations to workflows

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

globalThis.git = {
  getDiff: gitGetDiff,
  getStatus: gitGetStatus,
  getBranch: gitGetBranch,
  getCommitLog: gitGetCommitLog,
  add: gitAdd,
  commit: gitCommit,
  push: gitPush,
};
