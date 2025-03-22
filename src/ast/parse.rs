use crate::ast::processed_ast::{Program, StandardTransform};
use crate::ast::raw_ast::parse_ast;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "lc3.pest"]
struct LC3Parser;

pub fn get_ast(content: &str) -> Result<Program, pest::error::Error<Rule>> {
    match LC3Parser::parse(Rule::Program, content) {
        Ok(pairs) => {
            let program = parse_ast(pairs.into_iter().next().unwrap());
            let program = StandardTransform::new(true, content).transform(program);
            Ok(program)
        }
        Err(e) => Err(e),
    }
}
