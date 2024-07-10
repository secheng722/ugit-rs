/*
cli.rs
*/

use crate::data;

pub fn parse_args() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("uGit: not enough arguments");
        std::process::exit(1);
    }
    let command = &args[1];
    match command.as_str() {
        "init" => init(),
        "hash-object" | "cat-file" => {
            if args.len() < 3 {
                eprintln!("uGit: not enough arguments");
                std::process::exit(1);
            }
            match command.as_str() {
                "hash-object" => hash_object(&args[2]),
                "cat-file" => cat_file(&args[2]),
                _ => unreachable!(), // 已经在上面的匹配中检查了命令
            }
        }
        _ => {
            eprintln!("uGit: invalid command {}", command);
            std::process::exit(1);
        }
    }
}

pub fn init() {
    data::init();
}

fn hash_object(args: &str) {
    let data = std::fs::read(args).unwrap();
    let oid = data::hash_object(&data, None);
    println!("{}", oid);
}

fn cat_file(args: &str) {
    println!("{}", data::get_object(args, None));
}
