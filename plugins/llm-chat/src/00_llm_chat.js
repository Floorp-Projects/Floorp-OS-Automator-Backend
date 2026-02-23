// LLM Chat Plugin for Sapphillon (Ollama backend)

(function () {
  const core = Deno.core;

  /**
   * Send a chat request to LLM via local Ollama API.
   * @param {string} systemPrompt - The system prompt for the AI
   * @param {string} userPrompt - The user prompt/message
   * @returns {string} AI response content
   */
  function chat(systemPrompt, userPrompt) {
    return core.ops.op2_llm_chat_chat(systemPrompt, userPrompt);
  }

  globalThis.llm_chat = {
    chat,
  };
})();
