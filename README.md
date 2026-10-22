# sys_compiler

一个用 Rust 编写的 SysY(C 语言子集)编译器,编译原理课程项目,持续开发中。
> 顺带复习从未学扎实的 Rust 语法。 

## 词法分析器支持的 token

| 类别 | 说明 |
|---|---|
| 关键字 | `int`、`return` |
| 标识符 | 字母或 `_` 开头,后跟字母、数字、`_` |
| 整型常量 | 十进制(`42`)、八进制(`012`)、十六进制(`0x1A` / `0XFF`) |
| 分隔符 | `(` `)` `{` `}` `;` |
| 注释 | 单行 `//`、多行 `/* ... */`(不参与 token 生成) |

## 语法分析器支持的AST

递归下降实现,每条文法规则对应一个函数,`peek` / `bump` / `expect` 三件套驱动游标前进。

当前支持的文法(Lv1):

```ebnf
CompUnit  ::= FuncDef;
FuncDef   ::= FuncType IDENT "(" ")" Block;
FuncType  ::= "int";
Block     ::= "{" Stmt "}";
Stmt      ::= "return" Number ";";
Number    ::= INT_CONST;
```

AST 节点一览:

| 节点 | 含义 | 备注 |
|---|---|---|
| `CompUnit` | 编译单元 = 一个函数定义 | |
| `FuncDef` | 函数定义: 返回类型 + 名字 + 函数体 | |
| `FuncType` | 返回类型 | 单分支枚举,只有 `Int`,Lv4 加 `void` |
| `Block` | 语句块 `{ ... }` | |
| `Stmt` | 语句 | 单分支枚举,只有 `Return(i32)`(Number 已拍平成值),Lv2 起扩展赋值、if、while |

> 单分支枚举看起来像废话,是刻意为后续等级留的扩展位,加新语法时往里塞分支即可。

## 项目结构

```text
src/
  main.rs    入口: 读文件 → lex → 打印 token → parse → 打印 AST
  lexer.rs   词法分析: &str → Vec<Token>
  parser.rs  语法分析: &[Token] → AST(递归下降)
  ast.rs     AST 节点定义(纯数据结构)
tests/
  lex/       词法测试样例(.sy)
```

## 运行

```bash
cargo run -- tests/lex/basic.sy
```

输出分两段: 先是切出来的所有 token(`{:?}`),再是解析出的 AST(`{:#?}` 美化打印)。
