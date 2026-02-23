# OCR Plugin

Built-in OCR plugin for Sapphillon.

## Exposed functions

- `ocr.extract_text(pdfPath)`
  - Extracts text from a PDF with fallback (`pdftotext` -> `ocrmypdf` -> `tesseract`).

- `ocr.extract_document(pdfPath, model?, baseUrl?)`
  - Returns OCR text + heuristic structured fields.
  - Always performs Ollama vision extraction from the first PDF page image.
  - `model`/`baseUrl` can be omitted when using defaults (`glm-ocr:latest`, `http://127.0.0.1:11434`).

## Requirements

- `pdftotext`
- `ocrmypdf`
- `tesseract`
- Ollama with a vision/OCR model (e.g. `glm-ocr:latest`)
