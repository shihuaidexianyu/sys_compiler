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
            '0'..='9' => {
                // 处理数字，数字可能有多位，所以需要循环读取
                let mut num_str = String::new();
                while let Some(&digit) = chars.peek() {
                    if digit.is_digit(10) {
                        num_str.push(digit);
                        chars.next();
                    } else {
                        break;
                    }
                }
                // 转换字符串为整数
                let value = num_str.parse::<i32>().unwrap();
                tokens.push(Token {
                    kind: TokenKind::Number(value),
                    text: num_str,
                });
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
                let mut ident_str = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_alphanumeric() || ch == '_' {
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
