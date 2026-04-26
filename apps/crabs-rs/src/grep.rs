use grep::regex::RegexMatcher;
use grep::searcher::{Searcher, Sink, SinkMatch};
use ignore::WalkBuilder;
use std::io;

pub struct GrepEngine;

impl GrepEngine {
    pub fn search(pattern: &str, path: &str) -> Vec<String> {
        let matcher = RegexMatcher::new(pattern).unwrap();
        let mut results = Vec::new();
        let mut searcher = Searcher::new();

        // Use build() instead of build_parallel() for a simple iterator
        for entry in WalkBuilder::new(path).build().filter_map(|e| e.ok()) {
            if entry.file_type().map_or(false, |ft| ft.is_file()) {
                let _ = searcher.search_path(&matcher, entry.path(), SinkAdapter { results: &mut results });
            }
        }
        results
    }
}

struct SinkAdapter<'a> {
    results: &'a mut Vec<String>,
}

impl<'a> Sink for SinkAdapter<'a> {
    type Error = io::Error;

    fn matched(&mut self, searcher: &Searcher, mat: &SinkMatch<'_>) -> Result<bool, io::Error> {
        // SinkMatch provides access to the line bytes. The path is usually managed by the Searcher.
        // We can get the line number from the match.
        let line_num = mat.line_number().unwrap_or(0);
        self.results.push(format!("match at line {}", line_num));
        Ok(true)
    }
}
