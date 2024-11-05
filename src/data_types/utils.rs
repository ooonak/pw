use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    net::Ipv4Addr,
    path::Path,
    process::Command,
};

// Read all lines from file into vector.
pub fn read_lines(path: impl AsRef<Path>) -> io::Result<Vec<String>> {
    let file = File::open(path)?;
    let buf = BufReader::new(file);
    return buf.lines().map(|l| l).collect();
}

// Find first occurence of line in lines that begins with each element in elements.
// If elements is empty, all lines are returned.
pub fn parse_lines(lines: Vec<String>, mut elements: Vec<(&str, bool)>) -> Vec<String> {
    let mut info = vec![];

    for line in &lines {
        for element in &mut elements {
            if element.1 == false && line.starts_with(element.0) {
                element.1 = true;
                let words: Vec<&str> = line.split_whitespace().collect();
                info.push(words.join(" "));
            }
        }

        if elements.is_empty() {
            let words: Vec<&str> = line.split_whitespace().collect();
            info.push(words.join(" "));
        }
    }

    info
}

pub fn find_default_dev() -> Option<String> {
    // Simple manual approach instead of local-ip-address crate, sysfs and getifaddrs is not an option on Android.
    if let Ok(output) = Command::new("ip").arg("route").output() {
        let stdout = String::from_utf8(output.stdout).unwrap();
        for line in stdout.lines() {
            if line.starts_with("default") {
                let words: Vec<&str> = line.split_whitespace().collect();
                if words.len() >= 4 {
                    return Some(words[4].to_string());
                }
            }
        }
    }

    None
}

fn mac_from_string(mac_string: &str) -> u64 {
    let mac_string = mac_string.replace(":", "");
    let mac = u64::from_str_radix(&mac_string, 16);
    match mac {
        Ok(mac) => mac,
        Err(_e) => 0,
    }
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

pub fn find_iface_info(dev: &str) -> Option<(u64, u32)> {
    // Simple manual approach instead of local-ip-address crate, sysfs and getifaddrs is not an option on Android.
    if let Ok(output) = Command::new("ip")
        .arg("address")
        .arg("show")
        .arg(dev)
        .output()
    {
        let stdout = String::from_utf8(output.stdout).unwrap();

        let mut mac: u64 = 0;
        let mut ip4: u32 = 0;

        for line in stdout.lines() {
            if line.contains("link/ether") {
                let words: Vec<&str> = line.split_whitespace().collect();
                if words.len() >= 2 {
                    mac = mac_from_string(words[1]);
                }
            } else if line.contains("inet") {
                let words: Vec<&str> = line.split_whitespace().collect();
                if words.len() >= 2 {
                    ip4 = ip_from_string(words[1]);
                }
            }

            if mac != 0 && ip4 != 0 {
                return Some((mac, ip4));
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::{find_default_dev, *};

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
    fn find_default_dev_ok() {
        let _data = "default via 192.168.42.1 dev wlp3s0 proto dhcp src 192.168.42.114 metric 20 
default via 192.168.42.1 dev wlp3s0 proto dhcp src 192.168.42.122 metric 600 
172.17.0.0/16 dev docker0 proto kernel scope link src 172.17.0.1 linkdown 
192.168.42.0/24 dev wlp3s0 proto kernel scope link src 192.168.42.122 metric 600 
192.168.42.1 dev wlp3s0 proto dhcp scope link src 192.168.42.114 metric 20";

        let expected = "wlp3s0";
        let result = find_default_dev();
        assert!(result.is_some());
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn find_iface_info_ok() {
        let _data = "3: wlp3s0: <BROADCAST,MULTICAST,UP,LOWER_UP> mtu 1500 qdisc noqueue state UP group default qlen 1000
    link/ether b4:6b:fc:ed:d5:78 brd ff:ff:ff:ff:ff:ff
    inet 192.168.42.122/24 brd 192.168.42.255 scope global dynamic noprefixroute wlp3s0
       valid_lft 86389sec preferred_lft 86389sec
    inet 192.168.42.114/24 metric 20 brd 192.168.42.255 scope global secondary dynamic wlp3s0
       valid_lft 86399sec preferred_lft 86399sec
    inet6 fe80::a16:8647:5dac:4ed6/64 scope link noprefixroute 
       valid_lft forever preferred_lft forever";

        // We store MAC and IPv4 as their unsigned representation.
        let expected: (u64, u32) = (0xb46bfcedd578, 0xc0a82a7a);
        let result = find_iface_info("wlp3s0");
        assert!(result.is_some());
        assert_eq!(expected, result.unwrap());
    }
}
