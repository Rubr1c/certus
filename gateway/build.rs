use std::path::Path;

fn main() {
    let out = Path::new(env!("CARGO_MANIFEST_DIR")).join("../dashboard/out");
    if !out.is_dir() {
        panic!(
            "missing {}; run `bun run build` in ../dashboard first",
            out.display()
        );
    }
}
