use crate::{base, data};

pub fn parse_args() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("uGit: not enough arguments");
        std::process::exit(1);
    }
    let command = &args[1];
    match command.as_str() {
        "init" => init(&args[1]),
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
        "write-tree" => write_tree(&args[2]),
        "read-tree" => read_tree(&args[2]),
        "commit" => commit(&args[2]),
        "log" => {
            if args.len() < 3 {
                log(None).unwrap()
            } else {
                log(Some(&args[2])).unwrap()
            }
        }
        _ => {
            eprintln!("uGit: invalid command {}", command);
            std::process::exit(1);
        }
    }
}

pub fn init(args: &str) {
    let _ = data::init();
}

fn write_tree(args: &str) {
    println!("{}", base::write_tree(Some(args)).unwrap());
}

fn read_tree(args: &str) {
    base::read_tree(Some(args));
}

fn hash_object(args: &str) {
    let data = std::fs::read(args).unwrap();
    match data::hash_object(&data, None) {
        Ok(oid) => println!("{}", oid),
        Err(e) => eprintln!("{}", e),
    };
}

fn cat_file(args: &str) {
    // println!("{}", data::get_object(args, None));
    match data::get_object(args, None) {
        Ok(oid) => println!("{}", oid),
        Err(e) => eprintln!("{}", e),
    }
}

fn commit(args: &str) {
    let oid = base::commmit(args);
    println!("{}", oid.unwrap());
}

fn log(args: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let binding = data::get_head()?;
    let oid = args.or(Some(&binding)).ok_or("oid is null")?;
    println!("oid: {}", oid);
    let commit = base::get_commit(&oid)?;
    println!("{:?}", commit);
    Ok(())
}
