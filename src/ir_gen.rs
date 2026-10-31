use crate::ast::*;

/*
AST -> Koopa IR 的核心是后序遍历:先生成子表达式,再生成当前运算。

每个 gen_xxx 表达式函数都遵守同一条约定:
1. 把计算子表达式所需的指令写入 IrGen::body。
2. 返回一个 String,表示当前表达式结果的 Koopa Value。

返回值有两种常见形式:
- 常量:例如 "42",不需要生成指令。
- 临时值:例如 "%0",表示前面某条指令的计算结果。

因此父表达式不需要关心子表达式是数字还是指令结果,都可以直接把它填进下一条指令。
优先级也不需要在这里重新判断:parser 已经通过 AST 的形状确定了递归生成顺序。
*/

// 生成一个函数的 IR。表达式指令和临时值编号都集中在 IrGen 中管理。
pub fn gen_ir(ast: &CompUnit) -> String {
    let return_type = match &ast.func_def.func_type {
        FuncType::Int => "i32",
    };
    let mut generator = IrGen::new();
    generator.gen_block(&ast.func_def.block);

    // gen_block/gen_stmt 只生成函数体,这里补上 Koopa 的函数定义框架。
    let mut ir = format!("fun @{}(): {} {{\n", ast.func_def.ident, return_type);
    ir.push_str(&generator.body);
    ir.push_str("}\n");
    ir
}

struct IrGen {
    // 当前函数体内已经生成的指令和基本块。
    body: String,
    // 下一个临时值编号,用于生成 %0、%1……,防止重名。
    next_temp: usize,
    // 下一个基本块标签编号,和临时值分开计数,方便阅读。
    next_label: usize,
}

impl IrGen {
    fn new() -> Self {
        Self {
            body: String::new(),
            next_temp: 0,
            next_label: 0,
        }
    }

    // 生成 %0、%1……,保证同一个函数内的临时值不会重名。
    // &mut self 表示这个函数会修改 next_temp;调用后编号自动加一。
    fn new_temp(&mut self) -> String {
        let name = format!("%{}", self.next_temp);
        self.next_temp += 1;
        name
    }

    // 基本块标签和临时值分开命名,方便阅读生成的 IR。
    fn new_label(&mut self, prefix: &str) -> String {
        let label = format!("%{}_{}", prefix, self.next_label);
        self.next_label += 1;
        label
    }

    // 生成一条二元运算,并返回保存结果的临时值。
    // 例如 lhs="1"、rhs="%0"、op="add" 时,向 body 追加 "%1 = add 1, %0",
    // 然后返回 "%1" 给父表达式继续使用。
    fn emit_binary(&mut self, op: &str, lhs: String, rhs: String) -> String {
        let result = self.new_temp();
        self.body
            .push_str(&format!("  {} = {} {}, {}\n", result, op, lhs, rhs));
        result
    }

    // 短路表达式可能从两个基本块到达同一个出口,两个路径都要能提供结果。
    // 这里先用 alloc 得到一块临时内存,再由各路径 store,出口处统一 load。
    fn emit_alloc(&mut self) -> String {
        let result = self.new_temp();
        self.body.push_str(&format!("  {} = alloc i32\n", result));
        result
    }

    fn emit_store(&mut self, value: &str, pointer: &str) {
        self.body
            .push_str(&format!("  store {}, {}\n", value, pointer));
    }

    fn emit_load(&mut self, pointer: &str) -> String {
        let result = self.new_temp();
        self.body
            .push_str(&format!("  {} = load {}\n", result, pointer));
        result
    }

    fn gen_block(&mut self, block: &Block) {
        // Koopa 函数体的第一个基本块是入口基本块。
        self.body.push_str("%entry:\n");
        self.gen_stmt(&block.stmt);
    }

    fn gen_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Return(exp) => {
                // gen_exp 先产生表达式所需指令,再把返回的 Value 填进 ret。
                let value = self.gen_exp(exp);
                self.body.push_str(&format!("  ret {}\n", value));
            }
        }
    }

    // 每个 gen_xxx 都返回一个 Koopa Value:可能是整数,也可能是 %0 这样的符号。
    // 这一层只负责按照 Exp 的变体分派,不直接生成运算。
    fn gen_exp(&mut self, exp: &Exp) -> String {
        match exp {
            Exp::UnaryExp(exp) => self.gen_unary_exp(exp),
            Exp::MulExp(exp) => self.gen_mul_exp(exp),
            Exp::AddExp(exp) => self.gen_add_exp(exp),
            Exp::LOrExp(exp) => self.gen_l_or_exp(exp),
        }
    }

    fn gen_primary_exp(&mut self, exp: &PrimaryExp) -> String {
        match exp {
            // 数字本身就是 Koopa Value,不需要额外指令。
            PrimaryExp::Number(value) => value.to_string(),
            // Box 会在传参时自动解引用。
            // 括号已经在 parser 中改变了 AST 结构,生成 IR 时只需递归处理内部表达式。
            PrimaryExp::Exp(exp) => self.gen_exp(exp),
        }
    }

    // Koopa 没有一元指令,因此需要降低(lower)为二元运算:
    // +x 直接返回 x;-x 变成 sub 0, x;!x 变成 eq x, 0。
    fn gen_unary_exp(&mut self, exp: &UnaryExp) -> String {
        match exp {
            UnaryExp::PrimaryExp(exp) => self.gen_primary_exp(exp),
            UnaryExp::UnaryOp(op, exp) => {
                // 必须先生成内部表达式,才能使用它的 Value 生成外层运算。
                let value = self.gen_unary_exp(exp);
                match op {
                    UnaryOp::Plus => value,
                    UnaryOp::Minus => self.emit_binary("sub", "0".to_string(), value),
                    UnaryOp::Bang => self.emit_binary("eq", value, "0".to_string()),
                }
            }
        }
    }

    // 所有普通二元表达式层都采用同一种模式:
    // - 没有本层运算符的变体,直接交给下一层,不产生指令。
    // - 有二元运算符的变体,先递归生成 lhs 和 rhs,再生成当前运算。
    // 例如 1 + 2 * 3 会先生成 "%0 = mul 2, 3",再生成 "%1 = add 1, %0"。
    // AST 已经决定了这棵树,所以这里不需要重新比较运算符优先级。
    fn gen_mul_exp(&mut self, exp: &MulExp) -> String {
        match exp {
            MulExp::UnaryExp(exp) => self.gen_unary_exp(exp),
            MulExp::MulExp(lhs, op, rhs) => {
                // 先生成左右子树,再生成当前运算;AST 已经处理了优先级。
                let lhs = self.gen_mul_exp(lhs);
                let rhs = self.gen_unary_exp(rhs);
                let op = match op {
                    MulOp::Star => "mul",
                    MulOp::Slash => "div",
                    MulOp::Percent => "mod",
                };
                self.emit_binary(op, lhs, rhs)
            }
        }
    }

    fn gen_add_exp(&mut self, exp: &AddExp) -> String {
        match exp {
            AddExp::MulExp(exp) => self.gen_mul_exp(exp),
            AddExp::AddExp(lhs, op, rhs) => {
                let lhs = self.gen_add_exp(lhs);
                let rhs = self.gen_mul_exp(rhs);
                let op = match op {
                    AddOp::Plus => "add",
                    AddOp::Minus => "sub",
                };
                self.emit_binary(op, lhs, rhs)
            }
        }
    }

    fn gen_rel_exp(&mut self, exp: &RelExp) -> String {
        match exp {
            RelExp::AddExp(exp) => self.gen_add_exp(exp),
            RelExp::RelExp(lhs, op, rhs) => {
                let lhs = self.gen_rel_exp(lhs);
                let rhs = self.gen_add_exp(rhs);
                let op = match op {
                    RelOp::Less => "lt",
                    RelOp::Greater => "gt",
                    RelOp::LessEqual => "le",
                    RelOp::GreaterEqual => "ge",
                };
                self.emit_binary(op, lhs, rhs)
            }
        }
    }

    fn gen_eq_exp(&mut self, exp: &EqExp) -> String {
        match exp {
            EqExp::RelExp(exp) => self.gen_rel_exp(exp),
            EqExp::EqExp(lhs, op, rhs) => {
                let lhs = self.gen_eq_exp(lhs);
                let rhs = self.gen_rel_exp(rhs);
                let op = match op {
                    EqOp::Equal => "eq",
                    EqOp::NotEqual => "ne",
                };
                self.emit_binary(op, lhs, rhs)
            }
        }
    }

    // lhs && rhs 的语义是:
    // - lhs == 0:整个表达式直接为 0,不能计算 rhs。
    // - lhs != 0:才计算 rhs,并把 rhs 是否非 0 作为结果。
    //
    // 因此不能只生成 and lhs, rhs,而要用 br 形成两个执行路径:
    //   result = 0
    //   if lhs != 0 { result = (rhs != 0) }
    //   return result
    //
    // rhs 的指令在编译时仍会生成到 rhs 基本块中;短路指的是运行时可能跳过这个基本块。
    fn gen_l_and_exp(&mut self, exp: &LAndExp) -> String {
        match exp {
            LAndExp::EqExp(exp) => self.gen_eq_exp(exp),
            LAndExp::LAndExp(lhs, rhs) => {
                let lhs = self.gen_l_and_exp(lhs);
                let result_pointer = self.emit_alloc();

                // 默认结果为 false。lhs 为 0 时直接跳到 end,不执行 rhs 基本块。
                self.emit_store("0", &result_pointer);
                let rhs_label = self.new_label("land_rhs");
                let end_label = self.new_label("land_end");
                self.body
                    .push_str(&format!("  br {}, {}, {}\n", lhs, rhs_label, end_label));

                self.body.push_str(&format!("{}:\n", rhs_label));
                let rhs = self.gen_eq_exp(rhs);
                // SysY 逻辑表达式的结果必须是 0 或 1,不能把任意非零 rhs 直接作为结果。
                let rhs_bool = self.emit_binary("ne", rhs, "0".to_string());
                self.emit_store(&rhs_bool, &result_pointer);
                self.body.push_str(&format!("  jump {}\n", end_label));

                // 两条路径在这里汇合:可能保留默认 0,也可能使用 rhs_bool。
                self.body.push_str(&format!("{}:\n", end_label));
                self.emit_load(&result_pointer)
            }
        }
    }

    // lhs || rhs 和 && 正好相反:
    // - lhs != 0:整个表达式直接为 1,不能计算 rhs。
    // - lhs == 0:才计算 rhs,并把 rhs 是否非 0 作为结果。
    //
    // 对应伪代码:
    //   result = 1
    //   if lhs == 0 { result = (rhs != 0) }
    //   return result
    fn gen_l_or_exp(&mut self, exp: &LOrExp) -> String {
        match exp {
            LOrExp::LAndExp(exp) => self.gen_l_and_exp(exp),
            LOrExp::LOrExp(lhs, rhs) => {
                let lhs = self.gen_l_or_exp(lhs);
                let result_pointer = self.emit_alloc();

                // 默认结果为 true。lhs 非 0 时直接跳到 end,不执行 rhs 基本块。
                self.emit_store("1", &result_pointer);
                let rhs_label = self.new_label("lor_rhs");
                let end_label = self.new_label("lor_end");
                self.body
                    .push_str(&format!("  br {}, {}, {}\n", lhs, end_label, rhs_label));

                self.body.push_str(&format!("{}:\n", rhs_label));
                let rhs = self.gen_l_and_exp(rhs);
                // 和 && 一样,把 rhs 规范化为 0 或 1,再保存到共同的临时内存。
                let rhs_bool = self.emit_binary("ne", rhs, "0".to_string());
                self.emit_store(&rhs_bool, &result_pointer);
                self.body.push_str(&format!("  jump {}\n", end_label));

                self.body.push_str(&format!("{}:\n", end_label));
                self.emit_load(&result_pointer)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generate(expression: &str) -> String {
        let source = format!("int main() {{ return {}; }}", expression);
        let tokens = crate::lexer::lex(&source);
        let ast = crate::parser::parse(&tokens);
        let ir = gen_ir(&ast);

        // 用 Koopa 自己的解析器检查文本、类型和基本块结构。
        koopa::front::Driver::from(ir.as_str())
            .generate_program()
            .unwrap();
        ir
    }

    #[test]
    fn generates_number_unary_and_arithmetic_expressions() {
        assert!(generate("42").contains("ret 42"));
        generate("-!+(1 + 2 * 3)");
        generate("8 / 4 / 2");
    }

    #[test]
    fn generates_comparison_expressions() {
        generate("1 + 2 < 4 == 1");
        generate("3 >= 2 != 0");
    }

    #[test]
    fn generates_short_circuit_logical_expressions() {
        let ir = generate("0 && 1 || 2");
        assert!(ir.contains("land_rhs"));
        assert!(ir.contains("lor_rhs"));
        generate("1 || 0 && (2 || 3)");
    }
}
