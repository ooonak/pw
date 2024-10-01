use std::fs;

fn boot_id() -> String {
    let contents = fs::read_to_string("/proc/sys/kernel/random/boot_id").unwrap_or_default();
    contents
}

fn main() {
    println!("boot_id: {}", boot_id());
}
