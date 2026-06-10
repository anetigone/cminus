// 语法分析错误测试 -- 展示各种语法错误的检测
//
// 本示例展示 parser 在遇到各种语法错误时产生的 ParseError。
// 不打印 token 流，只展示源代码和对应的语法错误。

use c_minus::parser::Parser;

fn parse_and_show(name: &str, source: &str) {
    println!("==========================================");
    println!("测试: {}", name);
    println!("==========================================");
    println!("源代码:\n{}", source);
    println!("------------------------------------------");

    let mut lexer = c_minus::lexer::Lexer::new(source.to_string());
    let tokens = lexer.tokenize();

    if !lexer.errors.is_empty() {
        println!("✗ 词法错误 (跳过语法分析):");
        for err in &lexer.errors {
            println!("  {}", err);
        }
        println!();
        return;
    }

    let mut parser = Parser::new(tokens);
    match parser.parse_program() {
        Ok(ast) => {
            println!("✓ 语法分析成功 (本应报错!)");
            println!("AST:\n{}", c_minus::parser::print::print_tree(&ast));
        }
        Err(err) => {
            println!("✗ 语法错误:");
            println!("  {}", err);
        }
    }
    println!();
}

fn main() {
    println!("╔══════════════════════════════════════════╗");
    println!("║     C-Minus 语法分析错误测试             ║");
    println!("╚══════════════════════════════════════════╝\n");

    // 错误 1: 缺少分号
    parse_and_show(
        "缺少分号",
        "int x",
    );

    // 错误 2: 缺少类型说明符
    parse_and_show(
        "缺少类型说明符",
        "x;",
    );

    // 错误 3: 函数声明缺少右括号
    parse_and_show(
        "函数声明缺少右小括号",
        "void main( {}",
    );

    // 错误 4: 函数体缺少右花括号
    parse_and_show(
        "函数体缺少右花括号",
        "void main() {",
    );

    // 错误 5: 声明中使用了数字代替标识符
    parse_and_show(
        "声明中使用数字代替标识符",
        "int 123;",
    );

    // 错误 6: 数组声明缺少右方括号
    parse_and_show(
        "数组声明缺少右方括号",
        "int arr[10;",
    );

    // 错误 7: 数组声明缺少大小
    parse_and_show(
        "数组声明缺少大小",
        "int arr[];",
    );

    // 错误 8: if 语句缺少条件中的右括号
    parse_and_show(
        "if 语句缺少右小括号",
        "void main() { if (x return; }",
    );

    // 错误 9: while 语句缺少左括号
    parse_and_show(
        "while 语句缺少左小括号",
        "void main() { while x > 0) return; }",
    );

    // 错误 10: return 语句缺少分号
    parse_and_show(
        "return 语句缺少分号",
        "int main() { return 42 }",
    );

    // 错误 11: 赋值表达式缺少分号
    parse_and_show(
        "赋值表达式缺少分号",
        "void main() { x = 1 }",
    );

    // 错误 12: 表达式中出现非法 token
    parse_and_show(
        "表达式中缺少右小括号",
        "void main() { return (x + 1; }",
    );

    // 错误 13: 声明位置出现非法 token
    parse_and_show(
        "声明位置出现非法 token (缺少 ';' 或 '(' 或 '[')",
        "int x int y;",
    );

    // 错误 14: 复合语句缺少右花括号 (嵌套)
    parse_and_show(
        "嵌套复合语句缺少右花括号",
        "void main() { { x; }",
    );

    // 错误 15: 函数调用缺少右括号
    parse_and_show(
        "函数调用缺少右小括号",
        "void main() { output(x, y; }",
    );

    // ===== 作为对比，最后展示一段正确的代码 =====
    println!("==========================================");
    println!("对比: 正确的代码 (无语法错误)");
    println!("==========================================");

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

    println!("源代码:{}", source);
    println!("------------------------------------------");

    let mut lexer = c_minus::lexer::Lexer::new(source.to_string());
    let tokens = lexer.tokenize();

    let mut parser = Parser::new(tokens);
    match parser.parse_program() {
        Ok(ast) => {
            println!("✓ 语法分析成功");
            println!("AST:\n{}", c_minus::parser::print::print_tree(&ast));
        }
        Err(err) => {
            println!("✗ 语法错误: {}", err);
        }
    }
}
