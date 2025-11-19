use std::collections::HashMap;

// JSDocタグ情報
#[derive(Debug)]
struct JsDocTag {
    tag_name: String,
    type_name: String,
    param_name: String,
    description: String,
}

// JSDocブロック
#[derive(Debug)]
struct JsDocBlock {
    summary: String,
    tags: Vec<JsDocTag>,
}

fn parse_jsdoc(code: &str) -> Vec<JsDocBlock> {
    let mut blocks = Vec::new();
    let code_chars: Vec<char> = code.chars().collect();
    let mut i = 0;

    while i < code_chars.len() {
        // Find "/**" delimiter
        if i + 2 < code_chars.len()
            && code_chars[i] == '/'
            && code_chars[i + 1] == '*'
            && code_chars[i + 2] == '*'
        {
            i += 3;
            let start_index = i;

            // Find "*/" delimiter
            let mut end_index = None;
            while i + 1 < code_chars.len() {
                if code_chars[i] == '*' && code_chars[i + 1] == '/' {
                    end_index = Some(i);
                    break;
                }
                i += 1;
            }

            if let Some(end_index) = end_index {
                let block_text: String = code_chars[start_index..end_index].iter().collect();
                if let Some(block) = process_jsdoc_text(&block_text) {
                    blocks.push(block);
                }
                i += 2;
            } else {
                i = code_chars.len();
            }
        } else {
            i += 1;
        }
    }
    blocks
}

/// Remove asterisks and parse JSDoc tags
fn process_jsdoc_text(raw_text: &str) -> Option<JsDocBlock> {
    let mut summary = String::new();
    let mut tags = Vec::new();

    // Remove asterisks and leading whitespace from each line
    let processed_lines: Vec<String> = raw_text
        .lines()
        .map(|line| {
            line.trim_start()
                .trim_start_matches('*')
                .trim()
                .to_string()
        })
        .filter(|line| !line.is_empty())
        .collect();

    for line in processed_lines {
        if line.starts_with('@') {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() { continue; }

            let tag_name = parts[0].trim_start_matches('@').to_string();

            // Extract type info {type}
            let type_name = if parts.len() > 1 && parts[1].starts_with('{') && parts[1].ends_with('}') {
                parts[1].trim_start_matches('{').trim_end_matches('}').to_string()
            } else {
                "".to_string()
            };

            // Extract parameter name and description
            let mut name_and_desc_parts = if type_name.is_empty() {
                &parts[1..]
            } else {
                &parts[2..]
            };

            let param_name = if !name_and_desc_parts.is_empty() && !name_and_desc_parts[0].starts_with('@') {
                let name = name_and_desc_parts[0].to_string();
                name_and_desc_parts = &name_and_desc_parts[1..];
                name
            } else {
                "".to_string()
            };

            let description = name_and_desc_parts.join(" ");

            tags.push(JsDocTag {
                tag_name,
                type_name,
                param_name,
                description,
            });
        } else {
            summary.push_str(&line);
            summary.push('\n');
        }
    }

    if summary.is_empty() && tags.is_empty() {
        None
    } else {
        Some(JsDocBlock {
            summary: summary.trim().to_string(),
            tags,
        })
    }
}

#[test]
fn test() {
    let js_code = r#"
        /**
         * 2つの数値を加算します。
         *
         * @param {number} a 最初の数値
         * @param {number} b 2番目の数値
         * @returns {number} 合計を返します。
         */
        function add(a, b) {
            return a + b;
        }

        /* これはJSDocではないブロックコメント */
        
        /**
         * メッセージを出力します。
         * @param {string} msg 出力するメッセージ
         * @returns {void}
         */
        const log = (msg) => console.log(msg);
    "#;

    println!("--- 元のJavaScriptコード ---");
    println!("{}", js_code);
    println!("\n============================\n");

    let jsdoc_blocks = parse_jsdoc(js_code);

    for (index, block) in jsdoc_blocks.iter().enumerate() {
        println!("### JSDoc Block #{}", index + 1);
        println!("  Summary: {}", block.summary);
        println!("  Tags:");
        for tag in &block.tags {
            println!("    - @{}: type={:?}, name={:?}, desc={:?}", 
                tag.tag_name, tag.type_name, tag.param_name, tag.description
            );
        }
        println!("---");
    }
}