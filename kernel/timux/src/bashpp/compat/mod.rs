//! Bash++ Compatibility Layer — runs .sh scripts unmodified
//! Translates bash constructs to Bash++ AST at parse time
extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use crate::bashpp::lexer::Lexer;
use crate::bashpp::parser::Parser;
use crate::bashpp::runtime::Runtime;

/// Result of running a bash-compat script
#[derive(Debug)]
pub struct CompatResult {
    pub exit_code: i32,
    pub output:    Vec<String>,
    pub errors:    Vec<String>,
}

/// Bash compatibility shims — transforms bash-specific syntax
/// before handing off to the Bash++ parser
pub struct BashCompat;

impl BashCompat {
    /// Preprocess a bash script to Bash++ compatible form
    pub fn preprocess(source: &str) -> String {
        let mut out = String::new();
        for line in source.lines() {
            let line = line.trim_end();
            // Skip shebang
            if line.starts_with("#!") { out.push('\n'); continue; }
            // Transform bash-specific patterns
            let transformed = Self::transform_line(line);
            out.push_str(&transformed);
            out.push('\n');
        }
        out
    }

    fn transform_line(line: &str) -> String {
        let mut l = String::from(line);
        // [ expr ] → if-condition compat — leave as-is, parser handles it
        // function name() { → fn name() {
        if l.contains("function ") {
            l = l.replace("function ", "fn ");
        }
        // local var=val → let var := val
        if l.trim_start().starts_with("local ") {
            l = l.replacen("local ", "let ", 1);
            if l.contains('=') && !l.contains(":=") {
                l = l.replacen('=', " := ", 1);
            }
        }
        // declare -x → export
        if l.trim_start().starts_with("declare -x ") {
            l = l.replacen("declare -x ", "export ", 1);
        }
        // declare -i → let (integer)
        if l.trim_start().starts_with("declare -i ") {
            l = l.replacen("declare -i ", "let ", 1);
        }
        // readonly → export (closest equivalent)
        if l.trim_start().starts_with("readonly ") {
            l = l.replacen("readonly ", "export ", 1);
        }
        // printf "fmt" args → echo args (simplified)
        if l.trim_start().starts_with("printf ") {
            l = l.replacen("printf ", "echo ", 1);
        }
        // [[ expr ]] → expr (strip double brackets)
        while l.contains("[[") && l.contains("]]") {
            l = l.replacen("[[", "", 1).replacen("]]", "", 1);
        }
        // $((expr)) → $(expr) (arithmetic — simplified)
        while l.contains("$((") {
            l = l.replacen("$((", "$(", 1).replacen("))", ")", 1);
        }
        l
    }

    /// Run a bash-compatible script string
    pub fn run(source: &str) -> CompatResult {
        let preprocessed = Self::preprocess(source);
        let mut lexer = Lexer::new(&preprocessed);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse();
        let mut runtime = Runtime::new();
        runtime.exec_block(&ast);
        let output = runtime.output.drain();
        CompatResult {
            exit_code: runtime.env.last_exit,
            output,
            errors: Vec::new(),
        }
    }
}

/// Detect if a script is bash-compatible (has shebang or bash-isms)
pub fn is_bash_script(source: &str) -> bool {
    let first = source.lines().next().unwrap_or("");
    first.starts_with("#!/bin/bash")
        || first.starts_with("#!/usr/bin/env bash")
        || first.starts_with("#!/bin/sh")
        || source.contains("function ")
        || source.contains("local ")
        || source.contains("[[")
}
