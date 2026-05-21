// 词法分析错误测试 -- 展示各种词法错误的检测与恢复
//
// 由于未闭合的字符串和块注释会"贪婪地"消耗到文件末尾，
// 它们无法在同一个 lexer 运行中共存，因此本示例使用两个独立的 lexer 运行。

fn main() {
    // ===== 第一组: 非法字符, 单独的!, 未知转义, 未闭合字符串 =====
    {
        let source = r#"
/* Error 1: Unexpected character @ */
int x = 10 @ 5;

/* Error 2: Standalone ! (expecting !=) */
int check(int a) {
    if (a ! 0) return a;
}

/* Error 3: Unknown escape character */
/* C-Minus only supports \n \t \\ \" */
char* path = "C:\path";

/* Error 4: Unterminated string */
/* The string opens with " but never closes -- consumes to EOF */
char* bad = "this string has no closing quote.
"#;

        println!("==========================================");
        println!("Group 1: Invalid char / Standing ! / Unknown escape / Unterminated string");
        println!("==========================================\n");
        println!("Source:\n{}", source);

        let mut lexer = c_minus::lexer::Lexer::new(source.to_string());
        let tokens = lexer.tokenize();

        println!("Tokens:\n");
        for token in &tokens {
            println!(
                "  [{:>3}:{:<3}] {}",
                token.span.row, token.span.col, token.kind
            );
        }

        println!("\n------------------------------------------");

        if lexer.errors.is_empty() {
            println!("No errors found");
        } else {
            println!("{} error(s) found:", lexer.errors.len());
            for err in &lexer.errors {
                println!("  {}", err);
            }
        }

        println!("\nTotal tokens: {} (including EOF)\n", tokens.len());
    }

    // ===== 第二组: 未闭合的块注释 =====
    {
        let source = r#"
/* Error 5: Unterminated block comment */
int valid_code = 42;

/* This block comment has no closing -- it swallows everything to EOF!
int swallowed = 100;
"#;

        println!("==========================================");
        println!("Group 2: Unterminated block comment");
        println!("==========================================\n");
        println!("Source:\n{}", source);

        let mut lexer = c_minus::lexer::Lexer::new(source.to_string());
        let tokens = lexer.tokenize();

        println!("Tokens:\n");
        for token in &tokens {
            println!(
                "  [{:>3}:{:<3}] {}",
                token.span.row, token.span.col, token.kind
            );
        }

        println!("\n------------------------------------------");

        if lexer.errors.is_empty() {
            println!("No errors found");
        } else {
            println!("{} error(s) found:", lexer.errors.len());
            for err in &lexer.errors {
                println!("  {}", err);
            }
        }

        println!("\nTotal tokens: {} (including EOF)\n", tokens.len());
    }
}
