// INIAD-AI-MOP Plugin for Sapphillon

(function () {
  const core = Deno.core;

  /**
   * Send a chat request to INIAD-AI-MOP OpenAI API.
   * @param {string} systemPrompt - The system prompt for the AI
   * @param {string} userPrompt - The user prompt/message
   * @returns {string} AI response content
   */
  function chat(systemPrompt, userPrompt) {
    return core.ops.op2_iniad_ai_mop_chat(systemPrompt, userPrompt);
  }

  globalThis.iniad_ai_mop = {
    chat,
  };
})();
