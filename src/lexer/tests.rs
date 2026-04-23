use crate::lexer::{Lexer, Token, TokenKind, Span};
use crate::lexer::LexError;

// 辅助函数：创建词法分析器并获取所有token
fn tokenize(source: &str) -> (Vec<Token>, Vec<LexError>) {
    let mut lexer = Lexer::new(source.to_string());
    let tokens = lexer.tokenize();
    let errors = lexer.errors;
    (tokens, errors)
}


#[cfg(test)]
mod basic_tests {
    use super::*;

    #[test]
    fn test_empty_input() {
        let (tokens, errors) = tokenize("");
        assert_eq!(tokens.len(), 1); // 只有EOF
        assert!(tokens[0].is_eof());
        assert!(errors.is_empty());
    }

    #[test]
    fn test_whitespace_only() {
        let (tokens, errors) = tokenize("   \n\t  ");
        assert_eq!(tokens.len(), 1); // 只有EOF
        assert!(tokens[0].is_eof());
        assert!(errors.is_empty());
    }

    #[test]
    fn test_single_plus() {
        let (tokens, errors) = tokenize("+");
        assert_eq!(tokens.len(), 2); // PLUS + EOF
        assert_eq!(tokens[0].kind, TokenKind::Plus);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_operators() {
        let (tokens, errors) = tokenize("+-*/=");
        assert_eq!(tokens.len(), 6); // 5个运算符 + EOF
        assert_eq!(tokens[0].kind, TokenKind::Plus);
        assert_eq!(tokens[1].kind, TokenKind::Minus);
        assert_eq!(tokens[2].kind, TokenKind::Star);
        assert_eq!(tokens[3].kind, TokenKind::Slash);
        assert_eq!(tokens[4].kind, TokenKind::Assign);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_separators() {
        let (tokens, errors) = tokenize(";,(){}[]");
        assert_eq!(tokens.len(), 9); // 8个分隔符 + EOF
        assert_eq!(tokens[0].kind, TokenKind::Semicolon);
        assert_eq!(tokens[1].kind, TokenKind::Comma);
        assert_eq!(tokens[2].kind, TokenKind::LParen);
        assert_eq!(tokens[3].kind, TokenKind::RParen);
        assert_eq!(tokens[4].kind, TokenKind::LBrace);
        assert_eq!(tokens[5].kind, TokenKind::RBrace);
        assert_eq!(tokens[6].kind, TokenKind::LBracket);
        assert_eq!(tokens[7].kind, TokenKind::RBracket);
        assert!(errors.is_empty());
    }
}

#[cfg(test)]
mod number_tests {
    use super::*;

    #[test]
    fn test_single_digit() {
        let (tokens, errors) = tokenize("5");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind, TokenKind::Number(5));
        assert!(errors.is_empty());
    }

    #[test]
    fn test_multi_digit() {
        let (tokens, errors) = tokenize("12345");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind, TokenKind::Number(12345));
        assert!(errors.is_empty());
    }

    #[test]
    fn test_zero() {
        let (tokens, errors) = tokenize("0");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind, TokenKind::Number(0));
        assert!(errors.is_empty());
    }

    #[test]
    fn test_multiple_numbers() {
        let (tokens, errors) = tokenize("123 456 789");
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0].kind, TokenKind::Number(123));
        assert_eq!(tokens[1].kind, TokenKind::Number(456));
        assert_eq!(tokens[2].kind, TokenKind::Number(789));
        assert!(errors.is_empty());
    }

    #[test]
    fn test_large_number() {
        let (tokens, errors) = tokenize("9223372036854775807"); // i64::MAX
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind, TokenKind::Number(9223372036854775807));
        assert!(errors.is_empty());
    }
}

#[cfg(test)]
mod identifier_tests {
    use super::*;

    #[test]
    fn test_simple_identifier() {
        let (tokens, errors) = tokenize("foo");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind, TokenKind::Identifier("foo".to_string()));
        assert!(errors.is_empty());
    }

    #[test]
    fn test_identifier_with_underscore() {
        let (tokens, errors) = tokenize("my_var");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind, TokenKind::Identifier("my_var".to_string()));
        assert!(errors.is_empty());
    }

    #[test]
    fn test_identifier_with_digits() {
        let (tokens, errors) = tokenize("var123");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind, TokenKind::Identifier("var123".to_string()));
        assert!(errors.is_empty());
    }

    #[test]
    fn test_identifier_starting_with_underscore() {
        let (tokens, errors) = tokenize("_private");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind, TokenKind::Identifier("_private".to_string()));
        assert!(errors.is_empty());
    }

    #[test]
    fn test_multiple_identifiers() {
        let (tokens, errors) = tokenize("foo bar baz");
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0].kind, TokenKind::Identifier("foo".to_string()));
        assert_eq!(tokens[1].kind, TokenKind::Identifier("bar".to_string()));
        assert_eq!(tokens[2].kind, TokenKind::Identifier("baz".to_string()));
        assert!(errors.is_empty());
    }
}

#[cfg(test)]
mod keyword_tests {
    use super::*;

    #[test]
    fn test_if_keyword() {
        let (tokens, errors) = tokenize("if");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind, TokenKind::If);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_else_keyword() {
        let (tokens, errors) = tokenize("else");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind, TokenKind::Else);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_while_keyword() {
        let (tokens, errors) = tokenize("while");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind, TokenKind::While);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_return_keyword() {
        let (tokens, errors) = tokenize("return");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind, TokenKind::Return);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_void_keyword() {
        let (tokens, errors) = tokenize("void");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind, TokenKind::Void);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_int_keyword() {
        let (tokens, errors) = tokenize("int");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind, TokenKind::Int);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_all_keywords() {
        let (tokens, errors) = tokenize("if else while return void int");
        assert_eq!(tokens.len(), 7);
        assert_eq!(tokens[0].kind, TokenKind::If);
        assert_eq!(tokens[1].kind, TokenKind::Else);
        assert_eq!(tokens[2].kind, TokenKind::While);
        assert_eq!(tokens[3].kind, TokenKind::Return);
        assert_eq!(tokens[4].kind, TokenKind::Void);
        assert_eq!(tokens[5].kind, TokenKind::Int);
        assert!(errors.is_empty());
    }
}

#[cfg(test)]
mod string_tests {
    use super::*;

    #[test]
    fn test_empty_string() {
        let (tokens, errors) = tokenize("\"\"");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind, TokenKind::String("".to_string()));
        assert!(errors.is_empty());
    }

    #[test]
    fn test_simple_string() {
        let (tokens, errors) = tokenize("\"hello\"");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind, TokenKind::String("hello".to_string()));
        assert!(errors.is_empty());
    }

    #[test]
    fn test_string_with_spaces() {
        let (tokens, errors) = tokenize("\"hello world\"");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind, TokenKind::String("hello world".to_string()));
        assert!(errors.is_empty());
    }

    #[test]
    fn test_string_with_escape_n() {
        let (tokens, errors) = tokenize("\"hello\\nworld\"");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind, TokenKind::String("hello\nworld".to_string()));
        assert!(errors.is_empty());
    }

    #[test]
    fn test_string_with_escape_t() {
        let (tokens, errors) = tokenize("\"hello\\tworld\"");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind, TokenKind::String("hello\tworld".to_string()));
        assert!(errors.is_empty());
    }

    #[test]
    fn test_string_with_escape_backslash() {
        let (tokens, errors) = tokenize("\"hello\\\\world\"");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind, TokenKind::String("hello\\world".to_string()));
        assert!(errors.is_empty());
    }

    #[test]
    fn test_string_with_escape_quote() {
        let (tokens, errors) = tokenize("\"hello\\\"world\"");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind, TokenKind::String("hello\"world".to_string()));
        assert!(errors.is_empty());
    }

    #[test]
    fn test_unterminated_string() {
        let (tokens, errors) = tokenize("\"unterminated");
        assert!(tokens.len() >= 1); // 至少有EOF
        assert!(tokens.last().unwrap().is_eof());
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("Unterminated string"));
    }

    #[test]
    fn test_unknown_escape_char() {
        let (tokens, errors) = tokenize("\"hello\\xworld\"");
        assert_eq!(tokens.len(), 2);
        // 应该仍然返回字符串，但会有错误
        assert!(errors.len() > 0);
        assert!(errors[0].message.contains("Unknown escape character"));
    }
}

#[cfg(test)]
mod comparison_operator_tests {
    use super::*;

    #[test]
    fn test_equals() {
        let (tokens, errors) = tokenize("==");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind, TokenKind::Eq);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_not_equals() {
        let (tokens, errors) = tokenize("!=");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind, TokenKind::Ne);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_less_than() {
        let (tokens, errors) = tokenize("<");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind, TokenKind::Lt);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_less_than_or_equal() {
        let (tokens, errors) = tokenize("<=");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind, TokenKind::Lte);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_greater_than() {
        let (tokens, errors) = tokenize(">");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind, TokenKind::Gt);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_greater_than_or_equal() {
        let (tokens, errors) = tokenize(">=");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind, TokenKind::Gte);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_assignment_vs_equals() {
        let (tokens, errors) = tokenize("x = 5; if (x == 5)");
        assert_eq!(tokens.len(), 11);
        assert_eq!(tokens[1].kind, TokenKind::Assign); // =
        assert_eq!(tokens[7].kind, TokenKind::Eq);     // ==
        assert!(errors.is_empty());
    }
}

#[cfg(test)]
mod comment_tests {
    use super::*;

    #[test]
    fn test_line_comment() {
        let (tokens, errors) = tokenize("// This is a comment\nx");
        assert!(tokens.len() >= 2); // 至少有 ID(x) + EOF
        assert_eq!(tokens[0].kind, TokenKind::Identifier("x".to_string()));
        assert!(errors.is_empty());
    }

    #[test]
    fn test_line_comment_at_end() {
        let (tokens, errors) = tokenize("x // comment");
        assert!(tokens.len() >= 2); // 至少有 ID(x) + EOF
        assert_eq!(tokens[0].kind, TokenKind::Identifier("x".to_string()));
        assert!(errors.is_empty());
    }

    #[test]
    fn test_block_comment() {
        let (tokens, errors) = tokenize("/* comment */ x");
        assert!(tokens.len() >= 2); // 至少有 ID(x) + EOF
        assert_eq!(tokens[0].kind, TokenKind::Identifier("x".to_string()));
        assert!(errors.is_empty());
    }

    #[test]
    fn test_block_comment_multiline() {
        let (tokens, errors) = tokenize("/* line1\nline2 */ x");
        assert!(tokens.len() >= 2); // 至少有 ID(x) + EOF
        assert_eq!(tokens[0].kind, TokenKind::Identifier("x".to_string()));
        assert!(errors.is_empty());
    }

    #[test]
    fn test_unterminated_block_comment() {
        let (tokens, errors) = tokenize("/* unterminated comment");
        assert_eq!(tokens.len(), 1); // 只有EOF
        assert!(tokens[0].is_eof());
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("Unterminated block comment"));
    }

    #[test]
    fn test_nested_code_with_comments() {
        let (tokens, errors) = tokenize("int x; // initialize\n/* block */ x = 5;");
        assert!(tokens.len() >= 8); // 至少有 8 个 token（不包括 EOF）
        assert_eq!(tokens[0].kind, TokenKind::Int);
        assert_eq!(tokens[1].kind, TokenKind::Identifier("x".to_string()));
        assert_eq!(tokens[2].kind, TokenKind::Semicolon);
        assert_eq!(tokens[3].kind, TokenKind::Identifier("x".to_string()));
        assert_eq!(tokens[4].kind, TokenKind::Assign);
        assert_eq!(tokens[5].kind, TokenKind::Number(5));
        assert_eq!(tokens[6].kind, TokenKind::Semicolon);
        assert!(errors.is_empty());
    }
}

#[cfg(test)]
mod position_tests {
    use super::*;

    #[test]
    fn test_simple_position() {
        let (tokens, errors) = tokenize("x");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].span, Span::new(1, 1));
        assert!(errors.is_empty());
    }

    #[test]
    fn test_multiple_tokens_position() {
        let (tokens, errors) = tokenize("x + y");
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0].span, Span::new(1, 1)); // x
        assert_eq!(tokens[1].span, Span::new(1, 3)); // +
        assert_eq!(tokens[2].span, Span::new(1, 5)); // y
        assert!(errors.is_empty());
    }

    #[test]
    fn test_newline_position() {
        let (tokens, errors) = tokenize("x\ny");
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].span, Span::new(1, 1)); // x
        assert_eq!(tokens[1].span, Span::new(2, 1)); // y
        assert!(errors.is_empty());
    }

    #[test]
    fn test_multiline_position() {
        let (tokens, errors) = tokenize("x +\ny + z");
        assert!(tokens.len() >= 6); // 至少有 6 个 token
        assert_eq!(tokens[0].span, Span::new(1, 1)); // x
        assert_eq!(tokens[1].span, Span::new(1, 3)); // +
        assert_eq!(tokens[2].span, Span::new(2, 1)); // y
        assert_eq!(tokens[3].span, Span::new(2, 3)); // +
        assert_eq!(tokens[4].span, Span::new(2, 5)); // z
        assert!(errors.is_empty());
    }

    #[test]
    fn test_tabs_count_in_position() {
        let (tokens, errors) = tokenize("\tx");
        assert_eq!(tokens.len(), 2);
        // tab应该算作一个字符
        assert_eq!(tokens[0].span, Span::new(1, 2));
        assert!(errors.is_empty());
    }
}

#[cfg(test)]
mod error_tests {
    use super::*;

    #[test]
    fn test_invalid_character() {
        let (tokens, errors) = tokenize("@");
        assert_eq!(tokens.len(), 1); // 只有EOF
        assert!(tokens[0].is_eof());
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("Unexpected character"));
        assert!(errors[0].message.contains("@"));
    }

    #[test]
    fn test_invalid_bang_operator() {
        let (tokens, errors) = tokenize("!");
        assert_eq!(tokens.len(), 1); // 只有EOF
        assert!(tokens[0].is_eof());
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("expected '!='"));
    }

    #[test]
    fn test_multiple_errors() {
        let (tokens, errors) = tokenize("@ $ #");
        assert_eq!(tokens.len(), 1); // 只有EOF
        assert!(tokens[0].is_eof());
        assert_eq!(errors.len(), 3);
        for error in &errors {
            assert!(error.message.contains("Unexpected character"));
        }
    }

    #[test]
    fn test_error_recovery() {
        let (tokens, errors) = tokenize("@ x");
        assert_eq!(tokens.len(), 2); // ID(x) + EOF
        assert_eq!(tokens[0].kind, TokenKind::Identifier("x".to_string()));
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("@"));
    }
}

#[cfg(test)]
mod complex_tests {
    use super::*;

    #[test]
    fn test_variable_declaration() {
        let (tokens, errors) = tokenize("int x;");
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0].kind, TokenKind::Int);
        assert_eq!(tokens[1].kind, TokenKind::Identifier("x".to_string()));
        assert_eq!(tokens[2].kind, TokenKind::Semicolon);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_variable_assignment() {
        let (tokens, errors) = tokenize("x = 42;");
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0].kind, TokenKind::Identifier("x".to_string()));
        assert_eq!(tokens[1].kind, TokenKind::Assign);
        assert_eq!(tokens[2].kind, TokenKind::Number(42));
        assert_eq!(tokens[3].kind, TokenKind::Semicolon);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_if_statement() {
        let (tokens, errors) = tokenize("if (x < 5) { return x; }");
        assert!(tokens.len() >= 12); // 至少有 12 个 token
        assert_eq!(tokens[0].kind, TokenKind::If);
        assert_eq!(tokens[1].kind, TokenKind::LParen);
        assert_eq!(tokens[2].kind, TokenKind::Identifier("x".to_string()));
        assert_eq!(tokens[3].kind, TokenKind::Lt);
        assert_eq!(tokens[4].kind, TokenKind::Number(5));
        assert_eq!(tokens[5].kind, TokenKind::RParen);
        assert_eq!(tokens[6].kind, TokenKind::LBrace);
        assert_eq!(tokens[7].kind, TokenKind::Return);
        assert_eq!(tokens[8].kind, TokenKind::Identifier("x".to_string()));
        assert_eq!(tokens[9].kind, TokenKind::Semicolon);
        assert_eq!(tokens[10].kind, TokenKind::RBrace);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_while_loop() {
        let (tokens, errors) = tokenize("while (i < 10) { i = i + 1; }");
        assert!(tokens.len() >= 15); // 至少有 15 个 token
        assert_eq!(tokens[0].kind, TokenKind::While);
        assert_eq!(tokens[1].kind, TokenKind::LParen);
        assert_eq!(tokens[2].kind, TokenKind::Identifier("i".to_string()));
        assert_eq!(tokens[3].kind, TokenKind::Lt);
        assert_eq!(tokens[4].kind, TokenKind::Number(10));
        assert_eq!(tokens[5].kind, TokenKind::RParen);
        assert_eq!(tokens[6].kind, TokenKind::LBrace);
        assert_eq!(tokens[7].kind, TokenKind::Identifier("i".to_string()));
        assert_eq!(tokens[8].kind, TokenKind::Assign);
        assert_eq!(tokens[9].kind, TokenKind::Identifier("i".to_string()));
        assert_eq!(tokens[10].kind, TokenKind::Plus);
        assert_eq!(tokens[11].kind, TokenKind::Number(1));
        assert_eq!(tokens[12].kind, TokenKind::Semicolon);
        assert_eq!(tokens[13].kind, TokenKind::RBrace);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_function_call_like() {
        let (tokens, errors) = tokenize("foo(x, y, z);");
        assert_eq!(tokens.len(), 10);
        assert_eq!(tokens[0].kind, TokenKind::Identifier("foo".to_string()));
        assert_eq!(tokens[1].kind, TokenKind::LParen);
        assert_eq!(tokens[2].kind, TokenKind::Identifier("x".to_string()));
        assert_eq!(tokens[3].kind, TokenKind::Comma);
        assert_eq!(tokens[4].kind, TokenKind::Identifier("y".to_string()));
        assert_eq!(tokens[5].kind, TokenKind::Comma);
        assert_eq!(tokens[6].kind, TokenKind::Identifier("z".to_string()));
        assert_eq!(tokens[7].kind, TokenKind::RParen);
        assert_eq!(tokens[8].kind, TokenKind::Semicolon);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_array_access_like() {
        let (tokens, errors) = tokenize("arr[0]");
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0].kind, TokenKind::Identifier("arr".to_string()));
        assert_eq!(tokens[1].kind, TokenKind::LBracket);
        assert_eq!(tokens[2].kind, TokenKind::Number(0));
        assert_eq!(tokens[3].kind, TokenKind::RBracket);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_complex_expression() {
        let (tokens, errors) = tokenize("result = (a + b) * (c - d) / 2;");
        assert!(tokens.len() >= 16); // 至少有 16 个 token
        assert_eq!(tokens[0].kind, TokenKind::Identifier("result".to_string()));
        assert_eq!(tokens[1].kind, TokenKind::Assign);
        assert_eq!(tokens[2].kind, TokenKind::LParen);
        assert_eq!(tokens[3].kind, TokenKind::Identifier("a".to_string()));
        assert_eq!(tokens[4].kind, TokenKind::Plus);
        assert_eq!(tokens[5].kind, TokenKind::Identifier("b".to_string()));
        assert_eq!(tokens[6].kind, TokenKind::RParen);
        assert_eq!(tokens[7].kind, TokenKind::Star);
        assert_eq!(tokens[8].kind, TokenKind::LParen);
        assert_eq!(tokens[9].kind, TokenKind::Identifier("c".to_string()));
        assert_eq!(tokens[10].kind, TokenKind::Minus);
        assert_eq!(tokens[11].kind, TokenKind::Identifier("d".to_string()));
        assert_eq!(tokens[12].kind, TokenKind::RParen);
        assert_eq!(tokens[13].kind, TokenKind::Slash);
        assert_eq!(tokens[14].kind, TokenKind::Number(2));
        assert_eq!(tokens[15].kind, TokenKind::Semicolon);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_string_in_code() {
        let (tokens, errors) = tokenize("print(\"Hello, World!\");");
        assert_eq!(tokens.len(), 6);
        assert_eq!(tokens[0].kind, TokenKind::Identifier("print".to_string()));
        assert_eq!(tokens[1].kind, TokenKind::LParen);
        assert_eq!(tokens[2].kind, TokenKind::String("Hello, World!".to_string()));
        assert_eq!(tokens[3].kind, TokenKind::RParen);
        assert_eq!(tokens[4].kind, TokenKind::Semicolon);
        assert!(errors.is_empty());
    }
}

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn test_tokens_without_spaces() {
        let (tokens, errors) = tokenize("x+y");
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0].kind, TokenKind::Identifier("x".to_string()));
        assert_eq!(tokens[1].kind, TokenKind::Plus);
        assert_eq!(tokens[2].kind, TokenKind::Identifier("y".to_string()));
        assert!(errors.is_empty());
    }

    #[test]
    fn test_mixed_whitespace() {
        let (tokens, errors) = tokenize("x\n\t+\ry");
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0].kind, TokenKind::Identifier("x".to_string()));
        assert_eq!(tokens[1].kind, TokenKind::Plus);
        assert_eq!(tokens[2].kind, TokenKind::Identifier("y".to_string()));
        assert!(errors.is_empty());
    }

    #[test]
    fn test_keyword_as_identifier_prefix() {
        let (tokens, errors) = tokenize("ifx");
        assert_eq!(tokens.len(), 2);
        // "ifx" 应该被识别为标识符，不是关键字
        assert_eq!(tokens[0].kind, TokenKind::Identifier("ifx".to_string()));
        assert!(errors.is_empty());
    }

    #[test]
    fn test_consecutive_operators() {
        let (tokens, errors) = tokenize("++--");
        assert_eq!(tokens.len(), 5); // 4个运算符 + EOF
        assert_eq!(tokens[0].kind, TokenKind::Plus);
        assert_eq!(tokens[1].kind, TokenKind::Plus);
        assert_eq!(tokens[2].kind, TokenKind::Minus);
        assert_eq!(tokens[3].kind, TokenKind::Minus);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_slash_vs_division() {
        let (tokens, errors) = tokenize("a / b");
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0].kind, TokenKind::Identifier("a".to_string()));
        assert_eq!(tokens[1].kind, TokenKind::Slash);
        assert_eq!(tokens[2].kind, TokenKind::Identifier("b".to_string()));
        assert!(errors.is_empty());
    }

    #[test]
    fn test_very_long_identifier() {
        let long_id = "a".repeat(1000);
        let (tokens, errors) = tokenize(&long_id);
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind, TokenKind::Identifier(long_id));
        assert!(errors.is_empty());
    }

    #[test]
    fn test_very_long_number() {
        let long_num = "9".repeat(100);
        let (tokens, errors) = tokenize(&long_num);
        assert_eq!(tokens.len(), 2);
        // 应该解析为0，因为超出i64范围
        assert_eq!(tokens[0].kind, TokenKind::Number(0));
        assert!(errors.is_empty());
    }
}
