use std::error::Error;
use vergen_gitcl::*;

fn main() -> Result<(), Box<dyn Error>> {
    let gitcl = GitclBuilder::default().sha(false).build()?;
    let rustc = RustcBuilder::all_rustc()?;

    Emitter::default()
        .add_instructions(&gitcl)?
        .add_instructions(&rustc)?
        .emit()?;

    Ok(())
}
