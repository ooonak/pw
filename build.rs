use std::io::Result;
fn main() -> Result<()> {
    prost_build::compile_protos(&["src/data_types/pw.messages.proto"], &["src/data_types"])?;
    Ok(())
}
