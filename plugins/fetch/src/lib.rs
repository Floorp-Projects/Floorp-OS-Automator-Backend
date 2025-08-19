use std::string;

// Sapphillon
// Copyright 2025 Yuta Takahashi
//
// This file is part of Sapphillon
//
// Sapphillon is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.
//
use deno_core::{op2, extension, error::AnyError};
use anyhow::Result;
use deno_error::JsErrorBox;
use sapphillon_core::plugin::CorePluginFunction;

pub fn fetch_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.fetch".to_string(),
        "Fetch".to_string(),
        "Fetches the content of a URL using reqwest and returns it as a string.".to_string(),
        op2_fetch(),
        Some(include_str!("00_fetch.js").to_string())
    )
}

#[op2]
#[string]
fn op2_fetch(#[string] url: String) -> Result<String, JsErrorBox> {
    let result = fetch(&url);
    match result {
        Ok(body) => Ok(body),
        Err(e) => Err(JsErrorBox::new("Error", e.to_string())),
    }
}

fn fetch(url: &str) -> Result<String> {
    let response = reqwest::blocking::get(url)?;
    if response.status().is_success() {
        let body = response.text()?;
        Ok(body)
    } else {
        Err(AnyError::msg(format!("Failed to fetch URL: {url}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch() {
        let url = "https://dummyjson.com/test";
        let result = fetch(url);
        assert!(result.is_ok());
        let body = result.unwrap();
        assert!(body.contains("ok"));
        println!("Fetched content: {body}");
        
    }
}
