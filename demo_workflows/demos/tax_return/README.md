# Tax Return Workflow Demo (Draft)

This folder contains a work-in-progress demo workflow for tax-return preparation (tax year 2025), focused on local processing with Ollama-based models.

## Workflow Overview

The current implementation direction is:

1. Ingest PDF documents from `/Users/user/Documents/Invoices`.
2. Run OCR and extract structured accounting fields from each document.
3. Pair related documents (for example, invoice and receipt/order detail).
4. Build normalized tax-input JSON for later form automation.
5. Emit warnings for manual review before final filing.

## Warning and Review Policy

The workflow is expected to output warning signals such as:

- possible duplicate documents
- documents that may not qualify as business expenses
- unmatched records
- low-confidence extraction/pairing results

These warnings are intended to be reviewed by a human before submission.

## Current Target URL

- `https://www.keisan.nta.go.jp/kyoutu/ky/sm/top_web#bsctrl`

## Related Files

- `PLAN.md`: detailed architecture and phased plan
- `note.md`: short memo (filing URL)
