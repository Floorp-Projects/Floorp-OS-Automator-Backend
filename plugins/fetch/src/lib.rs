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
use anyhow::{Result, Error};

fn fetch(url: &str) -> Result<String> {
    let response = reqwest::blocking::get(url)?;
    if response.status().is_success() {
        let body = response.text()?;
        Ok(body)
    } else {
        Err(Error::msg(format!("Failed to fetch URL: {url}")))
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
