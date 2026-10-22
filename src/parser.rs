use crate::ast::*;
use crate::lexer::{Token, TokenKind};

// 语法分析器: 递归下降。每条文法规则对应一个函数,
// pos 是游标,指向「下一个还没消费的 token」
// grammar: 结构体里只要存了借用(&),就必须在名字后面声明生命周期占位符 <'a>
struct Parser<'a> {
    // &'a [Token]: 借来的 token 切片,这个借用的寿命标记为 'a
    tokens: &'a [Token],
    pos: usize, // pos 是自己的数据不是借用,不用贴标记
}
// grammar: impl后面<>里一般会放占位符，包括泛型，生命周期
// impl<'a> 是「登记名字」,Parser<'a> 是「使用名字」;不登记的话后面谁都不认识 'a
impl<'a> Parser<'a> {
    // 看当前 token,不消费
    // grammar: &self = 只读借用自己; Option = 可能有也可能没有(读到头就是 None)
    // 返回 &'a Token: 指的是 tokens 里的元素,寿命跟 'a 走,和 &self 这次调用无关
    fn peek(&self) -> Option<&'a Token> {
        self.tokens.get(self.pos)
    }

    // 消费当前 token 并返回它
    // grammar: &mut self = 可变借用自己,因为要把 pos 加一
    fn bump(&mut self) -> &'a Token {
        let t = &self.tokens[self.pos];
        self.pos += 1;
        t
    }

    // 当前 token 必须是 kind,否则报错;通过则消费掉
    fn expect(&mut self, kind: TokenKind) -> &'a Token {
        match self.peek() {
            // grammar: 分支里的 if 叫「匹配守卫」,形状对上后还要再满足 if 才算数
            Some(t) if t.kind == kind => self.bump(),
            Some(t) => panic!("期望 {:?},实际是 {:?}", kind, t.kind),
            None => panic!("期望 {:?},但输入已结束", kind),
        }
    }

    fn comp_unit(&mut self) -> CompUnit {
        CompUnit {
            func_def: self.func_def(),
        }
    }

    fn func_def(&mut self) -> FuncDef {
        self.expect(TokenKind::Int); // FuncType ::= "int"
        // expect 只还回借用 &Token,想要 String 本体得 clone 复制一份(所有权问题)
        let ident = self.expect(TokenKind::Ident).text.clone();
        self.expect(TokenKind::LParen);
        self.expect(TokenKind::RParen);
        let block = self.block();
        FuncDef {
            func_type: FuncType::Int,
            ident, // grammar: 字段名和变量名同名的简写,等价于 ident: ident
            block, // 同上
        }
    }

    fn block(&mut self) -> Block {
        self.expect(TokenKind::LBrace);
        let stmt = self.stmt();
        self.expect(TokenKind::RBrace);
        Block { stmt } // 同样的简写,等价于 Block { stmt: stmt }
    }
    // tip: t是通配符,匹配所有类型的 token,所以可以用来做错误处理
    // 区别于_,是因为_没法访问对应的 token,而 t 可以访问 token 的 kind 和 text
    fn stmt(&mut self) -> Stmt {
        self.expect(TokenKind::Return);
        let value = match self.bump() {
            // grammar: 结构体解构 + 枚举模式嵌套,要求 kind 是 Number 并把数值抓出来叫 n
            // .. 表示「其余字段(这里的 text)我不关心」
            Token {
                kind: TokenKind::Number(n),
                ..
            } => *n, // 隔着引用匹配出来的 n 是 &i32,加 * 取数字本体
            t => panic!("期望整型常量,实际是 {:?}", t.kind),
        };
        self.expect(TokenKind::Semicolon);
        Stmt::Return(value)
    }
}

// grammar: 函数参数上的生命周期编译器能自动推断,所以这里不用写 <'a>
pub fn parse(tokens: &[Token]) -> CompUnit {
    // 用 tokens 初始化 parser, pos 从 0 开始
    let mut parser = Parser { tokens, pos: 0 };
    // 此函数会一层一层调用 parser 的方法,最终返回 CompUnit
    let comp_unit = parser.comp_unit();
    // 整个编译单元解析完后,游标必须正好停在 token 流末尾;
    // 否则说明有 parser 不认识的残余 token —— 语法错误
    if parser.pos != tokens.len() {
        panic!(
            "解析结束后仍有未消费的 token, 第一个是: {:?}",
            tokens[parser.pos]
        );
    }
    comp_unit
}
