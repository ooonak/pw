use std::{fs::File, io::{self, BufRead, BufReader}, path::Path};

/// Read all lines from file into vector.
pub fn read_lines(path: impl AsRef<Path>) -> io::Result<Vec<String>> {
    let file = File::open(path)?;
    let buf = BufReader::new(file);
    return buf.lines()
        .map(|l| l)
        .collect()
}

/// Find first occurence of line in lines that begins with each element in elements.
/// If elements is empty, all lines are returned.
pub fn parse_lines(lines: Vec<String>, mut elements: Vec<(&str, bool)>) -> Vec<String> {
    let mut info = vec![];

    for line in &lines {
        for element in & mut elements {
            if element.1 == false && line.starts_with(element.0) {
                element.1 = true;
                let words: Vec<&str> = line.split_whitespace().collect(); 
                info.push(words.join(" "));

            }
        }

        if elements.is_empty()
        {
            let words: Vec<&str> = line.split_whitespace().collect(); 
            info.push(words.join(" ")); 
        }
    }

    info
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_line_ok() {
        let lines = read_lines(std::path::Path::new("/proc/sys/kernel/random/boot_id")).expect("Failed to file");
        assert_eq!(lines.len(), 1); 
        assert_eq!(lines[0].len(), 36);
    }

    #[test]
    fn read_line_missing_file() {
        let result = read_lines("/tmp/dontexists/boot_id").map_err(|e| e.kind());
        let expected = Err(std::io::ErrorKind::NotFound);
        assert_eq!(result, expected);
    }

    #[test]
    fn parse_lines_no_lines() {

    }

    #[test]
    fn parse_lines_one_line() {

    }

    #[test]
    fn parse_lines_no_elements() {
        
    }

    #[test]
    fn parse_lines_multiple_elements() {
        
    }
}
