//! Stamps the build with its version: the crate version plus the 7-char
//! commit hash, `0.1.0 (2391c1d)`, or the version alone when git is not
//! there (a source tarball). Read as `env!("COFFEE_TIMER_VERSION")`.

use std::process::Command;

fn git(args: &[&str]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    let text = String::from_utf8(output.stdout).ok()?;
    let text = text.trim();
    (output.status.success() && !text.is_empty()).then(|| text.to_owned())
}

fn main() {
    let version = env!("CARGO_PKG_VERSION");
    let stamped = match git(&["rev-parse", "--short=7", "HEAD"]) {
        Some(hash) => format!("{version} ({hash})"),
        None => version.to_owned(),
    };
    println!("cargo:rustc-env=COFFEE_TIMER_VERSION={stamped}");

    // Re-stamp when HEAD moves: a checkout changes HEAD, a commit changes
    // a ref under refs/ (or packed-refs).
    if let Some(git_dir) = git(&["rev-parse", "--absolute-git-dir"]) {
        for path in ["HEAD", "refs", "packed-refs"] {
            // A watched path that does not exist would re-run this every build.
            let path = std::path::Path::new(&git_dir).join(path);
            if path.exists() {
                println!("cargo:rerun-if-changed={}", path.display());
            }
        }
    }
}
