use koopa::ir::{BasicBlock, BinaryOp, FunctionData, Value, ValueKind};
use std::collections::HashMap;

// 入口:先用 koopa crate 解析文本 IR,再逐个函数生成汇编。
pub fn gen_riscv(s: &str) -> String {
    let driver = koopa::front::Driver::from(s);
    let program = driver.generate_program().unwrap();
    let mut riscv_code = String::new();

    for &func in program.func_layout() {
        let func_data = program.func(func);
        let mut generator = RiscvGen::new();
        generator.prepare(func_data);
        generator.gen_func(func_data);
        riscv_code.push_str(&generator.code);
    }

    riscv_code
}

// 最简单的临时值管理方法:每个 Koopa 临时值都对应一个栈槽。
// 计算时使用 t0/t1,计算完成后立刻用 sw 保存,因此不用担心寄存器不够用。
struct RiscvGen {
    code: String,
    // Value 是 Koopa 给指令结果的编号;i32 是它相对于 sp 的栈偏移。
    offsets: HashMap<Value, i32>,
    // Koopa 基本块编号 -> RISC-V 汇编标签。
    labels: HashMap<BasicBlock, String>,
    frame_size: i32,
}

impl RiscvGen {
    fn new() -> Self {
        Self {
            code: String::new(),
            offsets: HashMap::new(),
            labels: HashMap::new(),
            frame_size: 0,
        }
    }

    // 第一遍:分配栈槽和基本块标签,此时还不生成具体指令。
    fn prepare(&mut self, func: &FunctionData) {
        let mut offset = 0;
        let func_name = func.name().trim_start_matches('@');

        for (bb, node) in func.layout().bbs() {
            let label = format!(".L{}_{}", func_name, self.labels.len());
            self.labels.insert(*bb, label);

            for &inst in node.insts().keys() {
                let value_data = func.dfg().value(inst);
                match value_data.kind() {
                    // Binary/Load 需要保存计算结果;Alloc 需要一块 i32 内存。
                    ValueKind::Binary(_) | ValueKind::Load(_) | ValueKind::Alloc(_) => {
                        self.offsets.insert(inst, offset);
                        offset += 4;
                    }
                    _ => {}
                }
            }
        }

        // RISC-V 栈指针保持 16 字节对齐。
        self.frame_size = (offset + 15) / 16 * 16;
    }

    fn gen_func(&mut self, func: &FunctionData) {
        let name = func.name().trim_start_matches('@');
        self.code.push_str(".text\n");
        self.code.push_str(&format!(".globl {}\n", name));
        self.code.push_str(&format!("{}:\n", name));

        if self.frame_size > 0 {
            self.code
                .push_str(&format!("    addi sp, sp, -{}\n", self.frame_size));
        }

        for (bb, node) in func.layout().bbs() {
            self.code.push_str(&format!("{}:\n", self.labels[bb]));
            for &inst in node.insts().keys() {
                self.gen_instruction(func, inst);
            }
        }
    }

    fn gen_instruction(&mut self, func: &FunctionData, inst: Value) {
        let value_data = func.dfg().value(inst);
        match value_data.kind() {
            ValueKind::Binary(binary) => {
                self.load_value(func, binary.lhs(), "t0");
                self.load_value(func, binary.rhs(), "t1");

                match binary.op() {
                    BinaryOp::Add => self.emit("add t0, t0, t1"),
                    BinaryOp::Sub => self.emit("sub t0, t0, t1"),
                    BinaryOp::Mul => self.emit("mul t0, t0, t1"),
                    BinaryOp::Div => self.emit("div t0, t0, t1"),
                    BinaryOp::Mod => self.emit("rem t0, t0, t1"),
                    BinaryOp::Lt => self.emit("slt t0, t0, t1"),
                    BinaryOp::Gt => self.emit("slt t0, t1, t0"),
                    BinaryOp::Le => {
                        // lhs <= rhs 等价于 !(rhs < lhs)。
                        self.emit("slt t0, t1, t0");
                        self.emit("xori t0, t0, 1");
                    }
                    BinaryOp::Ge => {
                        // lhs >= rhs 等价于 !(lhs < rhs)。
                        self.emit("slt t0, t0, t1");
                        self.emit("xori t0, t0, 1");
                    }
                    BinaryOp::Eq => {
                        // 两数异或为 0 时相等;sltiu ..., 1 实现结果为 0/1。
                        self.emit("xor t0, t0, t1");
                        self.emit("sltiu t0, t0, 1");
                    }
                    BinaryOp::NotEq => {
                        // 无符号比较 0 < t0:t0 非 0 时结果就是 1。
                        self.emit("xor t0, t0, t1");
                        self.emit("sltu t0, x0, t0");
                    }
                    BinaryOp::And => self.emit("and t0, t0, t1"),
                    BinaryOp::Or => self.emit("or t0, t0, t1"),
                    BinaryOp::Xor => self.emit("xor t0, t0, t1"),
                    BinaryOp::Shl => self.emit("sll t0, t0, t1"),
                    BinaryOp::Shr => self.emit("srl t0, t0, t1"),
                    BinaryOp::Sar => self.emit("sra t0, t0, t1"),
                }
                self.save_result(inst);
            }

            ValueKind::Alloc(_) => {
                // prepare 已经预留栈空间,运行时不需要额外指令。
            }

            ValueKind::Store(store) => {
                self.load_value(func, store.value(), "t0");
                self.emit_address(store.dest(), "t1");
                self.emit("sw t0, 0(t1)");
            }

            ValueKind::Load(load) => {
                self.emit_address(load.src(), "t0");
                self.emit("lw t0, 0(t0)");
                self.save_result(inst);
            }

            ValueKind::Branch(branch) => {
                self.load_value(func, branch.cond(), "t0");
                // 条件非 0 跳 true,否则按顺序执行下一条 j,跳 false。
                let true_label = &self.labels[&branch.true_bb()];
                let false_label = &self.labels[&branch.false_bb()];
                self.code
                    .push_str(&format!("    bnez t0, {}\n", true_label));
                self.code.push_str(&format!("    j {}\n", false_label));
            }

            ValueKind::Jump(jump) => {
                let target = &self.labels[&jump.target()];
                self.code.push_str(&format!("    j {}\n", target));
            }

            ValueKind::Return(ret) => {
                if let Some(value) = ret.value() {
                    self.load_value(func, value, "a0");
                }
                if self.frame_size > 0 {
                    self.code
                        .push_str(&format!("    addi sp, sp, {}\n", self.frame_size));
                }
                self.emit("ret");
            }

            _ => unreachable!("暂不支持的 Koopa 指令"),
        }
    }

    // 把 Koopa Value 加载到指定寄存器。
    fn load_value(&mut self, func: &FunctionData, value: Value, reg: &str) {
        let value_data = func.dfg().value(value);
        match value_data.kind() {
            ValueKind::Integer(int) => {
                self.code
                    .push_str(&format!("    li {}, {}\n", reg, int.value()));
            }
            _ => {
                let offset = self.offsets[&value];
                self.code
                    .push_str(&format!("    lw {}, {}(sp)\n", reg, offset));
            }
        }
    }

    // Alloc 的结果是指针;这里让 reg 指向它对应的栈槽。
    fn emit_address(&mut self, value: Value, reg: &str) {
        let offset = self.offsets[&value];
        self.code
            .push_str(&format!("    addi {}, sp, {}\n", reg, offset));
    }

    // t0 中的计算结果保存到当前指令对应的栈槽。
    fn save_result(&mut self, value: Value) {
        let offset = self.offsets[&value];
        self.code.push_str(&format!("    sw t0, {}(sp)\n", offset));
    }

    fn emit(&mut self, instruction: &str) {
        self.code.push_str("    ");
        self.code.push_str(instruction);
        self.code.push('\n');
    }
}
