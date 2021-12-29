#[derive(pest_derive::Parser)]
#[grammar = "grammar.pest"]
struct WGSLParser;

#[cfg(test)]
mod tests {
    use pest::Parser;

    use crate::*;

    #[test]
    fn bool_literal() {
        let result = WGSLParser::parse(Rule::const_literal, "true").unwrap();
        assert_eq!(result.as_str(), "true");

        let result = WGSLParser::parse(Rule::const_literal, "false").unwrap();
        assert_eq!(result.as_str(), "false");
    }

    #[test]
    fn int_literal() {
        let result = WGSLParser::parse(Rule::int_literal, "123").unwrap();
        assert_eq!(result.as_str(), "123");

        let result = WGSLParser::parse(Rule::int_literal, "-123").unwrap();
        assert_eq!(result.as_str(), "-123");
    }

    #[test]
    fn uint_literal() {
        let result = WGSLParser::parse(Rule::uint_literal, "123u").unwrap();
        assert_eq!(result.as_str(), "123u");
    }
}
