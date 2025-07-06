use std::process::Command;

fn main() {
    println!("cargo:rustc-rerun-if-changed=.git/HEAD");
    let output = Command::new("git").args(["rev-parse", "HEAD"]).output();
    let git_hash = match output {
        Ok(output) => String::from_utf8(output.stdout).unwrap_or("unknown".into()),
        Err(_) => "unknown".into(),
    };
    let output = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output();
    let git_hash_short = match output {
        Ok(output) => String::from_utf8(output.stdout).unwrap_or("unknown".into()),
        Err(_) => "unknown".into(),
    };
    println!("cargo:rustc-env=GIT_HASH={git_hash}");
    println!("cargo:rustc-env=GIT_HASH_SHORT={git_hash_short}");
}
