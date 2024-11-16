use anyhow::Result;
use vergen_git2::{ BuildBuilder, Git2Builder, Emitter };

fn main() -> Result<()> {
    let build_timestamp = BuildBuilder::default().build_timestamp(true).build()?;
    let commit_hash = Git2Builder::default().sha(true).build()?;
    let git_dirty = Git2Builder::default().dirty(true).build()?;

    Emitter::default()
        .add_instructions(&build_timestamp)?
        .add_instructions(&commit_hash)?
        .add_instructions(&git_dirty)?
        .emit()
}
