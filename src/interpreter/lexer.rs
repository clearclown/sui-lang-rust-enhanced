//! Lexer for the Sui programming language

/// Lexer for tokenizing Sui source code
pub struct Lexer;

impl Lexer {
    /// Tokenize a single line into string tokens
    ///
    /// Each line becomes a vector of tokens like ["=", "v0", "10"]
    pub fn tokenize_line(line: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            // Skip whitespace
            if chars[i].is_whitespace() {
                i += 1;
                continue;
            }

            // Comment - ignore rest of line
            if chars[i] == ';' {
                break;
            }

            // String literal
            if chars[i] == '"' {
                let start = i;
                i += 1;
                while i < chars.len() && chars[i] != '"' {
                    if chars[i] == '\\' && i + 1 < chars.len() {
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                if i < chars.len() {
                    i += 1; // Include closing quote
                }
                let token: String = chars[start..i].iter().collect();
                tokens.push(token);
                continue;
            }

            // Regular token (until whitespace)
            let start = i;
            while i < chars.len() && !chars[i].is_whitespace() {
                i += 1;
            }
            let token: String = chars[start..i].iter().collect();
            tokens.push(token);
        }

        tokens
    }

    /// Parse source code into lines of tokens
    pub fn parse(code: &str) -> Vec<Vec<String>> {
        code.lines()
            .map(|line| Self::tokenize_line(line))
            .filter(|tokens| !tokens.is_empty())
            .collect()
    }

    /// Parse a value string to determine its type
    pub fn parse_value(val: &str) -> ParsedValue {
        // Variable reference
        if val.starts_with('v') || val.starts_with('g') || val.starts_with('a') {
            if val.len() > 1 && val[1..].chars().all(|c| c.is_ascii_digit()) {
                return ParsedValue::Variable(val.to_string());
            }
        }

        // String literal
        if val.starts_with('"') && val.ends_with('"') && val.len() >= 2 {
            let inner = &val[1..val.len() - 1];
            // Process escape sequences
            let unescaped = Self::unescape_string(inner);
            return ParsedValue::String(unescaped);
        }

        // Float (contains decimal point)
        if val.contains('.') {
            if let Ok(f) = val.parse::<f64>() {
                return ParsedValue::Float(f);
            }
        }

        // Integer
        if let Ok(n) = val.parse::<i64>() {
            return ParsedValue::Integer(n);
        }

        // Fall back to string (for things like function names in P instruction)
        ParsedValue::String(val.to_string())
    }

    /// Unescape a string literal
    fn unescape_string(s: &str) -> String {
        let mut result = String::new();
        let chars: Vec<char> = s.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            if chars[i] == '\\' && i + 1 < chars.len() {
                match chars[i + 1] {
                    'n' => result.push('\n'),
                    't' => result.push('\t'),
                    'r' => result.push('\r'),
                    '\\' => result.push('\\'),
                    '"' => result.push('"'),
                    _ => {
                        result.push(chars[i]);
                        result.push(chars[i + 1]);
                    }
                }
                i += 2;
            } else {
                result.push(chars[i]);
                i += 1;
            }
        }

        result
    }
}

/// Parsed value type
#[derive(Debug, Clone, PartialEq)]
pub enum ParsedValue {
    Variable(String),
    Integer(i64),
    Float(f64),
    String(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_simple() {
        let tokens = Lexer::tokenize_line("= v0 10");
        assert_eq!(tokens, vec!["=", "v0", "10"]);
    }

    #[test]
    fn test_tokenize_with_string() {
        let tokens = Lexer::tokenize_line(". \"Hello World\"");
        assert_eq!(tokens, vec![".", "\"Hello World\""]);
    }

    #[test]
    fn test_tokenize_with_comment() {
        let tokens = Lexer::tokenize_line("= v0 10 ; this is a comment");
        assert_eq!(tokens, vec!["=", "v0", "10"]);
    }

    #[test]
    fn test_parse_value_variable() {
        assert_eq!(Lexer::parse_value("v0"), ParsedValue::Variable("v0".to_string()));
        assert_eq!(Lexer::parse_value("g10"), ParsedValue::Variable("g10".to_string()));
        assert_eq!(Lexer::parse_value("a0"), ParsedValue::Variable("a0".to_string()));
    }

    #[test]
    fn test_parse_value_integer() {
        assert_eq!(Lexer::parse_value("42"), ParsedValue::Integer(42));
        assert_eq!(Lexer::parse_value("-10"), ParsedValue::Integer(-10));
    }

    #[test]
    fn test_parse_value_float() {
        assert_eq!(Lexer::parse_value("3.14"), ParsedValue::Float(3.14));
    }

    #[test]
    fn test_parse_value_string() {
        assert_eq!(Lexer::parse_value("\"hello\""), ParsedValue::String("hello".to_string()));
    }
}
