use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    net::Ipv4Addr,
    path::Path,
    process::Command,
    str::FromStr,
};

// Read all lines from file into vector.
pub fn read_lines(path: impl AsRef<Path>) -> io::Result<Vec<String>> {
    let file = File::open(path)?;
    let buf = BufReader::new(file);
    buf.lines().collect()
}

// Find first occurence of line in lines that begins with each element in elements.
// If elements is empty, all lines are returned.
// If drop_key is true everything before and including first occurence of ": " will be dropped.
pub fn parse_lines(
    lines: Vec<String>,
    mut elements: Vec<(&str, bool)>,
    drop_key: bool,
) -> Vec<String> {
    let mut info = vec![];

    for line in &lines {
        for element in &mut elements {
            if !element.1 && line.starts_with(element.0) {
                element.1 = true;
                parse_line(line, drop_key, &mut info);
            }
        }

        if elements.is_empty() {
            parse_line(line, drop_key, &mut info);
        }
    }

    info
}

pub fn parse_lines_no_separator(lines: Vec<String>, elements: Vec<(&str, bool)>) -> Vec<String> {
    parse_lines(lines, elements, false)
}

fn parse_line(line: &String, drop_key: bool, info: &mut Vec<String>) {
    let words: Vec<&str> = line.split_whitespace().collect();
    let mut line = words.join(" ");

    if drop_key {
        remove_key(&mut line);
    }

    info.push(line);
}

fn remove_key(line: &mut String) {
    if let Some((_key, value)) = line.split_once(": ") {
        *line = value.to_owned();
    }
}

pub fn parse_number<T: FromStr>(input: &str) -> Result<T, <T as FromStr>::Err> {
    let i = input.find(|c: char| !c.is_numeric()).unwrap_or(input.len());
    input[..i].parse::<T>()
}

pub fn parse_number_no_separator<T: FromStr>(input: &str) -> Option<T> {
    let words: Vec<&str> = input.split_whitespace().collect();
    if words.len() >= 2 {
        return parse_number(words[1]).ok();
    }

    None
}

fn parse_ip_route_info(input: &str) -> Vec<String> {
    // We need room for three strings: name, protocol & gateway.
    let mut output = vec![ "".to_string(), "".to_string(), "".to_string() ];

    for line in input.lines() {
        if output[0].is_empty() && input.contains("default") {
            let words: Vec<&str> = input.trim().split_whitespace().collect();
            println!("{:?}", words);
            if words.len() >= 7 {
                // words[4] is name
                output[0] = words[4].to_string();
                // words[6] is proto
                output[1] = words[6].to_string();
                // words[2] is gateway
                output[2] = words[2].to_string();
            }
        }
    }

    output
}

fn parse_ip_address_info(input: &str) -> Vec<String> {
    // We need room for three strings: ip/mask, broadcast & mac.
    let mut output = vec![ "".to_string(), "".to_string(), "".to_string() ];

    for line in input.lines() {
        if output[2].is_empty() && line.contains("link/ether") {
            let words: Vec<&str> = line.split_whitespace().collect();
            if words.len() >= 2 {
                // words[1] is mac
                output[2] = words[1].to_string();
            }
        } else if output[0].is_empty() && line.contains("inet") {
            let words: Vec<&str> = line.split_whitespace().collect();
            if words.len() >= 4 {
                // words[1] is ipv4/subnet_mask
                output[0] = words[1].to_string();
                // words[3] is broadcast
                output[1] = words[3].to_string();
            }
        }
    }

    output
}

pub fn find_default_dev() -> Option<Vec<String>> {
    // Simple manual approach instead of local-ip-address crate, sysfs and getifaddrs is not an option on Android.
    if let Ok(output) = Command::new("ip").arg("route").output() {
        let stdout = String::from_utf8(output.stdout).unwrap();
                return Some(parse_ip_route_info(&stdout));
    }

    None
}

fn mac_from_string(mac_string: &str) -> u64 {
    let mac_string = mac_string.replace(":", "");
    let mac = u64::from_str_radix(&mac_string, 16);
    mac.unwrap_or_default()
}

fn ip_from_string(ip_string: &str) -> u32 {
    match ip_string.split_once("/") {
        Some(ip_string) => {
            let ipv4 = ip_string.0.parse::<Ipv4Addr>();
            match ipv4 {
                Ok(ipv4) => ipv4.to_bits(),
                _ => 0,
            }
        }
        _ => 0,
    }
}

pub fn find_iface_info(dev: &str) -> Option<Vec<String>> {
    // Simple manual approach instead of local-ip-address crate, sysfs and getifaddrs is not an option on Android.
    if let Ok(output) = Command::new("ip")
        .arg("address")
        .arg("show")
        .arg(dev)
        .output()
    {
        let stdout = String::from_utf8(output.stdout).unwrap();
        return Some(parse_ip_address_info(&stdout));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_line_ok() {
        let lines = read_lines(std::path::Path::new("/proc/sys/kernel/random/boot_id"))
            .expect("Failed to file");
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
    fn parse_ip_route_info_ok() {
        let input = "default via 192.168.42.1 dev eth0 proto dhcp src 192.168.42.105 metric 100\n192.168.42.0/24 dev eth0 proto kernel scope link src 192.168.42.105 metric 100";
        let expected = vec![ "eth0".to_string(), "dhcp".to_string(), "192.168.42.1".to_string() ];
        assert_eq!(parse_ip_route_info(input), expected);
    }

    #[test]
    fn parse_ip_address_info_ok() {
        let input = "2: eth0: <BROADCAST,MULTICAST,UP,LOWER_UP> mtu 1500 qdisc pfifo_fast state UP group default qlen 1000\n    link/ether b8:27:eb:10:c6:85 brd ff:ff:ff:ff:ff:ff\n    inet 192.168.42.105/24 brd 192.168.42.255 scope global dynamic noprefixroute eth0\n       valid_lft 69343sec preferred_lft 69343sec\n    inet6 fe80::bbdd:4dac:bd7c:d839/64 scope link noprefixroute\n       valid_lft forever preferred_lft forever";
        let expected = vec![ "192.168.42.105/24".to_string(), "192.168.42.255".to_string(), "b8:27:eb:10:c6:85".to_string() ];
        assert_eq!(parse_ip_address_info(input), expected);
    }
}
