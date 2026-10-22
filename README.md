# sys_compiler

一个用 Rust 编写的 SysY(C 语言子集)编译器,编译原理课程项目,持续开发中。

## 当前进度

- [x] **词法分析器(lex)**:把源代码切分成 token
- [ ] 语法分析器(parse)
- [ ] 语义分析
- [ ] 代码生成

## 词法分析器支持的 token

| 类别 | 说明 |
|---|---|
| 关键字 | `int`、`return` |
| 标识符 | 字母或 `_` 开头,后跟字母、数字、`_` |
| 整型常量 | 十进制(`42`)、八进制(`012`)、十六进制(`0x1A` / `0XFF`) |
| 分隔符 | `(` `)` `{` `}` `;` |
| 注释 | 单行 `//`、多行 `/* ... */`(不参与 token 生成) |

## 构建与运行

```bash
cargo build
./target/debug/compiler 输入文件.sy
```

程序会读取源文件,把切分出的所有 token 打印到标准输出,例如:

```c
int main() {
    return 42;
}
```

输出:

```
Token { kind: Int, text: "int" }
Token { kind: Ident, text: "main" }
Token { kind: LParen, text: "(" }
...
```

## 项目结构

```
src/main.rs   # lexer 与程序入口
tests/lex/    # 词法分析测试用例(本地,未纳入 git 管理)
```
