use crate::{base, data};

pub fn parse_args() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("uGit: not enough arguments");
        std::process::exit(1);
    }
    let command = &args[1];
    let name = args.get(2).cloned().unwrap_or("@".to_string());
    let oid = match base::get_oid(name) {
        Ok(oid) => oid,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };
    let tagname = args.get(3).cloned().unwrap_or_default();
    match command.as_str() {
        "init" => init(),
        "hash-object" => hash_object(&oid),
        "cat-file" => cat_file(&oid),
        "write-tree" => write_tree(&oid),
        "read-tree" => read_tree(&oid),
        "commit" => commit(&oid),
        "checkout" => checkout(&oid),
        "tag" => tag(&oid, &tagname),
        "k" => k(),
        "log" => log(&oid).unwrap(),
        _ => {
            eprintln!("uGit: invalid command {}", command);
            std::process::exit(1);
        }
    }
}

pub fn init() {
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
    match data::get_object(args, Some("commit")) {
        Ok(oid) => println!("{}", oid),
        Err(e) => eprintln!("{}", e),
    }
}

fn commit(args: &str) {
    let oid = base::commmit(args);
    println!("{}", oid.unwrap());
}
fn checkout(args: &str) {
    let res = base::checkout(args).unwrap();
}

fn tag(name: &str, oid: &str) {
    let _ = base::create_tag(name, oid);
}

fn k() {
    for (ref_name, oid) in data::iter_refs().unwrap() {
        println!("{} {}", oid, ref_name);
    }
}

fn log(args: &str) -> Result<(), Box<dyn std::error::Error>> {
    let commit = base::get_commit(&args)?;
    println!("{:?}", commit);
    Ok(())
}
