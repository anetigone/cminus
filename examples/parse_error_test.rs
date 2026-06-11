// 语法分析错误测试 -- 展示恐慌恢复模式下的一次性多错误检测
//
// 本示例展示 parser 在恐慌恢复模式下，遇到多个语法错误时
// 能够一次性报告所有错误，而不是遇到第一个错误就停止。
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
    let _ast = parser.parse_program();

    if parser.errors.is_empty() {
        println!("✓ 语法分析成功 (本应报错!)");
        println!("AST:\n{}", c_minus::parser::print::print_tree(&_ast));
    } else {
        println!("✗ 语法错误 (共 {} 个):", parser.errors.len());
        for (i, err) in parser.errors.iter().enumerate() {
            println!("  {}. {}", i + 1, err);
        }
    }
    println!();
}

fn main() {
    println!("╔══════════════════════════════════════════╗");
    println!("║  C-Minus 语法分析错误测试                ║");
    println!("║  (恐慌恢复模式 - 一次性报告所有错误)      ║");
    println!("╚══════════════════════════════════════════╝\n");

    // 测试 1: 多个声明各缺少分号
    parse_and_show(
        "多个声明缺少分号",
        r#"
int x
int y
int z
"#,
    );

    // 测试 2: 多种错误混合在一个函数中
    parse_and_show(
        "函数内多种错误混合",
        r#"
void main() {
    int a
    x = 1 + ;
    if (x > 0 return;
    while x > 0) { a = 1; }
}
"#,
    );

    // 测试 3: 多个函数各有不同错误
    parse_and_show(
        "多个函数各有不同错误",
        r#"
int add(int a, int b {
    return a + b
}

void main( {
    int arr[10;
    int x
    output(x, y;
}
"#,
    );

    // 测试 4: 数组声明缺少大小 + 缺少标识符
    parse_and_show(
        "数组声明系列错误",
        r#"
int arr1[];
int 123;
int arr2[10;
"#,
    );

    // 测试 5: 嵌套语句中多层错误
    parse_and_show(
        "嵌套语句多层错误",
        r#"
void main() {
    if (x > 0) {
        int a
        if (y > 0) {
            return 1
        }
    }
}
"#,
    );

    // 测试 6: 全局声明级别的多个错误
    parse_and_show(
        "全局声明多个错误",
        r#"
int x int y;
123;
int arr[;
void main() {}
"#,
    );

    // ===== 作为对比，最后展示一段正确的代码 =====
    println!("==========================================");
    println!("对比: 正确的代码");
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
    let ast = parser.parse_program();

    if parser.errors.is_empty() {
        println!("✓ 语法分析成功");
        println!("AST:\n{}", c_minus::parser::print::print_tree(&ast));
    } else {
        println!("✗ 语法错误:");
        for err in &parser.errors {
            println!("  {}", err);
        }
    }
}
