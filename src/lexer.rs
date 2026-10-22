// grammar: derive 让编译器自动生成这些 trait 的实现
// Debug 为了 {:?} 能打印, PartialEq 为了 parser 里能写 t.kind == kind
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Int,         // int 关键字
    Return,      // return 关键字
    Ident,       // 标识符(main 会落在这里)
    Number(i32), // 整型常量,带数值; grammar: 枚举变体可以携带数据
    LParen,      // (
    RParen,      // )
    LBrace,      // {
    RBrace,      // }
    Semicolon,   // ;
}
#[derive(Debug, Clone)]
pub struct Token {
    // grammar: pub 是逐个字段开的,不开的话外面模块就摸不到这个字段
    pub kind: TokenKind,
    pub text: String, // 原始文本,调试和报错用
}

// grammar: &str 是借来的字符串切片; 返回 Vec<Token> 是把所有权交给调用者
pub fn lex(input: &str) -> Vec<Token> {
    // 词法分析器: 将输入字符串切分成 token
    let mut tokens = Vec::new();
    // 使用 peekable 迭代器来方便地查看下一个字符
    // grammar: peekable 包的迭代器能 peek() 偷看下一个字符,但不消费它
    let mut chars = input.chars().peekable();
    // 逐个字符处理输入
    // grammar: while let = 模式匹配得上就一直循环; peek 返回 Option<&char>,
    // 写 Some(&c) 是顺手把引用解开,c 直接就是 char 本体
    while let Some(&c) = chars.peek() {
        match c {
            // 空白字符
            // grammar: | 是「或」模式,其中任意一个匹配都算数
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
                    // grammar: 字符串字面量是 &str,to_string() 复制出一份 String
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
            // grammar: '0'..='9' 是区间模式(含两端),匹配这个范围内的任意字符
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
                                    // grammar: Result 用 expect 处理: 成功拿值,失败带消息 panic
                                    let value = i32::from_str_radix(&hex_str, 16)
                                        .expect("无效的十六进制数");
                                    tokens.push(Token {
                                        kind: TokenKind::Number(value),
                                        // grammar: format! 用法像 println!,但不打印,而是造一个 String
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
                // grammar: match 不能直接用 String 匹配字面量,as_str() 转成 &str 才行
                // _ 是兜底模式: 不是任何关键字,就是普通标识符
                let kind = match ident_str.as_str() {
                    "int" => TokenKind::Int,
                    "return" => TokenKind::Return,
                    _ => TokenKind::Ident,
                };
                // 将 token 加入 tokens 向量
                tokens.push(Token {
                    kind, // grammar: 同名简写,等价于 kind: kind
                    text: ident_str,
                });
            }
            _ => panic!("未知字符: {}", c),
        } // grammar: 结尾不加分号的表达式就是返回值,等价于 return tokens;
    }
    tokens
}
