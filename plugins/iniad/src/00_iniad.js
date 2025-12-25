// INIAD Plugin for Sapphillon

(function () {
  const core = Deno.core;

  function generatePrDescription(fileContent) {
    return core.ops.op2_iniad_generate_pr_description(fileContent);
  }

  function generateCommitMessage(diff) {
    return core.ops.op2_iniad_generate_commit_message(diff);
  }

  function analyzeWindows(windowTitlesJson) {
    return core.ops.op2_iniad_analyze_windows(windowTitlesJson);
  }

  globalThis.iniad = {
    generatePrDescription,
    generateCommitMessage,
    analyzeWindows,
  };
})();
