// 抽象语法树(AST)节点定义。
// parser 产出 AST,之后的语义分析、代码生成都消费它,所以独立成模块。
// 全部标 pub: 字段默认私有,别的模块构造/读取都要靠 pub 开门。

//grammar: #![allow(dead_code)]作用域为整个模块,#[allow(dead_code)]作用域为紧随其后的语句,所以这里用 #![allow(dead_code)] 来避免编译器抱怨未使用的 AST 节点。

#![allow(dead_code)]

// grammar: trait = 「合同」,只规定要有哪些方法(签名),不给实现
//   比如 Debug 合同要求一个 fmt 方法,签完合同(实现它)的类型才能用 {:?} 打印
//   手写履约: impl Debug for CompUnit { fn fmt(...) {...} }
// grammar: derive = 「自动履约」。对有固定套路的合同(Debug/Clone/PartialEq 等),
//   编译器替你生成 impl 块,套路就是逐字段递归处理
//   所以有传染性: 字段类型也得实现同一个 trait 才能 derive
//   没有显然套路的合同(如 Display)不能 derive,只能手写 impl
// grammar: derive 是连锁反应——往下传染要求,往上白嫖成果:
//   用「已实现某 trait 的类型」当字段建 struct,这个 struct 就能 derive 这个 trait
//   有一项字段没实现就 derive 失败; 清单里每个 trait 独立检查(字段们既要都会 Debug 也要都会 Clone)
//   i32/String/Vec 等标准库类型基本都实现了常用 trait,只有自定义类型才要自己加

// 编译单元 = 一个函数定义
// grammar: derive(Debug) 自动生成 Debug 实现,main 里 println!("{:?}") 才能打印
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
