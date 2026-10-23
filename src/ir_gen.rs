use crate::ast;
pub fn gen_ir(ast: &ast::CompUnit) -> String {
    let mut ir = String::new();
    // 生成函数定义的 IR
    // grammar: format! 是宏,{} 是占位符,{} 里可以放变量/表达式,返回 String
    ir.push_str(&format!(
        // grammar: 用 {{ 来转义 { ,否则 { 会被当成未闭合的占位符
        "fun @{}(): {} {{\n",
        ast.func_def.ident,
        match ast.func_def.func_type {
            // 这里是因为 Lv1 只有 int 一种,所以直接写死成 i32,以后 Lv4 要加 void 就得改这里
            ast::FuncType::Int => "i32",
        }
    ));
    // 生成函数体的 IR
    ir.push_str(&gen_block_ir(&ast.func_def.block));
    ir.push_str("}\n");
    ir
}

pub fn gen_block_ir(block: &ast::Block) -> String {
    let mut ir = String::new();
    // 生成语句的 IR
    ir.push_str("%entry:\n");
    ir.push_str(&gen_stmt_ir(&block.stmt));
    ir
}

pub fn gen_stmt_ir(stmt: &ast::Stmt) -> String {
    match stmt {
        ast::Stmt::Return(value) => {
            // 生成 return 语句的 IR
            format!("  ret {}\n", value)
        }
    }
}
