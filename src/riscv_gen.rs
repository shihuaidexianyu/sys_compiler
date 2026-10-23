use koopa::ir::ValueKind;

pub fn gen_riscv(s: &str) -> String {
    let mut riscv_code = String::new();
    let driver = koopa::front::Driver::from(s);
    let program = driver.generate_program().unwrap();
    for &func in program.func_layout() {
        let func_data = program.func(func);
        riscv_code.push_str(".text\n");
        // fix: koopa IR 中函数名是以 @ 开头的, 需要去掉
        let name = func_data.name().trim_start_matches('@');
        riscv_code.push_str(&format!(".globl {}\n", &name));
        riscv_code.push_str(&format!("{}:\n", &name));
        // 遍历基本块
        for (_, node) in func_data.layout().bbs() {
            for &inst in node.insts().keys() {
                let value_data = func_data.dfg().value(inst);
                match value_data.kind() {
                    ValueKind::Return(ret) => {
                        // 处理 ret 指令
                        if let Some(value) = ret.value() {
                            let value_data = func_data.dfg().value(value);
                            if let ValueKind::Integer(int) = value_data.kind() {
                                riscv_code.push_str(&format!("    li a0, {}\n", int.value()));
                            }
                        }
                        // 不管有没有返回值, ret 都要输出
                        riscv_code.push_str("    ret\n");
                    }
                    _ => unreachable!(), // 其他指令暂不处理
                }
            }
        }
    }
    riscv_code
}
