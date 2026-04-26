use oxc_parser::Parser;
use oxc_allocator::Allocator;
use oxc_span::SourceType;
use rustpython_parser::parse;

pub struct Translator;

impl Translator {
    pub fn parse_js(code: &str) -> String {
        let allocator = Allocator::default();
        let source_type = SourceType::from_path("script.js").unwrap();
        let ret = Parser::new(&allocator, code, source_type).parse();
        
        if ret.errors.is_empty() {
            format!("AST parsed successfully: {} nodes", ret.program.len())
        } else {
            format!("Error parsing JS: {:?}", ret.errors)
        }
    }

    pub fn parse_python(code: &str) -> String {
        match parse(code, "<python>") {
            Ok(ast) => format!("AST parsed successfully: {:?}", ast),
            Err(e) => format!("Error parsing Python: {:?}", e),
        }
    }
}
