use std::env;
use std::fs;

#[derive(Debug, Clone, PartialEq)]
enum TokenKind {
    Int,         // int 关键字
    Return,      // return 关键字
    Ident,       // 标识符(main 会落在这里)
    Number(i32), // 整型常量,带数值
    LParen,      // (
    RParen,      // )
    LBrace,      // {
    RBrace,      // }
    Semicolon,   // ;
}
#[derive(Debug, Clone)]
struct Token {
    kind: TokenKind,
    text: String, // 原始文本,调试和报错用
}

fn lex(input: &str) -> Vec<Token> {
    // 词法分析器: 将输入字符串切分成 token
    let mut tokens = Vec::new();
    // 使用 peekable 迭代器来方便地查看下一个字符
    let mut chars = input.chars().peekable();
    // 逐个字符处理输入
    while let Some(&c) = chars.peek() {
        match c {
            // 空白字符
            ' ' | '\n' | '\t' => {
                chars.next();
            }
            // 单行和多行注释
            '/' => {
                chars.next(); // 跳过 '/'
                if let Some(&next_char) = chars.peek() {
                    if next_char == '/' {
                        // 处理单行注释
                        while let Some(&ch) = chars.peek() {
                            if ch == '\n' {
                                break; // 遇到换行符结束注释
                            }
                            chars.next(); // 跳过注释内容
                        }
                    } else if next_char == '*' {
                        // 处理多行注释
                        chars.next(); // 跳过 '*'
                        while let Some(&ch) = chars.peek() {
                            if ch == '*' {
                                chars.next(); // 跳过 '*'
                                if let Some(&next_ch) = chars.peek() {
                                    if next_ch == '/' {
                                        chars.next(); // 跳过 '/'
                                        break; // 结束多行注释
                                    }
                                }
                            } else {
                                chars.next(); // 跳过注释内容
                            }
                        }
                    }
                }
            }
            '(' => {
                tokens.push(Token {
                    kind: TokenKind::LParen,
                    text: "(".to_string(),
                });
                chars.next();
            }
            ')' => {
                tokens.push(Token {
                    kind: TokenKind::RParen,
                    text: ")".to_string(),
                });
                chars.next();
            }
            '{' => {
                tokens.push(Token {
                    kind: TokenKind::LBrace,
                    text: "{".to_string(),
                });
                chars.next();
            }
            '}' => {
                tokens.push(Token {
                    kind: TokenKind::RBrace,
                    text: "}".to_string(),
                });
                chars.next();
            }
            ';' => {
                tokens.push(Token {
                    kind: TokenKind::Semicolon,
                    text: ";".to_string(),
                });
                chars.next();
            }
            /*
            integer-const       ::= decimal-const
                                  | octal-const
                                  | hexadecimal-const;
            decimal-const       ::= nonzero-digit
                                  | decimal-const digit;
            octal-const         ::= "0"
                                  | octal-const octal-digit;
            hexadecimal-const   ::= hexadecimal-prefix hexadecimal-digit
                                  | hexadecimal-const hexadecimal-digit;
            hexadecimal-prefix  ::= "0x" | "0X";
             */
            '0'..='9' => {
                // 外层 peek 到的 c 就是首字符(还没消费),直接按它分派
                match c {
                    '0' => {
                        // 处理以 0 开头的数字,可能是八进制或十六进制
                        chars.next(); // 跳过 '0'
                        if let Some(&next_char) = chars.peek() {
                            match next_char {
                                'x' | 'X' => {
                                    // 十六进制
                                    chars.next(); // 跳过 'x' 或 'X'
                                    let mut hex_str = String::new();
                                    while let Some(&ch) = chars.peek() {
                                        if ch.is_ascii_hexdigit() {
                                            hex_str.push(ch);
                                            chars.next();
                                        } else {
                                            break;
                                        }
                                    }
                                    let value = i32::from_str_radix(&hex_str, 16)
                                        .expect("无效的十六进制数");
                                    tokens.push(Token {
                                        kind: TokenKind::Number(value),
                                        text: format!("0x{}", hex_str),
                                    });
                                }
                                '0'..='7' => {
                                    // 八进制
                                    let mut octal_str = String::from("0");
                                    while let Some(&ch) = chars.peek() {
                                        if ch >= '0' && ch <= '7' {
                                            octal_str.push(ch);
                                            chars.next();
                                        } else {
                                            break;
                                        }
                                    }
                                    let value =
                                        i32::from_str_radix(&octal_str, 8).expect("无效的八进制数");
                                    tokens.push(Token {
                                        kind: TokenKind::Number(value),
                                        text: octal_str,
                                    });
                                }
                                _ => {
                                    // 单独的 0
                                    tokens.push(Token {
                                        kind: TokenKind::Number(0),
                                        text: "0".to_string(),
                                    });
                                }
                            }
                        } else {
                            // 单独的 0
                            tokens.push(Token {
                                kind: TokenKind::Number(0),
                                text: "0".to_string(),
                            });
                        }
                    }
                    _ => {
                        // c 只可能是 '1'..='9'(外层分支已排除 '0'):十进制数
                        let mut decimal_str = String::new();
                        while let Some(&ch) = chars.peek() {
                            if ch >= '0' && ch <= '9' {
                                decimal_str.push(ch);
                                chars.next();
                            } else {
                                break;
                            }
                        }
                        let value = i32::from_str_radix(&decimal_str, 10).expect("无效的十进制数");
                        tokens.push(Token {
                            kind: TokenKind::Number(value),
                            text: decimal_str,
                        });
                    }
                }
            }
            // 处理标识符和关键字
            // 1. 先得出来整个标识符的字符串
            // 2. 再判断这个字符串是关键字(关键字其实是符合标识符语法的标识符)还是普通标识符
            /*
            identifier ::= identifier-nondigit
                   | identifier identifier-nondigit
                   | identifier digit;
            */
            'a'..='z' | 'A'..='Z' | '_' => {
                // 不能由数字开头,所以这里不处理数字
                let mut ident_str = String::new();
                while let Some(&ch) = chars.peek() {
                    // 除了第一个字符外，其余字符可以由字母、数字和下划线组成
                    if ch.is_ascii_alphanumeric() || ch == '_' {
                        ident_str.push(ch);
                        chars.next();
                    } else {
                        break;
                    }
                }
                // 判断标识符是关键字还是普通标识符
                let kind = match ident_str.as_str() {
                    "int" => TokenKind::Int,
                    "return" => TokenKind::Return,
                    _ => TokenKind::Ident,
                };
                // 将 token 加入 tokens 向量
                tokens.push(Token {
                    kind,
                    text: ident_str,
                });
            }
            _ => panic!("未知字符: {}", c),
        }
    }
    tokens
}

fn parse(tokens: &[Token]) {
    // 语法分析器: 将 token 序列解析成抽象语法树(AST)
    // 这里我们只做一个简单的示例，打印出 token 的种类和文本
    for token in tokens {
        println!("{:?}: {}", token.kind, token.text);
    }
}

fn main() {
    // 用法: sysy-compiler 输入文件.sy
    let args: Vec<String> = env::args().collect();
    let input = fs::read_to_string(&args[1]).expect("读源文件失败");

    // 先验证 lexer: 把切出来的 token 全打印出来
    let tokens = lex(&input);
    for t in &tokens {
        println!("{:?}", t);
    }
}
