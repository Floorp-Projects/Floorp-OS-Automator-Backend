# Tax Return Automation Demo Plan

## 1. Goal

Build a demo workflow that:

1. Scans documents in `/Users/user/Documents/Invoices`.
2. Extracts structured data from each file using local OCR/LMM.
3. Pairs invoice and receipt records.
4. Produces normalized tax input JSON.
5. Uses that JSON for form automation later (kept as a later phase).

Current priority is data extraction and pairing quality.

## 2. Scope

### In scope (now)

1. PDF document ingestion.
2. OCR and key-value extraction.
3. Document classification (`invoice`, `receipt`, `order_detail`, `unknown`).
4. Invoice-receipt pairing.
5. Normalized JSON output for tax filing input.
6. Review queue for uncertain matches.
7. Warning output (duplicate document, non-expense candidate, unmatched, low confidence).

### Out of scope (for now)

1. Final tax site submission click.
2. Full browser automation hardening.
3. Legal/tax rule engine by jurisdiction.

## 3. Model and Runtime Policy

1. Local-first model policy is required.
2. OCR extraction uses GLM OCR locally (Ollama path).
3. Pairing logic can rely on LLM judgment, but output must be validated by deterministic checks.
4. No cloud OCR dependency in the default path.
5. Tax year is fixed to `2025`.

## 4. Directory Layout

```text
Floorp-OS-Automator-Backend/demo_workflows/demos/tax_return/
  PLAN.md
  prompts/
    extract_prompt.md
    pairing_prompt.md
  schemas/
    extracted_document.schema.json
    pair_record.schema.json
    tax_input_normalized.schema.json
  scripts/
    run_extraction.sh
    run_pairing.sh
    build_tax_input.sh
  output/
    extracted_documents.jsonl
    pairs.json
    unmatched.json
    tax_input_normalized.json
    extraction_report.json
```

Note: only `PLAN.md` is required now. Other files are planned deliverables.

## 5. Pipeline Design

### Phase A: Ingestion and OCR

1. Enumerate all files under `/Users/user/Documents/Invoices` (current set is PDF only).
2. For each PDF:
   - Run text extraction.
   - If text quality is low, run OCR fallback.
3. Keep raw OCR text per document for audit.

### Phase B: Extraction and Classification

1. Use GLM OCR output to extract fields into a strict JSON schema.
2. Classify each document into one of:
   - `invoice`
   - `receipt`
   - `order_detail`
   - `unknown`
3. Keep confidence scores and extraction warnings.

### Phase C: Pairing (LLM-assisted)

1. Candidate generation with deterministic filters:
   - exact invoice/order number
   - amount proximity
   - date proximity
2. LLM decides best pair from candidates and emits reasons.
3. Validator applies rules:
   - hard checks for amount/date consistency
   - confidence threshold
4. Low-confidence records go to `unmatched.json`.

### Phase D: Normalization for Tax Input

Build `tax_input_normalized.json` from confirmed pairs and singletons.

This becomes the single source for later form input automation.

## 6. Data Contract (decided for item 1)

The extraction output must include at least:

1. `doc_id`
2. `source_path`
3. `doc_type`
4. `vendor_name`
5. `issue_date`
6. `currency`
7. `subtotal_amount`
8. `tax_amount`
9. `total_amount`
10. `invoice_number`
11. `order_number`
12. `payment_method`
13. `line_items` (name, qty, unit_price, amount, tax_rate)
14. `confidence_overall`
15. `confidence_by_field`
16. `raw_text_path`
17. `ocr_engine`
18. `warnings`

Pair record contract:

1. `pair_id`
2. `invoice_doc_id`
3. `receipt_doc_id`
4. `score`
5. `match_reasons`
6. `llm_explanation`
7. `manual_review_required`

Normalized tax input contract:

1. `tax_year`
2. `document_pairs`
3. `single_documents`
4. `totals_by_category`
5. `totals_by_tax_rate`
6. `deduction_candidates`
7. `validation_issues`
8. `warnings`
9. `generated_at`

## 7. Plugin Usage Plan

1. `app.sapphillon.core.exec.exec`:
   - run local OCR and helper scripts.
2. `app.sapphillon.core.filesystem.*`:
   - read/write intermediate JSON and logs.
3. `llm_chat.chat`:
   - LLM-assisted pairing and ambiguity resolution.
4. `floorp.*`:
   - postponed until extraction quality is stable.

## 8. Quality Gates

Before moving to form automation:

1. Extraction field completeness >= 95 percent on target samples.
2. Pairing precision >= 95 percent on manually checked sample.
3. Unmatched rate <= 10 percent (excluding truly missing pair docs).
4. `tax_input_normalized.json` passes schema validation.

## 9. Risks and Controls

1. OCR noise in mixed-language PDFs:
   - keep fallback OCR path and confidence-based review.
2. Similar amounts across multiple receipts:
   - require multi-signal match and LLM explanation.
3. Missing counterpart documents:
   - keep unmatched queue and do not force pair.
4. Sensitive financial data:
   - process locally and keep outputs in local workspace only.

## 10. Implementation Milestones

1. M1: OCR + extraction JSONL generation.
2. M2: LLM-assisted pairing + unmatched output.
3. M3: normalized tax input JSON generation.
4. M4: browser form dry-run automation (no final submit).

## 11. Immediate Next Step

Implement M1 first:

1. prepare extraction prompt,
2. run OCR on all PDFs,
3. emit `extracted_documents.jsonl` with the schema above.
