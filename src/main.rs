mod ast;
mod lexer;
mod parser;
use std::env;
use std::fs;

fn main() {
    // 用法: sysy-compiler 输入文件.sy
    let args: Vec<String> = env::args().collect();
    // 第一个参数是程序名,第二个参数是输入文件名
    let input = fs::read_to_string(&args[1]).expect("读源文件失败");
    if args.len() != 2 {
        eprintln!("用法: sysy-compiler 输入文件.sy");
        std::process::exit(1);
    }

    // 先验证 lexer: 把切出来的 token 全打印出来
    let tokens = lexer::lex(&input);
    for t in &tokens {
        println!("{:?}", t);
    }

    // 再验证 parser: 打印解析出的 AST
    let ast = parser::parse(&tokens);
    println!("{:#?}", ast);
}
