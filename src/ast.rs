// 抽象语法树(AST)节点定义。
// parser 产出 AST,之后的语义分析、代码生成都消费它,所以独立成模块。
// 全部标 pub: 字段默认私有,别的模块构造/读取都要靠 pub 开门。

// 编译单元 = 一个函数定义
#[derive(Debug)]
pub struct CompUnit {
    pub func_def: FuncDef,
}

// 函数定义: 类型 + 名字 + 函数体
#[derive(Debug)]
pub struct FuncDef {
    pub func_type: FuncType,
    pub ident: String,
    pub block: Block,
}

// Lv1 只有 int 一种,枚举只有一个分支 —— 看起来像废话,但 Lv4 要加 void,
// 现在写成枚举是为以后留的位置
#[derive(Debug)]
pub enum FuncType {
    Int,
}

#[derive(Debug)]
pub struct Block {
    pub stmt: Stmt,
}

// Lv1 的语句只有 return 一种;同样写成单分支枚举,
// Lv2 起会往这里塞赋值、if、while…… 枚举一分支一个
#[derive(Debug)]
pub enum Stmt {
    Return(i32), // Number 拍平: return 后面就一个数,直接存值
}
