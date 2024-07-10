use crate::data;

pub fn write_tree(directory: Option<&str>) -> String {
    let directory = match directory {
        Some(d) => d,
        None => ".",
    };
    let mut entries = Vec::new();
    std::fs::read_dir(directory).unwrap().for_each(|entry| {
        let entry = entry.unwrap();
        let path = entry.path();
        // 不跟随符号链接
        let metadata = std::fs::symlink_metadata(&path).unwrap();
        let path_str = path.to_str().unwrap();
        if is_ignored(path_str) {
            return;
        }
        let (type_, oid, file_name) = process_entry(&path, &metadata);
        entries.push((type_, oid, file_name));
    });
    //根据文件名排序 从小到大排序
    entries.sort_by(|a, b| a.2.cmp(&b.2));
    let tree = entries
        .iter()
        .map(|(type_, oid, name)| format!("{} {} {}\n", type_, oid, name.to_str().unwrap()))
        .collect::<String>();
    let oid = data::hash_object(tree.as_bytes(), Some("tree"));
    return oid;
}

fn is_ignored(path: &str) -> bool {
    let ignore_file = [".ugit","u-git"];
    path.split("/").any(|item| ignore_file.contains(&item))
}

fn process_entry(path: &std::path::Path, metadata: &std::fs::Metadata) -> (String, String, std::ffi::OsString) {
    let path_str = path.to_str().unwrap();
    if metadata.is_file() {
        let type_ = "blob".to_string(); // 将类型改为String
        let data = std::fs::read(path).unwrap();
        let oid = data::hash_object(&data, Some("blob"));
        (type_, oid, path.file_name().unwrap().to_os_string())
    } else if metadata.is_dir() {
        let type_ = "tree".to_string(); // 将类型改为String
        let oid = write_tree(Some(path_str));
        (type_, oid, path.file_name().unwrap().to_os_string())
    } else {
        panic!("Unsupported file type");
    }
}