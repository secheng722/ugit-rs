use sha1::{Digest, Sha1};
use std::fs::{self, File};
use std::io::Write;

const GIT_DIR: &str = ".ugit";
pub fn init() {
    fs::create_dir(GIT_DIR).unwrap();
    // fs::create_dir_all(format!("{}/objects", GIT_DIR)).unwrap();
}

pub fn hash_object(data: &[u8], type_: Option<&str>) -> String {
    let type_ = match type_ {
        Some(t) => t,
        None => "blob",
    };
    let obj = [type_.as_bytes(), b"\x00", data].concat();
    let oid = Sha1::digest(&obj);
    let oid_hex = format!("{:x}", oid);
    let path = format!("{}/objects/{}", GIT_DIR, oid_hex);

    fs::create_dir_all(format!("{}/objects", GIT_DIR)).unwrap();
    let mut file = File::create(path).unwrap();
    let _ = file.write_all(&obj);
    oid_hex
}

pub fn get_object(oid: &str, expected: Option<&str>) -> String {
    let path = format!("{}/objects/{}", GIT_DIR, oid);
    let data = fs::read(&path).unwrap();
    //从b"\x00"分割
    let mut parts = data.splitn(2, |&x| x == 0);
    let type_ = std::str::from_utf8(&parts.next().unwrap()).unwrap();
    match expected {
        Some(expected) => assert_eq!(expected, type_, "Expected {}, got {}", expected, type_),
        None => assert_eq!("blob", type_, "Expected blob, got {}", type_),
    }
    std::str::from_utf8(&parts.next().unwrap())
        .unwrap()
        .to_string()
}
