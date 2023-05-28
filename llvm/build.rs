use std::path::{Path, PathBuf};

fn read_dir_recursive(path: impl AsRef<Path>) -> std::io::Result<Vec<PathBuf>> {
    let mut paths = vec![];

    for entry in std::fs::read_dir(path)?.into_iter() {
        let entry = entry?;

        let path = entry.path();
        let metadata = entry.metadata()?;

        if metadata.is_dir() {
            let mut list = read_dir_recursive(path.as_path())?;
            paths.append(&mut list);
        } else if metadata.is_file() {
            let extension = path.extension().and_then(|extension| extension.to_str());

            if let Some("cpp") = extension {
                paths.push(path);
            } else {
                continue;
            }
        }
    }

    Ok(paths)
}

fn main() {
    let mut builder = cc::Build::new();
    let binding = builder.cpp(true).include("include");

    let list = read_dir_recursive("lib").unwrap();
    for path in list {
        println!("cargo:rerun-if-changed={}", path.to_str().unwrap());
        binding.file(path);
    }

    binding.compile("llvm");
}
