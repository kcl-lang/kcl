use std::error::Error;
use std::process::Command;
use vergen::{Emitter, RustcBuilder};

fn main() -> Result<(), Box<dyn Error>> {
    // Get git sha
    let git_sha = if let Ok(output) = Command::new("git").args(["rev-parse", "HEAD"]).output() {
        if output.status.success() {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        } else {
            "unknown".to_string()
        }
    } else {
        "unknown".to_string()
    };

    // Emit git sha
    println!("cargo:rustc-env=VERGEN_GIT_SHA={git_sha}");

    // Emit rustc info
    let rustc = RustcBuilder::all_rustc()?;
    Emitter::default().add_instructions(&rustc)?.emit()?;

    Ok(())
}
