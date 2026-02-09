// Sapphillon
// SPDX-FileCopyrightText: 2026 Yuta Takahashi
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later

use anyhow::{anyhow, Context};
use base64::Engine as _;
use deno_core::{op2, OpState};
use deno_error::JsErrorBox;
use regex::Regex;
use sapphillon_core::plugin::{CorePluginFunction, CorePluginPackage};
use sapphillon_core::proto::sapphillon::v1::{
    FunctionDefine, FunctionParameter, PluginFunction, PluginPackage,
};
use serde::Serialize;
use serde_json::{json, Map, Value};
use std::cmp;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba};

const MIN_VALID_AMOUNT: i64 = 10;
const MAX_VALID_AMOUNT: i64 = 100_000_000;

#[derive(Debug, Clone, Serialize)]
struct OcrTextOutput {
    text: String,
    ocr_engine: String,
    text_quality: f64,
    warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct OcrDocumentOutput {
    text: String,
    ocr_engine: String,
    text_quality: f64,
    warnings: Vec<String>,
    structured: Value,
    ollama_used: bool,
}

pub fn ocr_extract_text_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.ocr.extract_text".to_string(),
        function_name: "ocr.extract_text".to_string(),
        version: "".to_string(),
        description: "Extracts text from a PDF using local OCR fallbacks.".to_string(),
        permissions: vec![],
        function_define: Some(FunctionDefine {
            parameters: vec![FunctionParameter {
                name: "pdf_path".to_string(),
                r#type: "string".to_string(),
                description: "Absolute path of the PDF file".to_string(),
            }],
            returns: vec![FunctionParameter {
                name: "text".to_string(),
                r#type: "string".to_string(),
                description: "Extracted text".to_string(),
            }],
        }),
    }
}

pub fn ocr_extract_document_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.ocr.extract_document".to_string(),
        function_name: "ocr.extract_document".to_string(),
        version: "".to_string(),
        description: "Extracts OCR text and structured fields from a PDF using Ollama vision."
            .to_string(),
        permissions: vec![],
        function_define: Some(FunctionDefine {
            parameters: vec![
                FunctionParameter {
                    name: "pdf_path".to_string(),
                    r#type: "string".to_string(),
                    description: "Absolute path of the PDF file".to_string(),
                },
                FunctionParameter {
                    name: "model".to_string(),
                    r#type: "string".to_string(),
                    description: "Ollama model name (e.g. glm-ocr:latest)".to_string(),
                },
                FunctionParameter {
                    name: "base_url".to_string(),
                    r#type: "string".to_string(),
                    description: "Ollama base URL".to_string(),
                },
            ],
            returns: vec![FunctionParameter {
                name: "json".to_string(),
                r#type: "string".to_string(),
                description: "Structured OCR output in JSON string".to_string(),
            }],
        }),
    }
}

pub fn ocr_plugin_package() -> PluginPackage {
    PluginPackage {
        package_id: "app.sapphillon.core.ocr".to_string(),
        package_name: "OCR".to_string(),
        provider_id: "".to_string(),
        description: "Built-in OCR plugin with local OCR fallback and Ollama integration."
            .to_string(),
        functions: vec![
            ocr_extract_text_plugin_function(),
            ocr_extract_document_plugin_function(),
        ],
        package_version: env!("CARGO_PKG_VERSION").to_string(),
        deprecated: None,
        plugin_store_url: "BUILTIN".to_string(),
        internal_plugin: Some(true),
        installed_at: None,
        updated_at: None,
        verified: Some(true),
    }
}

pub fn core_ocr_extract_text_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.ocr.extract_text".to_string(),
        "OCRExtractText".to_string(),
        "Extracts text from a PDF using local OCR fallbacks.".to_string(),
        op2_ocr_extract_text(),
        Some(include_str!("00_ocr.js").to_string()),
    )
}

pub fn core_ocr_extract_document_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.ocr.extract_document".to_string(),
        "OCRExtractDocument".to_string(),
        "Extracts OCR text + structured fields using Ollama model call.".to_string(),
        op2_ocr_extract_document(),
        Some(include_str!("00_ocr.js").to_string()),
    )
}

pub fn core_ocr_plugin_package() -> CorePluginPackage {
    CorePluginPackage::new(
        "app.sapphillon.core.ocr".to_string(),
        "OCR".to_string(),
        vec![
            core_ocr_extract_text_plugin(),
            core_ocr_extract_document_plugin(),
        ],
    )
}

#[op2]
#[string]
fn op2_ocr_extract_text(
    _state: &mut OpState,
    #[string] pdf_path: String,
) -> std::result::Result<String, JsErrorBox> {
    let path = PathBuf::from(&pdf_path);
    let result = extract_text_with_fallback(&path)
        .map_err(|e| JsErrorBox::new("Error", format!("OCR extraction failed: {e}")))?;
    Ok(result.text)
}

#[op2]
#[string]
fn op2_ocr_extract_document(
    _state: &mut OpState,
    #[string] pdf_path: String,
    #[string] model: String,
    #[string] base_url: String,
) -> std::result::Result<String, JsErrorBox> {
    let path = PathBuf::from(&pdf_path);
    let mut text_result = extract_text_with_fallback(&path)
        .map_err(|e| JsErrorBox::new("Error", format!("OCR extraction failed: {e}")))?;
    let model = model.trim().to_string();
    let base_url = base_url.trim().to_string();
    if model.is_empty() {
        return Err(JsErrorBox::new(
            "Error",
            "Ollama model is required for ocr.extract_document",
        ));
    }
    if base_url.is_empty() {
        return Err(JsErrorBox::new(
            "Error",
            "Ollama base_url is required for ocr.extract_document",
        ));
    }

    let heuristic = heuristic_extract_document(&path, &text_result.text);
    let temp_dir = tempfile::tempdir()
        .map_err(|e| JsErrorBox::new("Error", format!("Failed to create temp dir: {e}")))?;
    let image_path = render_all_pages_image(&path, temp_dir.path())
        .map_err(|e| JsErrorBox::new("Error", format!("Failed to render PDF pages: {e}")))?;
    let llm_structured = call_ollama_document(&model, &base_url, &image_path, &text_result.text)
        .map_err(|e| JsErrorBox::new("Error", format!("Ollama request failed: {e}")))?
        .ok_or_else(|| JsErrorBox::new("Error", "Ollama did not return JSON payload"))?;

    let structured = merge_structured(heuristic, Some(llm_structured));
    text_result.warnings = dedupe_sorted(text_result.warnings);

    let payload = OcrDocumentOutput {
        text: text_result.text,
        ocr_engine: text_result.ocr_engine,
        text_quality: text_result.text_quality,
        warnings: text_result.warnings,
        structured,
        ollama_used: true,
    };

    serde_json::to_string(&payload)
        .map_err(|e| JsErrorBox::new("Error", format!("JSON serialization failed: {e}")))
}

fn run_command_output(command: &str, args: &[&str]) -> anyhow::Result<String> {
    let output = Command::new(command)
        .args(args)
        .output()
        .with_context(|| format!("failed to launch {command}"))?;

    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).to_string());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(anyhow!(
        "command failed: {} {} => {}",
        command,
        args.join(" "),
        stderr.trim()
    ))
}

fn text_quality_score(text: &str) -> f64 {
    let compact = text.trim();
    if compact.is_empty() {
        return 0.0;
    }

    let length_score = (compact.len() as f64 / 1400.0).min(1.0);
    let line_count = compact
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count();
    let line_score = (line_count as f64 / 40.0).min(1.0);

    let readable = compact
        .chars()
        .filter(|ch| ch.is_alphanumeric() || " .,:;/#-_()[]{}".contains(*ch))
        .count();
    let readable_ratio = readable as f64 / cmp::max(compact.len(), 1) as f64;

    ((length_score * 0.50) + (line_score * 0.25) + (readable_ratio * 0.25) * 1000.0).round()
        / 1000.0
}

fn extract_text_with_fallback(pdf_path: &Path) -> anyhow::Result<OcrTextOutput> {
    if !pdf_path.exists() {
        return Err(anyhow!("file not found: {}", pdf_path.display()));
    }

    let mut warnings: Vec<String> = Vec::new();
    let mut text = match run_command_output("pdftotext", &[&pdf_path.to_string_lossy(), "-"]) {
        Ok(value) => value,
        Err(err) => {
            warnings.push(format!("pdftotext_failed:{err}"));
            String::new()
        }
    };

    let mut engine = "pdftotext".to_string();
    let mut quality = text_quality_score(&text);

    if quality < 0.35 {
        if let Ok(temp_dir) = tempfile::tempdir() {
            let ocr_pdf = temp_dir.path().join("ocr.pdf");
            match run_command_output(
                "ocrmypdf",
                &[
                    "--force-ocr",
                    "-l",
                    "jpn+eng",
                    &pdf_path.to_string_lossy(),
                    &ocr_pdf.to_string_lossy(),
                ],
            ) {
                Ok(_) => {
                    match run_command_output("pdftotext", &[&ocr_pdf.to_string_lossy(), "-"]) {
                        Ok(ocr_text) => {
                            let ocr_quality = text_quality_score(&ocr_text);
                            if ocr_quality > quality {
                                text = ocr_text;
                                quality = ocr_quality;
                                engine = "ocrmypdf+pdftotext".to_string();
                            }
                        }
                        Err(err) => warnings.push(format!("pdftotext_after_ocr_failed:{err}")),
                    }
                }
                Err(err) => warnings.push(format!("ocrmypdf_failed:{err}")),
            }
        }
    }

    if quality < 0.25 {
        if let Ok(temp_dir) = tempfile::tempdir() {
            match render_first_page_image(pdf_path, temp_dir.path()) {
                Ok(image_path) => {
                    match run_command_output(
                        "tesseract",
                        &[&image_path.to_string_lossy(), "stdout", "-l", "jpn+eng"],
                    ) {
                        Ok(ocr_text) => {
                            let ocr_quality = text_quality_score(&ocr_text);
                            if ocr_quality > quality {
                                text = ocr_text;
                                quality = ocr_quality;
                                engine = "tesseract_page1".to_string();
                            }
                        }
                        Err(err) => warnings.push(format!("tesseract_failed:{err}")),
                    }
                }
                Err(err) => warnings.push(format!("render_page_failed:{err}")),
            }
        }
    }

    if quality < 0.2 {
        warnings.push("low_text_quality".to_string());
    }

    Ok(OcrTextOutput {
        text,
        ocr_engine: engine,
        text_quality: quality,
        warnings: dedupe_sorted(warnings),
    })
}

fn render_first_page_image(pdf_path: &Path, work_dir: &Path) -> anyhow::Result<PathBuf> {
    let out_prefix = work_dir.join("page1");
    let image_path = out_prefix.with_extension("png");

    if run_command_output(
        "pdftoppm",
        &[
            "-f",
            "1",
            "-singlefile",
            "-png",
            &pdf_path.to_string_lossy(),
            &out_prefix.to_string_lossy(),
        ],
    )
    .is_ok()
        && image_path.exists()
    {
        return Ok(image_path);
    }

    if run_command_output(
        "pdftocairo",
        &[
            "-f",
            "1",
            "-singlefile",
            "-png",
            &pdf_path.to_string_lossy(),
            &out_prefix.to_string_lossy(),
        ],
    )
    .is_ok()
        && image_path.exists()
    {
        return Ok(image_path);
    }

    Err(anyhow!("failed to render first PDF page: {}", image_path.display()))
}

fn render_all_pages_image(pdf_path: &Path, work_dir: &Path) -> anyhow::Result<PathBuf> {
    let out_prefix = work_dir.join("page");
    let merged_path = work_dir.join("merged.png");

    // Render all pages as separate images
    let pdftoppm_result = run_command_output(
        "pdftoppm",
        &[
            "-png",
            &pdf_path.to_string_lossy(),
            &out_prefix.to_string_lossy(),
        ],
    );

    if pdftoppm_result.is_err() {
        // Fallback to pdftocairo
        let pdftocairo_result = run_command_output(
            "pdftocairo",
            &[
                "-png",
                &pdf_path.to_string_lossy(),
                &out_prefix.to_string_lossy(),
            ],
        );
        if pdftocairo_result.is_err() {
            return Err(anyhow!("failed to render PDF pages"));
        }
    }

    // Collect all rendered page images
    let mut page_images: Vec<PathBuf> = Vec::new();
    let mut page_num = 1;
    loop {
        let page_path = out_prefix.with_extension(format!("png",)); // pdftoppm uses -1, -2, etc.
        // pdftoppm naming: page-1.png, page-2.png, etc.
        let alt_page_path = work_dir.join(format!("page-{}.png", page_num));

        if alt_page_path.exists() {
            page_images.push(alt_page_path);
            page_num += 1;
        } else if page_path.exists() {
            page_images.push(page_path);
            page_num += 1;
        } else {
            break;
        }
    }

    if page_images.is_empty() {
        // Try single page fallback
        return render_first_page_image(pdf_path, work_dir);
    }

    // Merge all pages vertically
    if page_images.len() == 1 {
        return Ok(page_images.into_iter().next().unwrap());
    }

    // Load and combine images
    let mut combined_height = 0u32;
    let mut max_width = 0u32;
    let mut images: Vec<DynamicImage> = Vec::new();

    for img_path in &page_images {
        let img = image::open(img_path)
            .with_context(|| format!("failed to load image: {}", img_path.display()))?;
        max_width = cmp::max(max_width, img.width());
        combined_height += img.height();
        images.push(img);
    }

    // Create combined image
    let mut combined_image = ImageBuffer::new(max_width, combined_height);
    let mut y_offset = 0u32;

    for img in images {
        for (x, y, pixel) in img.pixels() {
            if x < max_width {
                combined_image.put_pixel(x, y_offset + y, pixel);
            }
        }
        y_offset += img.height();
    }

    // Save combined image
    combined_image.save(&merged_path)
        .with_context(|| format!("failed to save merged image: {}", merged_path.display()))?;

    // Clean up individual page images
    for img_path in page_images {
        let _ = fs::remove_file(img_path);
    }

    Ok(merged_path)
}

fn parse_amount(raw: &str) -> Option<i64> {
    let digits: String = raw.chars().filter(|ch| ch.is_ascii_digit()).collect();
    if digits.is_empty() {
        return None;
    }

    let value = digits.parse::<i64>().ok()?;
    if !(MIN_VALID_AMOUNT..=MAX_VALID_AMOUNT).contains(&value) {
        return None;
    }
    Some(value)
}

fn find_amount_near_keywords(text: &str, keywords: &[&str]) -> Option<i64> {
    let amount_re =
        Regex::new(r"(?:[¥￥]?\s*)([0-9]{1,3}(?:,[0-9]{3})+|[0-9]{3,})(?:\s*円)?").ok()?;
    let lines: Vec<&str> = text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();

    for keyword in keywords {
        let key = keyword.to_lowercase();
        for idx in 0..lines.len() {
            if !lines[idx].to_lowercase().contains(&key) {
                continue;
            }
            let start = idx.saturating_sub(1);
            let end = cmp::min(idx + 2, lines.len());
            for line in &lines[start..end] {
                if let Some(caps) = amount_re.captures(line)
                    && let Some(m) = caps.get(1)
                    && let Some(value) = parse_amount(m.as_str())
                {
                    return Some(value);
                }
            }
        }
    }

    let mut max_amount: Option<i64> = None;
    for caps in amount_re.captures_iter(text) {
        if let Some(m) = caps.get(1)
            && let Some(value) = parse_amount(m.as_str())
        {
            max_amount = Some(max_amount.map_or(value, |prev| cmp::max(prev, value)));
        }
    }
    max_amount
}

fn extract_issue_date(text: &str) -> Option<String> {
    let patterns = [
        Regex::new(r"(?P<y>20\d{2})[/-](?P<m>\d{1,2})[/-](?P<d>\d{1,2})").ok()?,
        Regex::new(r"(?P<y>20\d{2})年\s*(?P<m>\d{1,2})月\s*(?P<d>\d{1,2})日").ok()?,
        Regex::new(r"(?P<y>20\d{2})\.(?P<m>\d{1,2})\.(?P<d>\d{1,2})").ok()?,
    ];

    for pattern in patterns {
        if let Some(caps) = pattern.captures(text) {
            let y = caps.name("y")?.as_str().parse::<i32>().ok()?;
            let m = caps.name("m")?.as_str().parse::<u32>().ok()?;
            let d = caps.name("d")?.as_str().parse::<u32>().ok()?;
            if let Some(date) = chrono::NaiveDate::from_ymd_opt(y, m, d) {
                return Some(date.format("%Y-%m-%d").to_string());
            }
        }
    }
    None
}

fn extract_invoice_number(text: &str) -> Option<String> {
    let patterns = [
        Regex::new(r"(?:請求書(?:番号)?|invoice(?:\s*no)?|伝票番号|No\.?)[^A-Za-z0-9]{0,8}([A-Za-z0-9-]{4,})").ok()?,
        Regex::new(r"\b(MB[0-9]{8})\b").ok()?,
        Regex::new(r"\bW[0-9]{10}\b").ok()?,
    ];

    for pattern in patterns {
        if let Some(caps) = pattern.captures(text)
            && let Some(m) = caps.get(1)
        {
            return Some(m.as_str().to_string());
        }
    }
    None
}

fn extract_order_number(text: &str) -> Option<String> {
    let patterns = [
        Regex::new(r"(?:注文番号|order\s*id|order\s*number)[^A-Za-z0-9]{0,8}([A-Za-z0-9-]{6,})")
            .ok()?,
        Regex::new(r"\b([0-9]{3}-[0-9]{7}-[0-9]{7})\b").ok()?,
    ];

    for pattern in patterns {
        if let Some(caps) = pattern.captures(text)
            && let Some(m) = caps.get(1)
        {
            return Some(m.as_str().to_string());
        }
    }
    None
}

fn extract_vendor_name(text: &str) -> Option<String> {
    let blocked = [
        "invoice",
        "receipt",
        "請求書",
        "領収書",
        "注文番号",
        "合計",
        "total",
    ];

    for line in text.lines().take(40) {
        let candidate = line.trim();
        if candidate.len() < 2 || candidate.len() > 80 {
            continue;
        }
        let digit_count = candidate.chars().filter(|ch| ch.is_ascii_digit()).count();
        if digit_count > candidate.len() * 2 / 5 {
            continue;
        }
        let lowered = candidate.to_lowercase();
        if blocked.iter().any(|word| lowered.contains(word)) {
            continue;
        }
        return Some(candidate.to_string());
    }

    None
}

fn detect_doc_type(file_name: &str, text: &str) -> String {
    let lower_name = file_name.to_lowercase();
    let corpus = format!(
        "{}\n{}",
        file_name,
        &text.chars().take(7000).collect::<String>()
    )
    .to_lowercase();

    let mut score_invoice = 0;
    let mut score_receipt = 0;
    let mut score_order = 0;

    for keyword in ["請求書", "invoice", "請求金額", "amount due", "bill to"] {
        if corpus.contains(&keyword.to_lowercase()) {
            score_invoice += 1;
        }
    }
    for keyword in [
        "領収書",
        "receipt",
        "支払",
        "payment received",
        "ご利用明細",
    ] {
        if corpus.contains(&keyword.to_lowercase()) {
            score_receipt += 1;
        }
    }
    for keyword in [
        "注文の詳細",
        "order detail",
        "注文番号",
        "amazon",
        "order summary",
    ] {
        if corpus.contains(&keyword.to_lowercase()) {
            score_order += 1;
        }
    }

    if lower_name.contains("receipt") || lower_name.contains("recipt") {
        score_receipt += 2;
    }
    if Regex::new(r"^mb[0-9]{6,}\.pdf$")
        .map(|re| re.is_match(&lower_name))
        .unwrap_or(false)
    {
        score_receipt += 2;
    }

    let mut best = ("unknown", 0);
    for candidate in [
        ("invoice", score_invoice),
        ("receipt", score_receipt),
        ("order_detail", score_order),
    ] {
        if candidate.1 > best.1 {
            best = candidate;
        }
    }

    best.0.to_string()
}

fn heuristic_extract_document(path: &Path, text: &str) -> Value {
    let doc_type = detect_doc_type(
        &path.file_name().unwrap_or_default().to_string_lossy(),
        text,
    );

    let total = find_amount_near_keywords(
        text,
        &[
            "合計",
            "請求金額",
            "お支払い金額",
            "総計",
            "total",
            "amount due",
            "税込",
        ],
    );
    let subtotal = find_amount_near_keywords(text, &["小計", "subtotal", "税抜"]);
    let tax = find_amount_near_keywords(text, &["税", "消費税", "vat", "tax"]);

    json!({
        "doc_type": doc_type,
        "vendor_name": extract_vendor_name(text),
        "issue_date": extract_issue_date(text),
        "currency": if text.contains('円') || text.contains('¥') || text.contains('￥') { Some("JPY") } else { None::<&str> },
        "subtotal_amount": subtotal,
        "tax_amount": tax,
        "total_amount": total,
        "invoice_number": extract_invoice_number(text),
        "order_number": extract_order_number(text),
        "payment_method": Value::Null,
        "line_items": Value::Array(vec![]),
        "warnings": Value::Array(vec![]),
    })
}

fn parse_json_candidate(raw: &str) -> Option<Value> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    let stripped = trimmed
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    if let Ok(value) = serde_json::from_str::<Value>(stripped)
        && value.is_object()
    {
        return Some(value);
    }

    let start = stripped.find('{')?;
    let end = stripped.rfind('}')?;
    if end <= start {
        return None;
    }

    let fragment = &stripped[start..=end];
    serde_json::from_str::<Value>(fragment)
        .ok()
        .filter(|value| value.is_object())
}

fn call_ollama_document(
    model: &str,
    base_url: &str,
    image_path: &Path,
    text: &str,
) -> anyhow::Result<Option<Value>> {
    let image_bytes = fs::read(image_path)
        .with_context(|| format!("failed to read image: {}", image_path.display()))?;
    let image_b64 = base64::engine::general_purpose::STANDARD.encode(image_bytes);

    let prompt = format!(
        "Extract accounting fields and return strict JSON only. \
         Keys: doc_type,vendor_name,issue_date,currency,subtotal_amount,tax_amount,total_amount,invoice_number,order_number,payment_method,line_items,warnings. \
         Use null when unknown and integer amounts in JPY.\n\nOCR_TEXT:\n{}",
        text.chars().take(6000).collect::<String>()
    );

    let payload = json!({
        "model": model,
        "stream": false,
        "messages": [{
            "role": "user",
            "content": prompt,
            "images": [image_b64]
        }]
    });

    let url = format!("{}/api/chat", base_url.trim_end_matches('/'));
    let mut response = ureq::post(&url)
        .header("Content-Type", "application/json")
        .send(serde_json::to_string(&payload)?)
        .map_err(|err| anyhow!("ollama request failed: {err}"))?;

    if response.status() != 200 {
        let status = response.status();
        let body = response.body_mut().read_to_string().unwrap_or_default();
        return Err(anyhow!("ollama status {}: {}", status, body));
    }

    let body = response
        .body_mut()
        .read_to_string()
        .map_err(|err| anyhow!("failed to read ollama response: {err}"))?;

    let root: Value = serde_json::from_str(&body)
        .map_err(|err| anyhow!("invalid ollama response json: {err}"))?;
    let content = root
        .get("message")
        .and_then(|msg| msg.get("content"))
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("missing message.content in ollama response"))?;

    Ok(parse_json_candidate(content))
}

fn merge_structured(heuristic: Value, llm: Option<Value>) -> Value {
    let Some(llm_value) = llm else {
        return heuristic;
    };

    let mut base = match heuristic {
        Value::Object(map) => map,
        _ => Map::new(),
    };
    let llm_map = match llm_value {
        Value::Object(map) => map,
        _ => return Value::Object(base),
    };

    let keys = [
        "doc_type",
        "vendor_name",
        "issue_date",
        "currency",
        "subtotal_amount",
        "tax_amount",
        "total_amount",
        "invoice_number",
        "order_number",
        "payment_method",
        "line_items",
        "warnings",
    ];

    for key in keys {
        if let Some(value) = llm_map.get(key) {
            if value.is_null() {
                continue;
            }
            if let Some(s) = value.as_str()
                && s.trim().is_empty()
            {
                continue;
            }
            base.insert(key.to_string(), value.clone());
        }
    }

    Value::Object(base)
}

fn dedupe_sorted(values: Vec<String>) -> Vec<String> {
    let mut values = values
        .into_iter()
        .filter(|value| !value.trim().is_empty())
        .collect::<Vec<_>>();
    values.sort();
    values.dedup();
    values
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_amount() {
        assert_eq!(parse_amount("2,847"), Some(2847));
        assert_eq!(parse_amount("3011103003992"), None);
        assert_eq!(parse_amount("abc"), None);
    }

    #[test]
    fn test_parse_json_candidate() {
        let raw = "```json\n{\"doc_type\":\"invoice\"}\n```";
        let parsed = parse_json_candidate(raw).unwrap();
        assert_eq!(parsed["doc_type"], "invoice");
    }

    #[test]
    fn test_detect_doc_type_by_filename() {
        let ty = detect_doc_type("Apple_Care_Recipt.pdf", "");
        assert_eq!(ty, "receipt");

        let ty = detect_doc_type("MB48595477.pdf", "");
        assert_eq!(ty, "receipt");
    }
}
