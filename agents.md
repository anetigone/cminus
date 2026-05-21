1. Project Overview
This is a C-Minus compiler written in Rust. C-Minus is a subset of the C language (based on Kenneth Louden's "Compiler Construction: Principles and Practice", Appendix A). The project implements a multi-pass compiler pipeline:
- Phase 1: Lexer (tokenizer) -- mostly complete
- Phase 2: Parser (recursive descent) -- partially implemented, several methods are stubs
- Phase 3: Semantic Analyzer -- not yet implemented
- Phase 4: Code Generator -- not yet implemented
The Rust edition is 2024. The only dependency is displaydoc = "0.2.5" (used for deriving Display on the TokenKind and BinaryOp enums via doc comments).
2. Directory Structure Overview
D:\vscode\rust\c_minus\
├── Cargo.toml                  # Package manifest (name: "c_minus", edition 2024)
├── Cargo.lock
├── .gitignore                  # Ignores target/, .vscode/
├── lexer.md                    # Design doc: lexer specification (847 lines)
├── parser.md                   # Design doc: parser specification (605 lines)
├── .vscode/                    # IDE configuration
├── examples/
│   └── gcd_test.rs             # Example: tokenizing a GCD program
├── target/                     # Build artifacts
└── src/
    ├── lib.rs                  # Library root: `pub mod lexer; pub mod parser;`
    ├── main.rs                 # Binary entry (just prints "Hello, world!")
    ├── lexer/
    │   ├── mod.rs              # Lexer struct and core scanning logic (315 lines)
    │   ├── token.rs            # TokenKind enum, Token struct, Span struct (137 lines)
    │   ├── error.rs            # LexError struct (25 lines)
    │   └── tests.rs            # Comprehensive test suite (726 lines, 40+ tests)
    └── parser/
        ├── mod.rs              # Parser struct (recursive descent) (411 lines)
        ├── ast.rs              # AST node definitions (132 lines)
        └── error.rs            # ParseError struct (24 lines)
Key design documents:
- lexer.md -- The original lexer design specification. Contains the token type definitions, DFA state transition diagram, error handling strategy, and original reference implementation code (the actual implementation in src/ has since diverged/evolved from this spec).
- parser.md -- The parser design specification including BNF grammar, EBNF rewrite, AST node definitions, and a reference implementation. The actual parser in src/parser/ partially follows this.
3. Contents of the examples Directory
There is a single example file:
D:\vscode\rust\c_minus\examples\gcd_test.rs (42 lines)
This example demonstrates lexing a complete C-Minus program consisting of two functions:
let source = r#"
int gcd(int u, int v){
    if(v==0) return u;
    else return gcd(v, u-u/v*v);
}
void main(void){
    int x;int y;
    x=input();
    y=input();
    output(gcd(x,y));
}
"#;
let mut lexer = c_minus::lexer::Lexer::new(source.to_string());
let tokens = lexer.tokenize();
// ... prints each token with [row:col] and token kind
// ... prints any errors found
It creates a lexer, tokenizes the source, prints every token with its position, and reports any lexical errors. It uses the library crate (c_minus::lexer).
4. Key Lexer Implementation Details
4.1 Core Types
Span (src/lexer/token.rs, line 6):
pub struct Span {
    pub row: usize,  // 1-based line number
    pub col: usize,  // 1-based column number
}
Token (src/lexer/token.rs, line 116):
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}
TokenKind (src/lexer/token.rs, lines 24-98) -- an enum with 30 variants:
- Keywords: If, Else, While, Return, Void, Int
- Identifiers & literals: Identifier(String), Number(i64), String(String)
- Operators: Plus, Minus, Star, Slash, Eq, Ne, Assign, Lt, Gt, Le, Ge
- Delimiters: Semicolon, Comma, LParen, RParen, LBrace, RBrace, LBracket, RBracket
- Special: EOF
Lexer (src/lexer/mod.rs, line 10):
pub struct Lexer {
    source: Vec<char>,       // source code as character array
    pos: usize,              // current position index
    line: usize,             // current line (1-based)
    column: usize,           // current column (1-based)
    pub errors: Vec<LexError>, // collected errors
}
4.2 Lexer Features Beyond the Design Spec
The actual implementation extends the original design doc (lexer.md) in several ways:
- String literals: Added support for "..." strings with escape sequences (\n, \t, \\, \")
- Single-line comments: Added // style comments (not in original C-Minus spec)
- Underscores in identifiers: _ is allowed as the first character in identifiers, and identifiers can contain alphanumeric characters (not just alphabetic as in the original spec)
- Fully tested: 40+ test cases in tests.rs
5. Lexical Error Types and How They're Reported
5.1 The LexError Struct
File: D:\vscode\rust\c_minus\src\lexer\error.rs
#[derive(Debug, Clone)]
pub struct LexError {
    pub message: String,
    pub span: Span,
}
impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LexError at {}: {}", self.span, self.message)
    }
}
impl Error for LexError {}
The output format is: "LexError at (row, col): <message>"
5.2 Four Specific Error Cases Detected
All errors are pushed into self.errors (a Vec<LexError> on the Lexer struct) and the lexer continues scanning (error recovery). The four error types are:
(a) Unterminated block comment -- D:\vscode\rust\c_minus\src\lexer\mod.rs, lines 93-102
pub fn skip_block_comment(&mut self, start_span: Span) {
    while let Some(ch) = self.advance() {
        if ch == '*' && self.peek() == Some('/') {
            self.advance();
            return;
        }
    }
    // EOF reached without closing */
    self.errors.push(LexError::new(
        "Unterminated block comment".to_string(),
        start_span
    ));
}
Detected when: /* is encountered but */ is never found before EOF. The span points to where the comment started.
(b) Unterminated string -- D:\vscode\rust\c_minus\src\lexer\mod.rs, lines 145-165
pub fn scan_string(&mut self) -> Option<Token> {
    let start_span = self.current_span();
    // ...
    while let Some(ch) = self.advance() {
        match ch {
            '"' => return Some(...),  // found closing quote
            '\\' => { /* handle escape */ }
            _ => string.push(ch),
        }
    }
    // EOF reached without closing "
    self.errors.push(LexError::new(
        "Unterminated string".to_string(),
        start_span
    ));
    None  // returns None, caller skips to next token
}
Detected when: " is encountered but the closing " is never found before EOF. Returns None (no token produced), and the caller (next_token()) recursively tries for the next token.
(c) Unknown escape character -- D:\vscode\rust\c_minus\src\lexer\mod.rs, lines 168-183
fn scan_escape_char(&mut self, start_span: &Span) -> Option<char> {
    match self.advance() {
        Some('n') => Some('\n'),
        Some('t') => Some('\t'),
        Some('\\') => Some('\\'),
        Some('"') => Some('"'),
        Some(ch) => {
            self.errors.push(LexError::new(
                format!("Unknown escape character: \\{}", ch),
                start_span.clone()
            ));
            Some(ch)  // still returns the raw character, continues parsing
        }
        None => {
            self.errors.push(LexError::new(
                "Unterminated string".to_string(),
                start_span.clone()
            ));
            None
        }
    }
}
Detected when: A backslash \ is followed by a character that is not one of n, t, \, ". The lexer continues parsing the string (returns the raw character).
(d) Unexpected character -- D:\vscode\rust\c_minus\src\lexer\mod.rs, lines 270-298
There are actually two variants:
! without following = (lines 270-283):
'!' => {
    if self.peek() == Some('=') {
        self.advance();
        Some(Token::new(TokenKind::Ne, span))
    } else {
        self.errors.push(LexError::new(
            format!("Unexpected character: '{}', expected '!='", ch),
            span.clone()
        ));
        self.next_token()  // skip and continue
    }
}
Any other unrecognized character (lines 292-298, the wildcard _ arm):
_ => {
    self.errors.push(LexError::new(
        format!("Unexpected character: '{}'", ch),
        span
    ));
    self.next_token()  // skip and continue
}
Detected when: A character like @, $, #, etc., is encountered that doesn't match any valid token starting character. The span points to the offending character.
5.3 Error Recovery Strategy
The error recovery is consistent across all error cases:
1. The error is pushed into self.errors (accumulated, not fatal)
2. The lexer calls self.next_token() recursively (or returns None for unterminated strings) to skip past the problematic input and continue scanning
3. This means the lexer always produces as many valid tokens as possible, even in the presence of errors
5.4 Error Testing
The test module error_tests in D:\vscode\rust\c_minus\src\lexer\tests.rs (lines 483-525) covers:
- test_invalid_character -- @ produces one error with "Unexpected character"
- test_invalid_bang_operator -- standalone ! produces error with "expected '!='"
- test_multiple_errors -- @ $ # produces 3 errors
- test_error_recovery -- @ x produces 1 error but still yields the x token
5.5 How Errors Are Accessed by Users
The Lexer struct exposes pub errors: Vec<LexError> directly. After calling tokenize(), callers check lexer.errors.is_empty() and iterate over errors. This pattern is visible in both examples/gcd_test.rs (lines 32-39):
if lexer.errors.is_empty() {
    println!("✓ 没有词法错误");
} else {
    println!("✗ 发现错误:");
    for err in &lexer.errors {
        println!("  {}", err);
    }
}