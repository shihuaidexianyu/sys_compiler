use koopa::ir::{BasicBlock, Function, Program, ValueKind};

pub fn gen_riscv(s: &str) -> String {
    let mut riscv_code = String::new();
    let driver = koopa::front::Driver::from(s);
    let program = driver.generate_program().unwrap();
    for &func in program.func_layout() {
        let func_data = program.func(func);
        riscv_code.push_str(".text\n");
        riscv_code.push_str(&format!(".globl {}\n", &func_data.name()));
        riscv_code.push_str(&format!("{}:\n", &func_data.name()));
        // 遍历基本块
        for (&bb, node) in func_data.layout().bbs() {
            for &inst in node.insts().keys() {
                let value_data = func_data.dfg().value(inst);
                match value_data.kind() {
                    ValueKind::Integer(int) => {
                        // 处理 integer 指令
                        riscv_code.push_str(&format!("    li a0, {}\n", int.value()));
                    }
                    ValueKind::Return(ret) => {
                        // 处理 ret 指令
                        riscv_code.push_str("    ret\n");
                    }
                    // 其他种类暂时遇不到
                    _ => unreachable!(),
                }
            }
        }
    }
    riscv_code
}
