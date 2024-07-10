use crate::data;

pub fn write_tree(directory: Option<&str>) {
    let directory = match directory {
        Some(d) => d,
        None => ".",
    };
    std::fs::read_dir(directory).unwrap().for_each(|entry| {
        let entry = entry.unwrap();
        let path = entry.path();
        // 不跟随符号链接
        let metadata = std::fs::symlink_metadata(&path).unwrap();
        let path_str = path.to_str().unwrap();
        if is_ignored(path_str) {
            return;
        }
        if metadata.is_file() {
            let data = std::fs::read(path).unwrap();
            let oid: String = data::hash_object(&data, None);
            println!("{}", oid);
        } else if metadata.is_dir() {
            write_tree(Some(path_str));
        }
        // TODO: actually create the tree object
    });
}

fn is_ignored(path: &str) -> bool {
    path.split("/").any(|item| item.eq(".ugit"))
}
