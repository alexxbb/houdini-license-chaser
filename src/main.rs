#![allow(unused)]

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

fn run_sesictrl(hfs: &Path) -> Result<String> {
    let output = std::process::Command::new(hfs.join("bin").join("sesictrl"))
        .arg("print-license")
        .output()?
        .stdout;
    let output = String::from_utf8_lossy(&output);

    let regex = regex_lite::RegexBuilder::new(
        r#"(.*)(\d+) license\(s\) in use\. (\d+) licenses free. (\d+) in total.*"#,
    );
    let regex = regex_lite::RegexBuilder::new(r#".*Lic .*?:.*?\d+ \"(?<app>.+)\".*"#)
        .dot_matches_new_line(false)
        .multi_line(true)
        .crlf(true)
        .build()?;
    for m in regex.captures_iter(&output) {
        dbg!(m.get(1));
    }

    Ok("".to_string())
}

fn main() -> Result<()> {
    let _ = dotenv::dotenv()?;

    let hfs = PathBuf::from(std::env::var("HFS").context("HFS Variable not set")?);

    let out = run_sesictrl(&hfs)?;
    dbg!(out);
    Ok(())
}
