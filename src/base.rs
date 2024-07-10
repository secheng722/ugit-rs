pub fn write_tree(directory: Option<&str>) {
    let directory = match directory {
        Some(d) => d,
        None => ".",
    };
    std::fs::read_dir(directory).unwrap().for_each(|entry| {
        let entry = entry.unwrap();
        let path = entry.path();
        let path_str = path.to_str().unwrap();
        // 不跟随符号链接
        let metadata = std::fs::symlink_metadata(&path).unwrap();
        if metadata.is_file() {
            // TODO: write the file to object store
            println!("file: {}", path.to_str().unwrap());
        } else if metadata.is_dir() {
            write_tree(path.to_str());
        }
        // TODO: actually create the tree object
    });
}
