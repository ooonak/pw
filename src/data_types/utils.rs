
/// Read first line from file
pub fn read_line(path: &std::path::Path) -> Result<String, std::io::Error> {
    let mut contents = std::fs::read_to_string(path)?;
    
    if contents.ends_with('\n') {
        contents.pop();
    }

    Ok(contents)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_line_ok() {
        let boot_id = read_line(std::path::Path::new("/proc/sys/kernel/random/boot_id")).unwrap();
        assert_eq!(boot_id.len(), 36);
    }
}
