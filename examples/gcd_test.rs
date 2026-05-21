// gcd 函数词法分析测试

fn main() {
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

    println!("源代码:\n{}", source);
    println!("══════════════════════════════════════════");
    println!("Token 输出:\n");

    let mut lexer = c_minus::lexer::Lexer::new(source.to_string());
    let tokens = lexer.tokenize();

    for token in &tokens {
        println!("  [{:>3}:{:<3}] {}",
            token.span.row, token.span.col, token.kind);
    }

    println!("\n══════════════════════════════════════════");

    if lexer.errors.is_empty() {
        println!("✓ 没有词法错误");
    } else {
        println!("✗ 发现错误:");
        for err in &lexer.errors {
            println!("  {}", err);
        }
    }

    println!("\n总 Token 数: {} (包括 EOF)", tokens.len());
}
