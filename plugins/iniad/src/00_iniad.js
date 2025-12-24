// INIAD Plugin for Sapphillon

(function () {
  const core = Deno.core;

  function generatePrDescription(fileContent) {
    return core.ops.op2_iniad_generate_pr_description(fileContent);
  }

  globalThis.iniad = {
    generatePrDescription,
  };
})();
