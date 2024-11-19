use anyhow::Result;
use vergen_gix::{BuildBuilder, Emitter, GixBuilder};

fn main() -> Result<()> {
    let build_timestamp = BuildBuilder::default().build_timestamp(true).build()?;
    let commit_hash = GixBuilder::default().sha(true).build()?;
    let git_dirty = GixBuilder::default().dirty(true).build()?;

    Emitter::default()
        .add_instructions(&build_timestamp)?
        .add_instructions(&commit_hash)?
        .add_instructions(&git_dirty)?
        .emit()
}
