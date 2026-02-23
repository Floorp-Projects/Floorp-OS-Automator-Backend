(function () {
  const core = Deno.core;

  function extract_text(pdfPath) {
    return core.ops.op2_ocr_extract_text(pdfPath);
  }

  function extract_document(pdfPath, model, baseUrl) {
    const useModel = model || "glm-ocr:latest";
    const useBaseUrl = baseUrl || "http://127.0.0.1:11434";
    return core.ops.op2_ocr_extract_document(pdfPath, useModel, useBaseUrl);
  }

  globalThis.ocr = {
    extract_text,
    extract_document,
  };
})();
