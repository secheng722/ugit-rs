use sha1::{Digest, Sha1};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

const GIT_DIR: &str = ".ugit";

pub fn init() -> Result<(), std::io::Error> {
    fs::create_dir(GIT_DIR)?;
    Ok(())
}

pub fn update_ref(_ref: &str, oid: &str) -> Result<(), std::io::Error> {
    let path = format!("{}/{}", GIT_DIR, _ref);
    let path = Path::new(&path);
    //检查是否存在该目录
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, oid)?;
    Ok(())
}
pub fn get_ref(_ref: &str) -> Result<String, std::io::Error> {
    //检查是否存在HEAD
    let path = format!("{}/{}", GIT_DIR, _ref);
    //这里需要判断吗
    // if Path::new(&path).is_file() { }
    return Ok(fs::read_to_string(path)?.trim().to_string());
}

pub fn hash_object(data: &[u8], type_: Option<&str>) -> Result<String, std::io::Error> {
    let type_ = type_.unwrap_or("blob");
    let obj = [type_.as_bytes(), b"\x00", data].concat();
    let oid = Sha1::digest(&obj);
    let oid_hex = format!("{:x}", oid);
    let path = format!("{}/objects/{}", GIT_DIR, oid_hex);

    fs::create_dir_all(format!("{}/objects", GIT_DIR))?;
    let mut file = File::create(&path)?;
    file.write_all(&obj)?;
    Ok(oid_hex)
}
pub fn get_object(oid: &str, expected: Option<&str>) -> Result<String, std::io::Error> {
    let path = format!("{}/objects/{}", GIT_DIR, oid);
    let data = fs::read(&path)?;
    let mut parts = data.splitn(2, |&x| x == b'\x00');
    let type_ = parts.next().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid object format")
    })?;
    let type_str = std::str::from_utf8(type_).map_err(|_| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid UTF-8 in type")
    })?;

    match expected {
        Some(expected) if expected != type_str => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Expected {}, got {}", expected, type_str),
            ))
        }
        None if "blob" != type_str => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Expected blob, got {}", type_str),
            ))
        }
        _ => {}
    }

    let content = parts.next().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid object format")
    })?;
    std::str::from_utf8(content)
        .map(|s| s.to_string())
        .map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid UTF-8 in content")
        })
}
