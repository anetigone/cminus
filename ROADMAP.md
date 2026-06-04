# C-Minus 编译器开发路线图

> 当前状态：词法分析 ✅ | 语法分析 ✅ | 语义分析 ❌ | 中间代码生成 ❌ | 代码优化 ❌ | 目标代码生成 ❌

## 项目现状

已完成：
- **词法分析器** (`src/lexer/`)：支持关键字、标识符、整数/字符串字面量、运算符、分隔符、注释
- **语法分析器** (`src/parser/`)：递归下降解析，生成完整 AST
- **AST 定义** (`src/parser/ast.rs`)：Program / Declaration / Stmt / Expression 完整定义
- **AST 打印** (`src/parser/print.rs`)：树形可视化输出
- **源文件读取** (`src/source.rs`)：`.cm` 文件读取

C-Minus 语言特性（已解析）：
- 类型：`int`、`void`
- 变量声明（含数组）、函数声明与定义
- 语句：if/else、while、return、复合语句、表达式语句
- 表达式：赋值、二元算术/关系运算、函数调用、数组下标、字面量
- 注释：单行 `//`、多行 `/* */`

---

## 第一步：符号表 (Symbol Table)

### 目标

管理程序中所有标识符（变量、函数、参数）的名字、类型、作用域信息，为后续语义分析提供查询基础。

### 为什么先做符号表

符号表是语义分析的基石。类型检查需要查询「这个变量是什么类型」，作用域检查需要知道「这个变量在当前作用域是否可见」。没有符号表，后续所有分析都无法进行。

### 设计方案

```
src/
├── semantic/
│   ├── mod.rs            # 模块入口
│   ├── symbol.rs         # 符号定义 (SymbolKind, Symbol)
│   ├── symbol_table.rs   # 符号表 (SymbolTable, 支持作用域嵌套)
│   └── analyzer.rs       # 语义分析器 (后续步骤实现)
```

#### 1. 符号类型 (`symbol.rs`)

```rust
/// 符号的种类
pub enum SymbolKind {
    Var(VarInfo),       // 变量（含数组）
    Func(FuncInfo),     // 函数
    Param(ParamInfo),   // 函数参数
}

pub struct VarInfo {
    pub ty: TypeSpec,           // 变量类型
    pub is_array: bool,         // 是否为数组
    pub array_size: Option<u32>,// 数组大小
    pub offset: Option<i32>,    // 栈帧偏移（代码生成阶段使用）
}

pub struct FuncInfo {
    pub return_type: TypeSpec,
    pub params: Vec<ParamInfo>,
    pub param_count: usize,
}

pub struct ParamInfo {
    pub ty: TypeSpec,
    pub is_array: bool,
}

/// 一个符号条目
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub span: Span,             // 声明位置（用于报错）
}
```

#### 2. 符号表 (`symbol_table.rs`)

```rust
/// 单个作用域层的符号映射
pub struct Scope {
    symbols: HashMap<String, Symbol>,
    parent: Option<Box<Scope>>,
    scope_kind: ScopeKind,      // Global / Function / Block
}

pub enum ScopeKind {
    Global,
    Function { name: String },
    Block,
}

/// 符号表：支持嵌套作用域的栈式结构
pub struct SymbolTable {
    current: Scope,             // 当前作用域
    depth: usize,               // 当前嵌套深度
}
```

核心操作：
- `insert(name, symbol)` — 向当前作用域插入符号，检测重定义
- `lookup(name)` — 从当前作用域向外层查找符号
- `lookup_current(name)` — 仅在当前作用域查找（用于检测重定义）
- `enter_scope(kind)` — 进入新作用域
- `leave_scope()` — 离开当前作用域

#### 3. 遍历 AST 构建符号表

对 AST 做**两遍遍历**（two-pass）：

| 遍次 | 做什么 | 为什么 |
|------|--------|--------|
| 第 1 遍 | 收集所有全局变量和函数签名到全局符号表 | 允许函数前向引用（`gcd` 在 `main` 之后定义也能被调用） |
| 第 2 遍 | 进入函数体，处理局部变量、参数、作用域嵌套 | 函数内部的作用域和类型检查 |

### 要检测的错误

- 同一作用域内重复定义：`int x; int x;`
- 使用未声明的标识符

### 测试用例

```rust
#[test]
fn test_global_var_insert() { /* 全局变量插入和查询 */ }
#[test]
fn test_func_insert() { /* 函数签名插入和查询 */ }
#[test]
fn test_scope_nesting() { /* 嵌套作用域：内层可访问外层，外层看不到内层 */ }
#[test]
fn test_redefinition_error() { /* 同一作用域重定义报错 */ }
#[test]
fn test_undeclared_error() { /* 使用未声明变量报错 */ }
#[test]
fn test_forward_reference() { /* 函数前向引用 */ }
```

---

## 第二步：类型检查 (Type Checking)

### 目标

为 AST 中每个表达式节点确定其类型，并对类型不兼容的操作报错。

### 前置条件

符号表已完成。

### 设计方案

在 `src/semantic/analyzer.rs` 中实现 `SemanticAnalyzer`：

```rust
pub struct SemanticAnalyzer {
    symbol_table: SymbolTable,
    errors: Vec<SemanticError>,
    current_function: Option<String>,  // 当前所在的函数（用于 return 检查）
}
```

#### 类型系统

C-Minus 的类型很简单，只有 `int`、`void`，加上数组类型的退化：

```rust
/// 表达式的类型（比 AST 的 TypeSpec 更丰富）
pub enum ExprType {
    Int,                // int 值
    Void,               // void（仅用于函数返回）
    ArrayInt,           // int[]（数组变量本身，不可参与算术运算）
    Error,              // 类型错误（用于错误恢复，避免级联报错）
}
```

#### 类型检查规则

| 表达式 | 类型 | 约束 |
|--------|------|------|
| `Number(n)` | `Int` | — |
| `String(s)` | `String`（C-Minus 中可作为 `output` 参数） | — |
| `LVar(name)` 普通 | 查符号表 → `Int` | 必须已声明，不能是数组（无下标） |
| `LVar(name[expr])` 数组下标 | 查符号表 → `Int` | `name` 必须是数组，`expr` 必须是 `Int` |
| `BinOp(op, l, r)` 算术 | `Int` | `l`, `r` 必须是 `Int` |
| `BinOp(op, l, r)` 关系 | `Int` | `l`, `r` 必须是 `Int`（结果视为 0/1） |
| `Call(name, args)` | 查符号表函数返回类型 | 函数必须已声明，参数个数/类型匹配 |
| `Assign(lvar, expr)` | `Int` | `lvar` 必须是左值，`expr` 必须是 `Int` |

#### 语句级检查

| 语句 | 检查内容 |
|------|----------|
| `if (cond)` | `cond` 必须是 `Int` |
| `while (cond)` | `cond` 必须是 `Int` |
| `return expr;` | `expr` 类型必须与当前函数返回类型匹配 |
| `return;` | 当前函数必须返回 `void` |
| `expr;` | 表达式合法 |

#### 全局级检查

- `main` 函数必须存在
- `main` 函数签名应为 `void main(void)`
- `void` 类型变量不允许（`void x;` 非法）
- 函数参数不能是 `void`（`void` 只能单独出现在参数列表表示无参数）

### 要检测的错误

- 表达式类型不匹配：`int x = "hello";`、数组参与算术运算
- 对非数组变量使用下标：`x[0]` 但 `x` 是普通变量
- 对数组变量直接运算：`int a[10]; x = a + 1;`
- 函数调用参数个数不匹配
- 函数调用参数类型不匹配
- 对非函数标识符使用调用语法：`int x; x();`
- 在 `void` 函数中 `return expr;`
- 在非 `void` 函数中只有 `return;`
- 使用 `void` 表达式作为值：`int x = voidFunc();`
- `main` 函数缺失或签名错误

### 测试用例

```rust
#[test]
fn test_type_check_arithmetic() { /* 算术运算类型正确 */ }
#[test]
fn test_type_check_array_index() { /* 数组下标必须是 int */ }
#[test]
fn test_type_check_call_args() { /* 函数调用参数匹配 */ }
#[test]
fn test_return_type_mismatch() { /* 返回类型不匹配报错 */ }
#[test]
fn test_main_function_check() { /* main 函数存在性检查 */ }
#[test]
fn test_void_variable_error() { /* void 变量报错 */ }
```

---

## 第三步：语义分析的收尾与 AST 装饰

### 目标

将类型信息标注回 AST（或生成带类型的 AST / Typed AST），完成所有语义错误的检测。

### 设计方案

有两种实现策略：

**策略 A：给 AST 节点加上可选的 type 字段**

```rust
pub struct TypedExpression {
    pub kind: Expression,  // 原始表达式
    pub ty: ExprType,      // 推导出的类型
    pub span: Span,        // 位置信息
}
```

**策略 B：创建独立的注解表**

```rust
/// 用 HashMap 记录每个表达式节点的类型
pub struct TypeMap {
    types: HashMap<NodeId, ExprType>,
}
```

给每个 AST 节点加上 `id: NodeId` 字段即可关联。

> **建议采用策略 A**，因为 C-Minus 的 AST 不大，直接在节点上标注类型更简单直观。

### 其他语义检查

- **常量表达式检查**：数组声明大小 `int a[expr]` 中 `expr` 是否为常量（C-Minus 中应为数字字面量）
- **不可达代码警告**（可选）：`return` 之后的语句
- **函数是否所有路径都有返回值**（可选，C-Minus 可以放宽这个要求）

---

## 第四步：中间代码生成 (IR Generation)

### 目标

将类型检查后的 AST 翻译为三地址码 (Three-Address Code, TAC) 或类似中间表示。

### 为什么需要中间代码

中间代码是编译器的「枢纽」：
- 把树形 AST 展平为线性指令序列，便于优化和代码生成
- 与具体目标机器无关，方便移植
- 许多优化算法在三地址码上比在 AST 上更容易实现

### 三地址码设计

```rust
/// 三地址码指令
pub enum Tac {
    // 赋值
    Assign { dst: Operand, src: Operand },                      // dst = src
    BinOp { dst: Operand, op: BinaryOp, left: Operand, right: Operand }, // dst = left op right
    UnaryOp { dst: Operand, op: UnaryOp, src: Operand },       // dst = op src

    // 控制流
    Label(String),                                              // label:
    Jump(String),                                               // goto label
    CondJump { cond: Operand, op: RelOp, label_true: String, label_false: String },

    // 函数
    Call { dst: Option<Operand>, func: String, args: Vec<Operand> },
    Return(Option<Operand>),
    Param(Operand),                                             // 设置函数参数

    // 数组
    IndexAssign { arr: String, index: Operand, src: Operand },  // arr[index] = src
    IndexLoad { dst: Operand, arr: String, index: Operand },    // dst = arr[index]

    // 输入输出（C-Minus 内置）
    Read(Operand),
    Write(Operand),
}

/// 操作数
pub enum Operand {
    Var(String),          // 临时变量或命名变量
    Const(i32),           // 整数常量
    Label(String),        // 标签
}
```

### 文件结构

```
src/
├── ir/
│   ├── mod.rs
│   ├── tac.rs           # TAC 指令定义
│   ├── temp.rs          # 临时变量生成器
│   ├── label.rs         # 标签生成器
│   └── gen.rs           # IR 生成器：AST → TAC
```

### 生成策略

**表达式**：使用临时变量存储中间结果

```
// a + b * c
t1 = b * c
t2 = a + t1
```

**控制流**：使用标签和条件跳转

```
// if (a > b) then_stmt else else_stmt
    if a > b goto L_true
    goto L_false
L_true:
    ... then_stmt ...
    goto L_end
L_false:
    ... else_stmt ...
L_end:
```

**函数**：每个函数生成一组 TAC 指令

```
// int gcd(int u, int v) { ... }
FuncBegin gcd
Param u
Param v
    ... body ...
FuncEnd gcd
```

**数组**：计算偏移量后用 IndexLoad / IndexAssign

```
// a[i] = x + 1
t1 = x + 1
a[i] = t1
```

### 输出格式示例

对于 GCD 程序，生成的 TAC 类似：

```
FuncBegin gcd(u, v):
    L0:
    if v == 0 goto L1
    goto L2
    L1:
    return u
    L2:
    t0 = u / v
    t1 = t0 * v
    t2 = u - t1
    param v
    param t2
    t3 = call gcd
    return t3
FuncEnd gcd

FuncBegin main():
    L3:
    t4 = call input
    x = t4
    t5 = call input
    y = t5
    param x
    param y
    t6 = call gcd
    param t6
    call output
FuncEnd main
```

---

## 第五步：代码优化 (Optimization)（可选）

### 目标

在三地址码上进行基本优化，减少冗余计算。

### 可实现的优化

| 优化 | 难度 | 描述 |
|------|------|------|
| 常量折叠 | ⭐ | `2 + 3` → `5`，编译时计算 |
| 常量传播 | ⭐ | `x = 5; y = x + 1` → `y = 6` |
| 死代码删除 | ⭐⭐ | 删除结果未使用的赋值 |
| 复写传播 | ⭐⭐ | `t1 = x; y = t1 + 1` → `y = x + 1` |
| 基本块优化 | ⭐⭐ | 在基本块内做上述优化的组合 |

### 文件结构

```
src/
├── optimize/
│   ├── mod.rs
│   ├── constant_fold.rs    # 常量折叠
│   ├── constant_prop.rs    # 常量传播
│   ├── dead_code.rs        # 死代码删除
│   └── cfg.rs              # 控制流图（基本块划分）
```

> 这一步可以先跳过，先完成代码生成后再回来做优化。一个没有优化的编译器也能正确工作。

---

## 第六步：目标代码生成 (Code Generation)

### 目标

将三地址码翻译为可执行的目标代码。

### 目标平台选择

| 方案 | 优点 | 缺点 |
|------|------|------|
| **MIPS 汇编** | 经典教学选择，SPIM/MARS 模拟器丰富 | 需要模拟器运行 |
| **RISC-V 汇编** | 现代教学趋势，RARS 模拟器可用 | 同上 |
| **x86-64 汇编** | 原生运行，无需模拟器 | 指令集复杂 |
| **LLVM IR** | 可直接用 `llc` 编译为原生代码 | 引入 LLVM 依赖 |

> **建议选择 MIPS 或 RISC-V**，这与编译原理教材（如《编译原理》龙书、Tiger Book）的教学传统一致，且指令集简洁。

### 代码生成要点

#### 寄存器分配

简单方案：不做寄存器分配，所有变量都存在栈上。
进阶方案：使用简单的寄存器分配策略（如只分配临时变量到寄存器）。

#### 栈帧布局 (MIPS 示例)

```
高地址
┌──────────────┐
│  参数 n      │
│  ...         │
│  参数 1      │
│  返回地址     │  ← $ra
│  旧 $fp      │  ← 被保存的 $fp
├──────────────┤ ← $fp (帧指针)
│  局部变量 1   │  -4($fp)
│  局部变量 2   │  -8($fp)
│  ...         │
│  临时变量     │  -N($fp)
├──────────────┤ ← $sp (栈指针)
低地址
```

#### 指令选择示例 (MIPS)

| TAC | MIPS |
|-----|------|
| `t = a + b` | `lw $t0, a($fp)` → `lw $t1, b($fp)` → `add $t2, $t0, $t1` → `sw $t2, t($fp)` |
| `if a > b goto L` | `lw $t0, a($fp)` → `lw $t1, b($fp)` → `sgt $t2, $t0, $t1` → `bnez $t2, L` |
| `param x` | `lw $a0, x($fp)` → (存到参数位置) |
| `call f` | `jal f` |
| `return t` | `lw $v0, t($fp)` → `jr $ra` |

#### 文件结构

```
src/
├── codegen/
│   ├── mod.rs
│   ├── target.rs        # 目标平台描述（寄存器、字长等）
│   ├── frame.rs         # 栈帧布局
│   ├── emit.rs          # TAC → 汇编指令选择
│   └── regalloc.rs      # 寄存器分配（简单版）
```

---

## 第七步：整合与驱动程序 (Driver)

### 目标

将所有阶段串联为完整的编译管线。

### 编译管线

```
源代码 (.cm)
    │
    ▼
 词法分析 (Lexer)  ──→  Token 流
    │                      │
    │                 词法错误收集
    ▼
 语法分析 (Parser)  ──→  AST
    │                      │
    │                 语法错误收集
    ▼
 语义分析 (Analyzer) ──→  带类型的 AST + 符号表
    │                      │
    │                 语义错误收集
    ▼
 IR 生成 (IRGen)    ──→  三地址码 (TAC)
    │
    ▼
 优化 (Optimizer)   ──→  优化后的 TAC  (可选)
    │
    ▼
 代码生成 (CodeGen)  ──→  目标汇编代码 (.s)
```

### main.rs 改造

```rust
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let source_path = &args[1];

    // 1. 读取源文件
    let source = read_source_file(source_path)?;

    // 2. 词法分析
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    if !lexer.errors.is_empty() {
        for err in &lexer.errors { eprintln!("{}", err); }
        std::process::exit(1);
    }

    // 3. 语法分析
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program()?;
    // parser.errors ...

    // 4. 语义分析（符号表 + 类型检查）
    let analyzer = SemanticAnalyzer::new();
    let typed_ast = analyzer.analyze(&ast)?;
    // analyzer.errors ...

    // 5. IR 生成
    let ir = IRGenerator::new().generate(&typed_ast);

    // 6. 优化（可选）
    let ir = Optimizer::new().optimize(ir);

    // 7. 代码生成
    let assembly = CodeGenerator::new().emit(&ir);
    println!("{}", assembly);
}
```

### 命令行接口

```bash
# 完整编译
cm-compiler source.cm -o output.s

# 只输出某个阶段的结果
cm-compiler source.cm --tokens      # 只输出 Token
cm-compiler source.cm --ast         # 只输出 AST
cm-compiler source.cm --tac         # 只输出三地址码
cm-compiler source.cm --symtab      # 输出符号表

# 使用建议：引入 clap 或手动解析参数
```

---

## 总体文件结构（完成后）

```
src/
├── main.rs                 # 编译器驱动入口
├── lib.rs                  # 模块声明
├── source.rs               # 源文件读取
│
├── lexer/                  # ✅ 已完成
│   ├── mod.rs
│   ├── token.rs
│   └── error.rs
│
├── parser/                 # ✅ 已完成
│   ├── mod.rs
│   ├── ast.rs
│   ├── error.rs
│   └── print.rs
│
├── semantic/               # 🆕 第一步 + 第二步
│   ├── mod.rs
│   ├── symbol.rs           # 符号定义
│   ├── symbol_table.rs     # 符号表（作用域栈）
│   ├── types.rs            # ExprType 定义
│   ├── analyzer.rs         # 语义分析器（符号构建 + 类型检查）
│   └── error.rs            # 语义错误定义
│
├── ir/                     # 🆕 第四步
│   ├── mod.rs
│   ├── tac.rs              # 三地址码指令定义
│   ├── temp.rs             # 临时变量生成器
│   ├── label.rs            # 标签生成器
│   └── gen.rs              # AST → TAC 翻译器
│
├── optimize/               # 🆕 第五步（可选）
│   ├── mod.rs
│   ├── cfg.rs              # 控制流图
│   ├── constant_fold.rs
│   ├── dead_code.rs
│   └── ...
│
└── codegen/                # 🆕 第六步
    ├── mod.rs
    ├── target.rs           # 目标平台描述
    ├── frame.rs            # 栈帧布局
    ├── emit.rs             # TAC → 汇编
    └── regalloc.rs         # 寄存器分配
```

---

## 建议的开发顺序与时间估算

| 步骤 | 内容 | 预估复杂度 | 建议 |
|------|------|-----------|------|
| 1 | 符号表 | ⭐⭐ | 先跑通，再考虑优化 |
| 2 | 类型检查 | ⭐⭐⭐ | 规则虽多但每条都不复杂，逐条实现逐条测试 |
| 3 | AST 装饰 | ⭐ | 简单，主要是类型标注 |
| 4 | 中间代码生成 | ⭐⭐⭐ | 核心难点：表达式的临时变量、控制流的标签生成 |
| 5 | 代码优化 | ⭐⭐ | 可选，先跳过 |
| 6 | 目标代码生成 | ⭐⭐⭐⭐ | 最复杂的一步：栈帧、寄存器、调用约定 |
| 7 | 整合驱动 | ⭐ | 串联管线、命令行参数 |

> **核心建议**：每完成一步，都要用 `gcd_test.cm` 这个经典用例验证，确保不破坏已有功能。
