mod riscv_gen;
// grammar: mod 声明「这里有个模块」,编译器会去找同名的 .rs 文件
mod ast;
mod ir_gen;
mod lexer;
mod parser;
// grammar: use 把别处的名字引进来,下面就不用写全 std::env::args 了
use std::env;
use std::fs;

fn main() {
    // 用法: sysy-compiler 输入文件.sy
    // grammar: collect() 把迭代器收成集合,收进什么类型由左边标注决定,所以 Vec<String> 不能省
    let args: Vec<String> = env::args().collect();
    // 第一个参数是程序名,第二个参数是输入文件名
    // grammar: &args[1] 取下标再借用,得到 &String,会自动当 &str 用
    // tip: 这行在 args.len() 检查之前,不带参数运行会先在这里下标越界 panic
    // 先查个数,再取元素 —— 顺序不能反,反了就是越界 panic
    if args.len() != 5 {
        eprintln!("用法: sysy-compiler -koopa/-riscv 输入文件.sy -o 输出文件.s");
        std::process::exit(1);
    }
    let input = &args[2]; // 输入文件路径
    let output = &args[4]; // 输出文件路径; args[3] 是 "-o",不用管
    // tip: 切分参数是由shell 完成的，-只是用来表示选项；他实际上和输入输出文件名是同级的参数，-o 只是一个标记，告诉程序下一个参数是输出文件名
    let input = fs::read_to_string(input).expect("无法读取输入文件");
    // 先验证 lexer: 把切出来的 token 全打印出来
    let tokens = lexer::lex(&input);
    // grammar: &tokens 是借用着遍历,循环结束 tokens 还活着(下面 parse 还要用)
    // for t in &tokens {
    //     // grammar: {:?} 走的是 derive 出来的 Debug,不是给人看的 Display
    //     println!("{:?}", t);
    // }

    // 再验证 parser: 打印解析出的 AST
    let ast = parser::parse(&tokens);
    // grammar: # 让 Debug 输出自动换行缩进,结构深时更好看
    // println!("{:#?}", ast);

    // 生成 IR
    let ir = ir_gen::gen_ir(&ast);
    // println!("生成的 IR:\n{}", ir);
    let mode = &args[1]; // "-koopa"
    match mode.as_str() {
        "-koopa" => {
            fs::write(output, ir).expect("无法写入输出文件");
        }
        "-riscv" => {
            // 生成 RISC-V 汇编
            let riscv = riscv_gen::gen_riscv(&ir);
            println!("生成的 RISC-V 汇编:\n{}", riscv);
            fs::write(output, riscv).expect("无法写入输出文件");
        }
        _ => {
            eprintln!("不支持的模式: {}", mode);
            std::process::exit(1);
        }
    }
}
