use std::{ffi::OsString, fs::Metadata, path::Path};

use crate::data;

use std::error::Error;

pub fn write_tree(directory: Option<&str>) -> Result<String, Box<dyn Error>> {
    let directory = directory.unwrap_or(".");
    let mut entries = Vec::new();
    let read_dir = std::fs::read_dir(directory)?;
    for entry in read_dir {
        let entry = entry?;
        let path = entry.path();
        // 不跟随符号链接
        let metadata = std::fs::symlink_metadata(&path)?;
        let path_str = path.to_str().ok_or("路径转换为字符串失败")?;
        if is_ignored(path_str) {
            continue;
        }
        let (type_, oid, file_name) = process_entry(&path, &metadata)?;
        entries.push((type_, oid, file_name));
    }
    // 根据文件名排序 从小到大排序
    entries.sort_by(|a, b| a.2.cmp(&b.2));
    let tree = entries
        .iter()
        .map(|(type_, oid, name)| format!("{} {} {}\n", type_, oid, name.to_str().unwrap())) // 这里假设文件名的转换不会失败
        .collect::<String>();
    let oid = data::hash_object(tree.as_bytes(), Some("tree"))?;
    Ok(oid)
}

fn is_ignored(path: &str) -> bool {
    let ignore_file = [".ugit", "u-git", "target", ".vscode"];
    path.split("/").any(|item| ignore_file.contains(&item))
}

// 处理文件
fn process_entry(
    path: &Path,
    metadata: &Metadata,
) -> Result<(String, String, OsString), Box<dyn Error>> {
    let path_str = path.to_str().ok_or("路径转换为字符串失败")?;
    if metadata.is_file() {
        let type_ = "blob".to_string(); // 将类型改为String
        let data = std::fs::read(path)?;
        let oid = data::hash_object(&data, Some("blob"))?;
        Ok((
            type_,
            oid,
            path.file_name().ok_or("获取文件名失败")?.to_os_string(),
        ))
    } else if metadata.is_dir() {
        let type_ = "tree".to_string(); // 将类型改为String
        let oid = write_tree(Some(path_str))?;
        Ok((
            type_,
            oid,
            path.file_name().ok_or("获取文件名失败")?.to_os_string(),
        ))
    } else {
        Err("Unsupported file type".into())
    }
}

fn iter_tree_entries(oid: Option<&str>) -> impl Iterator<Item = (String, String, String)> {
    match oid {
        Some(oid) => {
            if let Ok(tree) = data::get_object(oid, Some("tree")) {
                return tree
                    .lines()
                    .map(|entry| {
                        let parts: Vec<&str> = entry.splitn(3, ' ').collect();
                        if parts.len() == 3 {
                            (
                                parts[0].to_string(),
                                parts[1].to_string(),
                                parts[2].to_string(),
                            )
                        } else {
                            panic!("Invalid tree entry");
                        }
                    })
                    .collect::<Vec<_>>()
                    .into_iter();
            }
            Vec::new().into_iter()
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
            if let Ok(data) = data::get_object(oid, Some("blob")) {
                std::fs::write(path, data).unwrap();
            }
        });
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

pub(crate) fn commmit(message: &str) -> Result<String, Box<dyn Error>> {
    let mut commit = format!("tree {}\n", write_tree(Some("./test"))?);
    if let Ok(oid) = data::get_ref("HEAD") {
        commit.push_str(&format!("parent {}\n", oid));
    }
    //message
    commit.push_str(&format!("\n{}\n", message));
    let oid = data::hash_object(commit.as_bytes(), Some("commit"))?;
    data::update_ref("HEAD",&oid)?;
    return Ok(oid);
}

pub(crate) fn checkout(oid: &str) -> Result<(), Box<dyn Error>> {
    let commit = get_commit(oid)?;
    read_tree(Some(&commit.tree));
    let _ = data::update_ref("HEAD",oid);
    Ok(())
}

pub(crate) fn create_tag(name:&str,oid:&str) {
    !todo!()
}


#[derive(Debug)]
pub(crate) struct Commit {
    tree: String,
    parent: Option<String>,
    message: String,
}

pub(crate) fn get_commit(oid: &str) -> Result<Commit, &'static str> {
    let commit = data::get_object(oid, Some("commit")).map_err(|_| "Failed to read commit")?;
    let parts = commit.split("\n").collect::<Vec<&str>>();
    let mut tree = None;
    let mut parent = None;
    let mut message = String::new();
    let mut parsing_headers = true;
    let mut lines_iter = parts.into_iter().peekable();
    while let Some(line) = lines_iter.next() {
        if line.is_empty() {
            parsing_headers = false;
            continue;
        }

        if parsing_headers {
            if let Some((key, value)) = line.split_once(" ") {
                match key {
                    "tree" => tree = Some(value.to_string()),
                    "parent" => parent = Some(value.to_string()),
                    _ => return Err("Unknown field"),
                }
            }
        } else {
            message.push_str(line);
            // 如果当前行不是最后一行，则添加换行符
            if lines_iter.peek().is_some() {
                message.push('\n');
            }
        }
    }
    Ok(Commit {
        tree: tree.ok_or("Missing tree")?,
        parent,
        message,
    })
}
