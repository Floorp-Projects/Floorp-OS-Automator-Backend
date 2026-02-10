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
use image::GenericImageView;
use image::codecs::jpeg::JpegEncoder;

/// Maximum width/height for images sent to Ollama (pixels).
const MAX_IMAGE_DIMENSION: u32 = 1024;
/// Target JPEG quality when compressing for Ollama (0-100).
const JPEG_QUALITY: u8 = 80;

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

    // Render each page as a separate image (no merging)
    let page_images = render_pages_separately(&path, temp_dir.path())
        .map_err(|e| JsErrorBox::new("Error", format!("Failed to render PDF pages: {e}")))?;

    log::info!(
        "Processing {} page(s) individually for {}",
        page_images.len(),
        path.display()
    );

    // Call Ollama once per page and merge the results
    let llm_structured =
        call_ollama_per_page(&model, &base_url, &page_images, &text_result.text)
            .map_err(|e| JsErrorBox::new("Error", format!("Ollama request failed: {e}")))?;

    let structured = merge_structured(heuristic, llm_structured);
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
            "-r",
            "150",
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
            "-r",
            "150",
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

/// Render each PDF page as a separate PNG image (at 150 DPI).
/// Returns a Vec of paths, one per page, in page order.
fn render_pages_separately(pdf_path: &Path, work_dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let out_prefix = work_dir.join("page");

    // Render all pages as separate images (at controlled DPI)
    let pdftoppm_result = run_command_output(
        "pdftoppm",
        &[
            "-r",
            "150",
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
                "-r",
                "150",
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
        let page_path = out_prefix.with_extension("png"); // pdftoppm uses -1, -2, etc.
        // pdftoppm naming: page-1.png, page-2.png, etc.
        let alt_page_path = work_dir.join(format!("page-{}.png", page_num));

        if alt_page_path.exists() {
            page_images.push(alt_page_path);
            page_num += 1;
        } else if page_path.exists() && page_num == 1 {
            page_images.push(page_path);
            page_num += 1;
        } else {
            break;
        }
    }

    if page_images.is_empty() {
        // Try single page fallback
        let single = render_first_page_image(pdf_path, work_dir)?;
        return Ok(vec![single]);
    }

    log::info!(
        "Rendered {} page(s) separately from {}",
        page_images.len(),
        pdf_path.display()
    );

    Ok(page_images)
}

/// Downscale an image if it exceeds MAX_IMAGE_DIMENSION on either axis.
/// Returns the path to a JPEG-compressed smaller image, or None if no resize was needed.
fn downscale_if_needed(image_path: &Path) -> anyhow::Result<Option<PathBuf>> {
    let img = image::open(image_path)
        .with_context(|| format!("failed to open image for downscale: {}", image_path.display()))?;

    let (w, h) = img.dimensions();
    if w <= MAX_IMAGE_DIMENSION && h <= MAX_IMAGE_DIMENSION {
        return Ok(None);
    }

    // Calculate new dimensions preserving aspect ratio
    let scale = f64::min(
        MAX_IMAGE_DIMENSION as f64 / w as f64,
        MAX_IMAGE_DIMENSION as f64 / h as f64,
    );
    let new_w = (w as f64 * scale).round() as u32;
    let new_h = (h as f64 * scale).round() as u32;

    // Round both dimensions DOWN to nearest multiple of 28.
    // Vision transformers split images into patches (typically 14px or 28px).
    // Dimensions not divisible by the patch size cause GGML_ASSERT failures
    // in the vision encoder (e.g. glm-ocr / CogVLM2).
    const PATCH_ALIGN: u32 = 28;
    // CogVLM2 uses 14px patches with 2×2 merge → 28px effective patches.
    // The model's position embeddings support at most ~900 merged patches.
    // Exceeding this causes GGML_ASSERT(a->ne[2] * 4 == b->ne[0]).
    const MAX_MERGED_PATCHES: u32 = 900;

    let mut new_w = (new_w / PATCH_ALIGN) * PATCH_ALIGN;
    let mut new_h = (new_h / PATCH_ALIGN) * PATCH_ALIGN;
    // Safety: ensure we don't go to zero
    new_w = new_w.max(PATCH_ALIGN);
    new_h = new_h.max(PATCH_ALIGN);

    // If total merged patches exceed the model's capacity, scale down further
    let patches_w = new_w / PATCH_ALIGN;
    let patches_h = new_h / PATCH_ALIGN;
    if patches_w * patches_h > MAX_MERGED_PATCHES {
        let shrink = ((MAX_MERGED_PATCHES as f64) / (patches_w * patches_h) as f64).sqrt();
        new_w = ((new_w as f64 * shrink) as u32 / PATCH_ALIGN) * PATCH_ALIGN;
        new_h = ((new_h as f64 * shrink) as u32 / PATCH_ALIGN) * PATCH_ALIGN;
        new_w = new_w.max(PATCH_ALIGN);
        new_h = new_h.max(PATCH_ALIGN);
    }

    log::info!(
        "Downscaling image {}x{} -> {}x{} for Ollama",
        w, h, new_w, new_h
    );

    let resized = img.resize(new_w, new_h, image::imageops::FilterType::Lanczos3);

    // Save as JPEG for much smaller file size
    let out_path = image_path.with_extension("compressed.jpg");
    let file = fs::File::create(&out_path)
        .with_context(|| format!("failed to create compressed image: {}", out_path.display()))?;
    let mut buf = std::io::BufWriter::new(file);
    let encoder = JpegEncoder::new_with_quality(&mut buf, JPEG_QUALITY);
    resized
        .write_with_encoder(encoder)
        .with_context(|| "failed to write compressed JPEG")?;

    Ok(Some(out_path))
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

    // Strip markdown code fences (```json ... ``` or ``` ... ```)
    let stripped = trimmed
        .trim_start_matches("```json")
        .trim_start_matches("```JSON")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    // Try direct parse first
    if let Ok(value) = serde_json::from_str::<Value>(stripped)
        && value.is_object()
    {
        return Some(value);
    }

    // Fix unquoted / partially-quoted keys BEFORE brace-matching,
    // because stray quotes in `identifier":` confuse the in_string tracker.
    let fixed = fix_unquoted_json_keys(stripped);

    // Try parsing the fixed text directly
    if let Ok(value) = serde_json::from_str::<Value>(&fixed)
        && value.is_object()
    {
        return Some(value);
    }

    // Find the outermost { ... } by brace-matching on the fixed text
    let bytes = fixed.as_bytes();
    let start = bytes.iter().position(|&b| b == b'{')?;
    let mut depth = 0i32;
    let mut end: Option<usize> = None;
    let mut in_string = false;
    let mut escape_next = false;

    for (i, &b) in bytes.iter().enumerate().skip(start) {
        if escape_next {
            escape_next = false;
            continue;
        }
        if b == b'\\' && in_string {
            escape_next = true;
            continue;
        }
        if b == b'"' {
            in_string = !in_string;
            continue;
        }
        if in_string {
            continue;
        }
        if b == b'{' {
            depth += 1;
        } else if b == b'}' {
            depth -= 1;
            if depth == 0 {
                end = Some(i);
                break;
            }
        }
    }

    let end = end?;
    let fragment = &fixed[start..=end];

    serde_json::from_str::<Value>(fragment)
        .ok()
        .filter(|value| value.is_object())
}

/// Normalize Japanese keys returned by the LLM into the expected English keys.
/// e.g. "税抜金額" → "subtotal_amount", "消費税" → "tax_amount"
fn normalize_japanese_keys(mut val: Value) -> Value {
    let key_map: &[(&str, &str)] = &[
        ("税抜金額", "subtotal_amount"),
        ("税_amount", "tax_amount"),
        ("消費税", "tax_amount"),
        ("税込金額", "total_amount"),
        ("合計金額", "total_amount"),
        ("合計", "total_amount"),
        ("小計", "subtotal_amount"),
        ("請求番号", "invoice_number"),
        ("注文番号", "order_number"),
        ("支払方法", "payment_method"),
        ("発行日", "issue_date"),
        ("通貨", "currency"),
        ("取引先", "vendor_name"),
        ("書類種別", "doc_type"),
        ("明細", "line_items"),
        ("品名", "description"),
        ("数量", "quantity"),
        ("単価", "unit_price"),
        ("金額", "amount"),
    ];

    if let Value::Object(ref mut map) = val {
        let keys_to_fix: Vec<(String, String)> = map
            .keys()
            .filter_map(|k| {
                key_map
                    .iter()
                    .find(|(jp, _)| k == *jp)
                    .map(|(_, en)| (k.clone(), en.to_string()))
            })
            .collect();

        for (jp_key, en_key) in keys_to_fix {
            if !map.contains_key(&en_key) {
                if let Some(v) = map.remove(&jp_key) {
                    log::info!("Normalized Japanese key: \"{}\" → \"{}\"", jp_key, en_key);
                    map.insert(en_key, v);
                }
            } else {
                // English key already exists; discard the Japanese duplicate
                map.remove(&jp_key);
            }
        }

        // Recursively normalize arrays (e.g. line_items)
        for v in map.values_mut() {
            match v {
                Value::Array(arr) => {
                    for item in arr.iter_mut() {
                        *item = normalize_japanese_keys(item.clone());
                    }
                }
                Value::Object(_) => {
                    *v = normalize_japanese_keys(v.clone());
                }
                _ => {}
            }
        }
    }

    val
}

/// Extract a numeric amount from a JSON value (number or string like "¥4,345").
fn extract_amount_from_value(v: &Value) -> Option<i64> {
    match v {
        Value::Number(n) => n.as_i64().or_else(|| n.as_f64().map(|f| f.round() as i64)),
        Value::String(s) => parse_amount(s),
        _ => None,
    }
}

/// Validate and fix amount consistency among subtotal, tax, and total.
///
/// Common LLM errors:
///   - total == subtotal (tax forgotten): fix → total = subtotal + tax
///   - total == subtotal + tax but subtotal is missing: fix → subtotal = total - tax
///   - tax is exactly 10% of subtotal but total is wrong: fix accordingly
fn fix_amount_consistency(mut val: Value) -> Value {
    if let Value::Object(ref mut map) = val {
        let subtotal = map.get("subtotal_amount").and_then(extract_amount_from_value);
        let tax = map.get("tax_amount").and_then(extract_amount_from_value);
        let total = map.get("total_amount").and_then(extract_amount_from_value);

        match (subtotal, tax, total) {
            (Some(sub), Some(tx), Some(tot)) => {
                let expected = sub + tx;
                if tot != expected && expected > 0 {
                    // total doesn't match subtotal + tax → model likely copied the wrong field
                    log::warn!(
                        "Amount inconsistency: subtotal({}) + tax({}) = {}, but total = {}. Correcting total.",
                        sub, tx, expected, tot
                    );
                    map.insert("total_amount".to_string(), Value::Number(expected.into()));
                }
            }
            (Some(sub), None, Some(tot)) if tot > sub => {
                // tax is missing → derive it
                let derived_tax = tot - sub;
                log::info!("Derived missing tax_amount: {} (total {} - subtotal {})", derived_tax, tot, sub);
                map.insert("tax_amount".to_string(), Value::Number(derived_tax.into()));
            }
            (None, Some(tx), Some(tot)) if tot > tx => {
                // subtotal is missing → derive it
                let derived_sub = tot - tx;
                log::info!("Derived missing subtotal_amount: {} (total {} - tax {})", derived_sub, tot, tx);
                map.insert("subtotal_amount".to_string(), Value::Number(derived_sub.into()));
            }
            _ => {} // Can't validate with fewer than 2 values
        }
    }
    val
}

/// Convert JS-style object notation to valid JSON by quoting unquoted keys.
/// e.g. `{ doc_type: "invoice" }` → `{ "doc_type": "invoice" }`
fn fix_unquoted_json_keys(input: &str) -> String {
    // Regex: match an unquoted key (word chars, possibly with underscores/hyphens)
    // that appears after `{`, `[`, or `,` followed by optional whitespace, then `:`.
    // We must avoid matching inside string values.
    let mut result = String::with_capacity(input.len() + 64);
    let mut chars = input.chars().peekable();
    let mut in_string = false;
    let mut escape_next = false;

    while let Some(ch) = chars.next() {
        if escape_next {
            result.push(ch);
            escape_next = false;
            continue;
        }
        if ch == '\\' && in_string {
            result.push(ch);
            escape_next = true;
            continue;
        }
        if ch == '"' {
            in_string = !in_string;
            result.push(ch);
            continue;
        }
        if in_string {
            result.push(ch);
            continue;
        }

        // Outside a string: look for unquoted key patterns
        // An unquoted key is a sequence of word-like chars followed by `:`
        if ch.is_alphabetic() || ch == '_' {
            // Collect the identifier
            let mut ident = String::new();
            ident.push(ch);
            while let Some(&next) = chars.peek() {
                if next.is_alphanumeric() || next == '_' || next == '-' {
                    ident.push(next);
                    chars.next();
                } else {
                    break;
                }
            }
            // Skip whitespace between identifier and potential colon
            let mut ws = String::new();
            while let Some(&next) = chars.peek() {
                if next == ' ' || next == '\t' {
                    ws.push(next);
                    chars.next();
                } else {
                    break;
                }
            }
            // Check if followed by colon → it's an unquoted key
            if chars.peek() == Some(&':') {
                result.push('"');
                result.push_str(&ident);
                result.push('"');
                result.push_str(&ws);
            } else if chars.peek() == Some(&'"') {
                // Handle partially-quoted keys like: line_items": [
                // where LLM put a stray quote after the identifier
                chars.next(); // consume the stray '"'
                // Check for optional whitespace then colon
                let mut ws2 = String::new();
                while let Some(&next) = chars.peek() {
                    if next == ' ' || next == '\t' {
                        ws2.push(next);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if chars.peek() == Some(&':') {
                    // It was  identifier": → emit as "identifier":
                    result.push('"');
                    result.push_str(&ident);
                    result.push('"');
                    result.push_str(&ws);
                    result.push_str(&ws2);
                } else {
                    // Not a key pattern, output as-is
                    result.push_str(&ident);
                    result.push_str(&ws);
                    result.push('"');
                    result.push_str(&ws2);
                }
            } else {
                // Not a key, keep as-is (e.g. `true`, `false`, `null`)
                result.push_str(&ident);
                result.push_str(&ws);
            }
        } else {
            result.push(ch);
        }
    }

    result
}

fn call_ollama_document(
    model: &str,
    base_url: &str,
    image_path: &Path,
    text: &str,
) -> anyhow::Result<Option<Value>> {
    // Downscale the image before sending to Ollama to avoid 500 errors on large payloads
    let compressed_path = downscale_if_needed(image_path)?;
    let send_path = compressed_path.as_deref().unwrap_or(image_path);
    let image_bytes = fs::read(send_path)
        .with_context(|| format!("failed to read image: {}", send_path.display()))?;
    let image_b64 = base64::engine::general_purpose::STANDARD.encode(&image_bytes);
    log::info!(
        "OCR image for Ollama: {} bytes (base64: {} bytes)",
        image_bytes.len(),
        image_b64.len()
    );

    let prompt = format!(
        "You are a JSON-only extractor. Look at this document image and the OCR text below. \
         Return ONLY a single JSON object with NO explanation, NO markdown, NO extra text. \
         The JSON must have exactly these keys: \
         doc_type, vendor_name, issue_date, currency, subtotal_amount, tax_amount, total_amount, \
         invoice_number, order_number, payment_method, line_items, warnings. \
         Rules: Use null for unknown values. Amounts must be integers in JPY (no decimals). \
         line_items is an array of objects with keys: description, quantity, unit_price, amount. \
         Output must start with {{ and end with }}.\n\nOCR_TEXT:\n{}",
        text.chars().take(6000).collect::<String>()
    );

    let payload = json!({
        "model": model,
        "stream": false,
        "options": {
            "temperature": 0.1
        },
        "messages": [
            {
                "role": "system",
                "content": "You are a structured data extractor. You MUST respond with a single JSON object only. Never include explanations, markdown formatting, or any text outside the JSON object."
            },
            {
                "role": "user",
                "content": prompt,
                "images": [image_b64]
            }
        ]
    });

    let url = format!("{}/api/chat", base_url.trim_end_matches('/'));
    let body_str = serde_json::to_string(&payload)?;

    // Use an agent that does NOT treat 4xx/5xx as Err so we can read error bodies
    let agent = ureq::Agent::config_builder()
        .http_status_as_error(false)
        .build()
        .new_agent();

    // Retry up to 3 times with exponential backoff (5s, 10s, 20s)
    let body = {
        let mut last_err: Option<anyhow::Error> = None;
        let mut result_body: Option<String> = None;
        let backoff_secs: &[u64] = &[5, 10, 20];
        for attempt in 0..3u32 {
            if attempt > 0 {
                let wait = backoff_secs.get(attempt as usize - 1).copied().unwrap_or(20);
                log::info!("Retrying Ollama request (attempt {}, backoff {}s)...", attempt + 1, wait);
                std::thread::sleep(std::time::Duration::from_secs(wait));
            }
            match agent.post(&url)
                .header("Content-Type", "application/json")
                .send(&body_str)
            {
                Ok(mut response) => {
                    if response.status() != 200 {
                        let status = response.status();
                        let err_body = response.body_mut().read_to_string().unwrap_or_default();
                        log::warn!("Ollama returned status {} (attempt {}): {}", status, attempt + 1, err_body);
                        last_err = Some(anyhow!("ollama status {}: {}", status, err_body));
                        continue;
                    }
                    match response.body_mut().read_to_string() {
                        Ok(b) => { result_body = Some(b); break; }
                        Err(e) => { last_err = Some(anyhow!("failed to read ollama response: {e}")); continue; }
                    }
                }
                Err(err) => {
                    log::warn!("Ollama request error (attempt {}): {}", attempt + 1, err);
                    last_err = Some(anyhow!("ollama request failed: {err}"));
                    continue;
                }
            }
        }
        match result_body {
            Some(b) => b,
            None => return Err(last_err.unwrap_or_else(|| anyhow!("ollama request failed after retries"))),
        }
    };

    let root: Value = serde_json::from_str(&body)
        .map_err(|err| anyhow!("invalid ollama response json: {err}"))?;
    let content = root
        .get("message")
        .and_then(|msg| msg.get("content"))
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("missing message.content in ollama response"))?;

    log::debug!("Ollama raw content ({} chars): {}", content.len(), &content[..content.len().min(500)]);

    let parsed = parse_json_candidate(content);
    match &parsed {
        None => {
            log::warn!(
                "Failed to parse JSON from Ollama response ({} chars): {}",
                content.len(),
                &content[..content.len().min(800)]
            );
        }
        Some(val) => {
            // Normalize any Japanese keys that the model sometimes returns
            let normalized = normalize_japanese_keys(val.clone());
            // Fix common LLM errors: total != subtotal + tax
            let fixed = fix_amount_consistency(normalized);
            return Ok(Some(fixed));
        }
    }
    Ok(parsed)
}

/// Call Ollama once per page image and merge the structured results.
/// For single-page PDFs this behaves like the old path.
/// For multi-page PDFs each page produces its own JSON, then we merge them
/// (line_items are concatenated; scalar fields prefer the first non-null value).
fn call_ollama_per_page(
    model: &str,
    base_url: &str,
    page_images: &[PathBuf],
    text: &str,
) -> anyhow::Result<Option<Value>> {
    if page_images.is_empty() {
        return Ok(None);
    }

    let total_pages = page_images.len();
    let mut page_results: Vec<Value> = Vec::new();

    for (i, page_path) in page_images.iter().enumerate() {
        // Cooldown between pages to let Ollama release GPU memory
        if i > 0 {
            log::info!("Cooldown 2s before page {}/{}", i + 1, total_pages);
            std::thread::sleep(std::time::Duration::from_secs(2));
        }
        log::info!(
            "Sending page {}/{} to Ollama: {}",
            i + 1,
            total_pages,
            page_path.display()
        );
        match call_ollama_document(model, base_url, page_path, text) {
            Ok(Some(value)) => {
                log::info!("Page {}/{}: got structured result", i + 1, total_pages);
                page_results.push(value);
            }
            Ok(None) => {
                log::warn!("Page {}/{}: Ollama returned no JSON", i + 1, total_pages);
            }
            Err(e) => {
                log::warn!("Page {}/{}: Ollama error: {}", i + 1, total_pages, e);
                // Continue with remaining pages rather than failing entirely
            }
        }
    }

    if page_results.is_empty() {
        return Ok(None);
    }

    // Single page: return as-is
    if page_results.len() == 1 {
        return Ok(Some(page_results.into_iter().next().unwrap()));
    }

    // Multi-page: merge all page results
    Ok(Some(merge_page_results(page_results)))
}

/// Merge structured JSON results from multiple pages.
/// - Scalar fields (vendor_name, total_amount, etc.): first non-null wins
/// - line_items: concatenated from all pages
/// - warnings: concatenated and deduplicated
fn merge_page_results(pages: Vec<Value>) -> Value {
    let mut merged = Map::new();

    let scalar_keys = [
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
    ];

    // Scalar fields: take the first non-null value across pages
    for key in scalar_keys {
        for page in &pages {
            if let Some(val) = page.get(key) {
                if !val.is_null() {
                    if let Some(s) = val.as_str() {
                        if s.trim().is_empty() {
                            continue;
                        }
                    }
                    merged.insert(key.to_string(), val.clone());
                    break;
                }
            }
        }
        // If no page had a value, insert null
        merged.entry(key.to_string()).or_insert(Value::Null);
    }

    // line_items: concatenate from all pages
    let mut all_items: Vec<Value> = Vec::new();
    for page in &pages {
        if let Some(Value::Array(items)) = page.get("line_items") {
            all_items.extend(items.iter().cloned());
        }
    }
    merged.insert("line_items".to_string(), Value::Array(all_items));

    // warnings: concatenate and deduplicate
    let mut all_warnings: Vec<String> = Vec::new();
    for page in &pages {
        if let Some(Value::Array(warns)) = page.get("warnings") {
            for w in warns {
                if let Some(s) = w.as_str() {
                    all_warnings.push(s.to_string());
                }
            }
        }
    }
    all_warnings.sort();
    all_warnings.dedup();
    merged.insert(
        "warnings".to_string(),
        Value::Array(all_warnings.into_iter().map(Value::String).collect()),
    );

    Value::Object(merged)
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
    fn test_parse_json_candidate_unquoted_keys() {
        // glm-ocr often returns JS-style objects with unquoted keys
        let raw = r#"```json
{
    doc_type: "納品書",
    vendor_name: "Apple Japan, Inc.",
    issue_date: "",
    currency: "JPY",
    subtotal_amount: 146909,
    tax_amount: 14691,
    total_amount: 161600
}
```"#;
        let parsed = parse_json_candidate(raw).unwrap();
        assert_eq!(parsed["doc_type"], "納品書");
        assert_eq!(parsed["vendor_name"], "Apple Japan, Inc.");
        assert_eq!(parsed["total_amount"], 161600);
    }

    #[test]
    fn test_parse_json_candidate_mixed_keys() {
        // Some keys quoted, some not
        let raw = r#"{
    "doc_type": "請求書",
    vendor_name: "Test Corp",
    line_items: [
        {
            description: "Item 1",
            "quantity": 1,
            unit_price: 1000,
            amount: 1000
        }
    ]
}"#;
        let parsed = parse_json_candidate(raw).unwrap();
        assert_eq!(parsed["doc_type"], "請求書");
        assert_eq!(parsed["vendor_name"], "Test Corp");
        assert_eq!(parsed["line_items"][0]["description"], "Item 1");
        assert_eq!(parsed["line_items"][0]["amount"], 1000);
    }

    #[test]
    fn test_fix_unquoted_preserves_values() {
        // Ensure true/false/null values are NOT quoted
        let input = r#"{ doc_type: "invoice", is_paid: true, notes: null }"#;
        let fixed = fix_unquoted_json_keys(input);
        let parsed: Value = serde_json::from_str(&fixed).unwrap();
        assert_eq!(parsed["doc_type"], "invoice");
        assert_eq!(parsed["is_paid"], true);
        assert!(parsed["notes"].is_null());
    }

    #[test]
    fn test_fix_partially_quoted_key() {
        // glm-ocr sometimes returns: line_items": [ ... ] (stray quote after key)
        let input = r#"{
    doc_type: "请求书",
    vendor_name: "Apple Japan, Inc.",
    line_items": [
        {
            description: "iPad",
            quantity: 1,
            unit_price: 141800,
            amount: 141800
        }
    ]
}"#;
        let parsed = parse_json_candidate(input).unwrap();
        assert_eq!(parsed["doc_type"], "请求书");
        assert_eq!(parsed["line_items"][0]["description"], "iPad");
        assert_eq!(parsed["line_items"][0]["amount"], 141800);
    }

    #[test]
    fn test_detect_doc_type_by_filename() {
        let ty = detect_doc_type("Apple_Care_Recipt.pdf", "");
        assert_eq!(ty, "receipt");

        let ty = detect_doc_type("MB48595477.pdf", "");
        assert_eq!(ty, "receipt");
    }

    #[test]
    fn test_normalize_japanese_keys() {
        let input = json!({
            "doc_type": "請求書",
            "vendor_name": "Apple Japan",
            "subtotal_amount": 113800,
            "税抜金額": 103455,
            "消費税": 10345,
            "total_amount": 113800
        });
        let result = normalize_japanese_keys(input);
        // "税抜金額" should NOT overwrite existing "subtotal_amount"
        assert_eq!(result.get("subtotal_amount").unwrap(), 113800);
        // "消費税" should become "tax_amount"
        assert_eq!(result.get("tax_amount").unwrap(), 10345);
        // Japanese keys should be removed
        assert!(result.get("税抜金額").is_none());
        assert!(result.get("消費税").is_none());
    }

    #[test]
    fn test_fix_amount_consistency() {
        // Case 1: total == subtotal (tax forgotten)
        let input = json!({
            "subtotal_amount": 161600,
            "tax_amount": 14691,
            "total_amount": 161600
        });
        let fixed = fix_amount_consistency(input);
        assert_eq!(fixed["total_amount"], 176291); // 161600 + 14691

        // Case 2: already correct → no change
        let input2 = json!({
            "subtotal_amount": 100000,
            "tax_amount": 10000,
            "total_amount": 110000
        });
        let fixed2 = fix_amount_consistency(input2);
        assert_eq!(fixed2["total_amount"], 110000);

        // Case 3: string amounts with yen sign
        let input3 = json!({
            "subtotal_amount": "¥4,345",
            "tax_amount": "¥434",
            "total_amount": "¥4,345"
        });
        let fixed3 = fix_amount_consistency(input3);
        assert_eq!(fixed3["total_amount"], 4779); // 4345 + 434

        // Case 4: tax missing, total > subtotal → derive tax
        let input4 = json!({
            "subtotal_amount": 100000,
            "total_amount": 110000
        });
        let fixed4 = fix_amount_consistency(input4);
        assert_eq!(fixed4["tax_amount"], 10000);
    }
}
