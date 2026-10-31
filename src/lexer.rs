// grammar: derive 让编译器自动实现列出的 trait,不用我们手写 impl。
// grammar: Debug 让 token 能用 {:?} 打印;Clone 允许克隆;PartialEq 允许用 == 比较。
// grammar: derive 会递归要求字段也实现对应 trait,所以 Number 里的 i32 也必须支持它们。
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // 关键字
    Const,
    Int,
    Void,
    If,
    Else,
    While,
    Break,
    Continue,
    Return,

    // 标识符和整型常量
    Ident,
    // grammar: 枚举变体可以携带数据;Number(42) 同时保存「数字 token」和数值 42。
    Number(i32),

    // 运算符
    Plus,         // +
    Minus,        // -
    Star,         // *
    Slash,        // /
    Percent,      // %
    Bang,         // !
    Assign,       // =
    Equal,        // ==
    NotEqual,     // !=
    Less,         // <
    LessEqual,    // <=
    Greater,      // >
    GreaterEqual, // >=
    LogicalAnd,   // &&
    LogicalOr,    // ||

    // 分隔符
    LeftParenthesis,  // (
    RightParenthesis, // )
    LeftBracket,      // [
    RightBracket,     // ]
    LeftBrace,        // {
    RightBrace,       // }
    Comma,            // ,
    Semicolon,        // ;
}

#[derive(Debug, Clone)]
pub struct Token {
    // grammar: struct 字段默认私有;加 pub 后,parser 等其他模块才能读取。
    pub kind: TokenKind,
    // String 拥有这段文本;词法分析结束后,token 不依赖输入字符串也能存活。
    pub text: String,
}

// grammar: &str 是借来的字符串切片;函数只读取它,不取得调用者字符串的所有权。
fn keyword_or_ident(text: &str) -> TokenKind {
    // grammar: match 按值逐一匹配;_ 是通配符,处理前面没有列出的所有情况。
    match text {
        "const" => TokenKind::Const,
        "int" => TokenKind::Int,
        "void" => TokenKind::Void,
        "if" => TokenKind::If,
        "else" => TokenKind::Else,
        "while" => TokenKind::While,
        "break" => TokenKind::Break,
        "continue" => TokenKind::Continue,
        "return" => TokenKind::Return,
        _ => TokenKind::Ident,
    }
}

// grammar: 返回 Vec<Token> 会把整个动态数组的所有权交给调用者。
pub fn lex(input: &str) -> Vec<Token> {
    // 有些文件开头带 UTF-8 BOM,先把它去掉。
    // grammar: strip_prefix 返回 Option<&str>:有前缀就返回去掉前缀后的切片,没有就返回 None。
    // grammar: unwrap_or(input) 在 None 时使用原输入;这里的 let 还隐藏(shadow)了旧 input。
    let input = input.strip_prefix('\u{feff}').unwrap_or(input);
    let mut tokens = Vec::new();
    // grammar: chars() 产生字符迭代器;peekable() 让它能偷看下一个字符而不消费。
    // grammar: mut 表示变量绑定可修改;peek/next 需要改变迭代器内部状态。
    let mut chars = input.chars().peekable();

    // grammar: while let 会在模式匹配成功时循环;输入结束得到 None,循环自动停止。
    // grammar: peek 返回 Option<&char>;Some(&c) 在匹配时解开引用,得到 char 本体 c。
    while let Some(&c) = chars.peek() {
        // 空白字符直接跳过。
        if c.is_ascii_whitespace() {
            chars.next();
            // grammar: continue 跳过本次循环剩余部分,直接处理下一个字符。
            continue;
        }

        // '/' 既可能是除号,也可能是注释的开头,所以单独处理。
        if c == '/' {
            // grammar: next() 消费迭代器中的下一个字符;这里先消费已经确认的 '/'。
            chars.next();
            // grammar: peek() 只看不消费;Some(&'/') 表示下一个字符必须正好是 '/'。
            if chars.peek() == Some(&'/') {
                // 单行注释:一直跳到换行符之前。
                chars.next();
                while let Some(&ch) = chars.peek() {
                    if ch == '\n' {
                        // grammar: break 只结束离它最近的循环,这里是结束跳过注释的 while。
                        break;
                    }
                    chars.next();
                }
            } else if chars.peek() == Some(&'*') {
                // 多行注释:一直跳到第一个 */。
                chars.next();
                let mut closed = false;
                // tip: next() 直接返回 Option<char>,所以这里的 ch 不用写成 &ch。
                // 它和上面 peek() 返回 Option<&char> 是两种不同的情况。
                while let Some(ch) = chars.next() {
                    // grammar: && 是逻辑与,只有两个条件都成立才进入 if。
                    if ch == '*' && chars.peek() == Some(&'/') {
                        chars.next();
                        closed = true;
                        break;
                    }
                }
                // grammar: !closed 是布尔取反,等价于 closed == false。
                if !closed {
                    panic!("未闭合的多行注释");
                }
            } else {
                // 后面不是 / 或 *,说明它就是除号。
                tokens.push(Token {
                    kind: TokenKind::Slash,
                    // grammar: 字符串字面量是 &str;to_string() 创建一个拥有所有权的 String。
                    text: "/".to_string(),
                });
            }
            continue;
        }

        // 整型常量:十进制、八进制或十六进制。
        if c.is_ascii_digit() {
            // grammar: String::new() 创建空的、可增长的字符串;push() 往末尾追加字符。
            let mut digits = String::new();
            let mut prefix = String::new();
            // radix 表示进制:十进制是 10,八进制是 8,十六进制是 16。
            let mut radix = 10;

            if c == '0' {
                chars.next();
                digits.push('0');
                radix = 8;

                // grammar: || 是逻辑或;左右任意一个条件成立即可。
                if chars.peek() == Some(&'x') || chars.peek() == Some(&'X') {
                    // 0x 和 0X 是前缀,不参与数值转换。
                    chars.next();
                    // grammar: clear() 清空 String 的内容,但保留这个 String 本身继续复用。
                    digits.clear();
                    prefix = "0x".to_string();
                    radix = 16;
                    while let Some(&ch) = chars.peek() {
                        if ch.is_ascii_hexdigit() {
                            digits.push(ch);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    if digits.is_empty() {
                        panic!("0x/0X 后至少要有一个十六进制数字");
                    }
                } else {
                    // 开头的 0 也属于八进制数字字符串。
                    while let Some(&ch) = chars.peek() {
                        if ch.is_ascii_digit() {
                            digits.push(ch);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                }
            } else {
                while let Some(&ch) = chars.peek() {
                    if ch.is_ascii_digit() {
                        digits.push(ch);
                        chars.next();
                    } else {
                        break;
                    }
                }
            }

            // 不把 123abc 或 0x1g 错切成两个 token。
            // grammar: if let 只在 Option 是 Some 时进入分支;是 None 就直接跳过。
            if let Some(&ch) = chars.peek() {
                if ch.is_ascii_alphabetic() || ch == '_' {
                    panic!("整型常量后不能直接跟字母或下划线");
                }
            }

            // 无效数字(例如八进制 08)和整数溢出都会在这里报错。
            // grammar: from_str_radix 返回 Result;expect 成功时取出 i32,失败时带消息 panic。
            // grammar: &digits 只把字符串借给转换函数,转换结束后 digits 仍然归当前函数所有。
            let value =
                i32::from_str_radix(&digits, radix).expect("整型常量无效或超出 [0, 2147483647]");
            // grammar: push_str 追加的是借来的字符串切片,所以这里使用 &digits。
            prefix.push_str(&digits);
            tokens.push(Token {
                kind: TokenKind::Number(value),
                // grammar: prefix 的所有权移动进 Token;移动后不能再使用原变量 prefix。
                text: prefix,
            });
            continue;
        }

        // 标识符或关键字。
        // 1. 先收集完整字符串;2. 再判断它是关键字还是普通标识符。
        if c.is_ascii_alphabetic() || c == '_' {
            let mut text = String::new();
            while let Some(&ch) = chars.peek() {
                // 标识符首字符不能是数字,但后续字符可以包含数字。
                if ch.is_ascii_alphanumeric() || ch == '_' {
                    text.push(ch);
                    chars.next();
                } else {
                    break;
                }
            }
            // grammar: 参数需要 &str,而 &text 是 &String;Rust 会通过自动解引用完成转换。
            // 这里只是借用 text,不会移动它,所以下一行仍能把它放进 Token。
            let kind = keyword_or_ident(&text);
            // grammar: 字段名和变量名相同时可写简写;这里等价于 Token { kind: kind, text: text }。
            tokens.push(Token { kind, text });
            continue;
        }

        // 其余符号共用一次 tokens.push。text 初始是单个字符,
        // 遇到 <=、==、&& 等符号时再把第二个字符补上。
        // grammar: char 也实现了 ToString,所以 c.to_string() 会得到一个字符的 String。
        let mut text = c.to_string();
        // grammar: match 本身也是表达式;每个分支产出一个 TokenKind,赋给左边的 kind。
        let kind = match c {
            '+' => {
                chars.next();
                TokenKind::Plus
            }
            '-' => {
                chars.next();
                TokenKind::Minus
            }
            '*' => {
                chars.next();
                TokenKind::Star
            }
            '%' => {
                chars.next();
                TokenKind::Percent
            }
            '(' => {
                chars.next();
                TokenKind::LeftParenthesis
            }
            ')' => {
                chars.next();
                TokenKind::RightParenthesis
            }
            '[' => {
                chars.next();
                TokenKind::LeftBracket
            }
            ']' => {
                chars.next();
                TokenKind::RightBracket
            }
            '{' => {
                chars.next();
                TokenKind::LeftBrace
            }
            '}' => {
                chars.next();
                TokenKind::RightBrace
            }
            ',' => {
                chars.next();
                TokenKind::Comma
            }
            ';' => {
                chars.next();
                TokenKind::Semicolon
            }
            // grammar: 分支块最后一条表达式不加分号,它就是该分支产出的值。
            // 例如 chars.next(); 只是执行操作,后面的 TokenKind::Bang 才是返回值。
            '!' => {
                chars.next();
                // grammar: if/else 也都是表达式,两个分支这里分别返回不同的 TokenKind。
                if chars.peek() == Some(&'=') {
                    chars.next();
                    text.push('=');
                    TokenKind::NotEqual
                } else {
                    TokenKind::Bang
                }
            }
            '=' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                    text.push('=');
                    TokenKind::Equal
                } else {
                    TokenKind::Assign
                }
            }
            '<' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                    text.push('=');
                    TokenKind::LessEqual
                } else {
                    TokenKind::Less
                }
            }
            '>' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                    text.push('=');
                    TokenKind::GreaterEqual
                } else {
                    TokenKind::Greater
                }
            }
            '&' => {
                chars.next();
                if chars.peek() == Some(&'&') {
                    chars.next();
                    text.push('&');
                    TokenKind::LogicalAnd
                } else {
                    panic!("SysY 的逻辑与必须写成 &&");
                }
            }
            '|' => {
                chars.next();
                if chars.peek() == Some(&'|') {
                    chars.next();
                    text.push('|');
                    TokenKind::LogicalOr
                } else {
                    panic!("SysY 的逻辑或必须写成 ||");
                }
            }
            // grammar: _ 匹配前面没有列出的所有字符;遇到它们说明输入不在 SysY 词法范围内。
            // grammar: panic! 会直接终止程序,因此这个分支不需要再返回 TokenKind。
            _ => panic!("未知字符: {}", c),
        };
        tokens.push(Token { kind, text });
    }

    // grammar: 函数末尾不加分号的表达式就是返回值,等价于 return tokens;。
    // 这里把 tokens 的所有权交给 main,而不是返回对它的借用。
    tokens
}
