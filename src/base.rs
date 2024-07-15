use std::{fmt::format, fs::read};

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
    let ignore_file = [".ugit", "u-git", "target", ".vscode"];
    path.split("/").any(|item| ignore_file.contains(&item))
}

//处理文件
fn process_entry(
    path: &std::path::Path,
    metadata: &std::fs::Metadata,
) -> (String, String, std::ffi::OsString) {
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

fn iter_tree_entries(oid: Option<&str>) -> impl Iterator<Item = (String, String, String)> {
    match oid {
        Some(oid) => {
            let tree = data::get_object(oid, Some("tree"));
            tree.lines()
                .map(|entry| {
                    let parts: Vec<&str> = entry.splitn(3, ' ').collect();
                    if parts.len() == 3 {
                        (
                            parts[0].to_string(),
                            parts[1].to_string(),
                            parts[2].to_string(),
                        )
                    } else {
                        panic!("Invalid tree entry format")
                    }
                })
                .collect::<Vec<_>>()
                .into_iter()
        }
        None => Vec::new().into_iter(),
    }
}

pub(crate) fn get_tree(
    oid: Option<&str>,
    base_path: Option<&str>,
) -> std::collections::HashMap<String, String> {
    let base_path = match base_path {
        Some(p) => p,
        None => "",
    };
    let mut map = std::collections::HashMap::new();
    iter_tree_entries(oid).for_each(|(type_, oid, name)| {
        assert_eq!(name.contains('/'), false);
        assert_eq!(name != ".." && name != ".", true);
        let path = format!("{}/{}", base_path, name);
        if type_ == "blob" {
            map.insert(path, oid);
        } else if type_ == "tree" {
            let sub_tree = get_tree(Some(&oid), Some(&format!("{}/", path)));
            map.extend(sub_tree);
        } else {
            panic!("Invalid tree entry")
        }
    });
    return map;
}

pub(crate) fn read_tree(tree_oid: Option<&str>) {
    empty_current_directory();
    get_tree(tree_oid, Some("./test"))
        .iter()
        .for_each(|(path, oid)| {
            let data = data::get_object(oid, Some("blob"));
            std::fs::write(path, data).unwrap();
        })
}

fn empty_current_directory() {
    let entries = std::fs::read_dir("./test").unwrap();
    entries.for_each(|entry| {
        let entry = entry.unwrap();
        let path = entry.path();
        let path_str = path.to_str().unwrap();
        if is_ignored(path_str) {
            return;
        }
        if path.is_dir() {
            std::fs::remove_dir_all(path).unwrap();
        } else {
            std::fs::remove_file(path).unwrap();
        }
    });
}

pub(crate) fn commmit(message: &str) -> String {
    let commit = format!("tree {}\n{}", write_tree(Some("./test")), message);
    let oid = data::hash_object(commit.as_bytes(), Some("commit"));
    data::set_head(&oid);
    return oid;
}