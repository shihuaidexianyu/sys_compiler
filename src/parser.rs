use crate::ast::*;
use crate::lexer::{Token, TokenKind};

/*
CompUnit  ::= FuncDef;

FuncDef   ::= FuncType IDENT "(" ")" Block;
FuncType  ::= "int";

Block     ::= "{" Stmt "}";
Stmt      ::= "return" Number ";";
Number    ::= INT_CONST;
*/

// 语法分析器: 递归下降。每条文法规则对应一个函数,
// pos 是游标,指向「下一个还没消费的 token」
struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl<'a> Parser<'a> {
    // 看当前 token,不消费
    fn peek(&self) -> Option<&'a Token> {
        self.tokens.get(self.pos)
    }

    // 消费当前 token 并返回它
    fn bump(&mut self) -> &'a Token {
        let t = &self.tokens[self.pos];
        self.pos += 1;
        t
    }

    // 当前 token 必须是 kind,否则报错;通过则消费掉
    fn expect(&mut self, kind: TokenKind) -> &'a Token {
        match self.peek() {
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
        let ident = self.expect(TokenKind::Ident).text.clone();
        self.expect(TokenKind::LParen);
        self.expect(TokenKind::RParen);
        let block = self.block();
        FuncDef {
            func_type: FuncType::Int,
            ident,
            block,
        }
    }

    fn block(&mut self) -> Block {
        self.expect(TokenKind::LBrace);
        let stmt = self.stmt();
        self.expect(TokenKind::RBrace);
        Block { stmt }
    }

    fn stmt(&mut self) -> Stmt {
        self.expect(TokenKind::Return);
        let value = match self.bump() {
            Token {
                kind: TokenKind::Number(n),
                ..
            } => *n,
            t => panic!("期望整型常量,实际是 {:?}", t.kind),
        };
        self.expect(TokenKind::Semicolon);
        Stmt::Return(value)
    }
}

pub fn parse(tokens: &[Token]) -> CompUnit {
    let mut parser = Parser { tokens, pos: 0 };
    parser.comp_unit()
}
