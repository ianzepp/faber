use std::fs;
use std::io;
use std::path::Path;

pub fn bundle(dir: &Path, entry_file: &Path) -> io::Result<()> {
    // 0. Inject required shims before wiring module tree.
    inject_shims(dir)?;

    // 1. Process the entry file to add top-level module declarations
    let entry_content = fs::read_to_string(entry_file)?;
    let mut mods = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().unwrap().to_str().unwrap();
            let mod_name = name.replace(".", "_");
            mods.push(format!("pub mod {};", mod_name));
            // 2. Recursively generate mod.rs for subdirectories
            generate_mod_rs(&path)?;
        }
    }

    if !mods.is_empty() {
        let mut new_content = mods.join("\n");
        new_content.push_str("\n\n");
        new_content.push_str(&entry_content);
        fs::write(entry_file, new_content)?;
    }

    Ok(())
}

fn generate_mod_rs(dir: &Path) -> io::Result<()> {
    let mut mods = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = path.file_name().unwrap().to_str().unwrap();

        if path.is_dir() {
            let mod_name = name.replace(".", "_");
            mods.push(format!("pub mod {};", mod_name));
            generate_mod_rs(&path)?;
        } else if path.extension().map_or(false, |ext| ext == "rs") && name != "mod.rs" {
            let stem = path.file_stem().unwrap().to_str().unwrap();
            let mod_name = stem.replace(".", "_");
            if stem.contains('.') {
                // Rust module name can't contain '.', so wire using #[path].
                mods.push(format!("#[path = \"{}.rs\"]\npub mod {};", stem, mod_name));
            } else {
                mods.push(format!("pub mod {};", mod_name));
            }
        }
    }

    if !mods.is_empty() {
        let mod_rs = dir.join("mod.rs");
        fs::write(mod_rs, mods.join("\n"))?;
    }
    Ok(())
}

fn inject_shims(dir: &Path) -> io::Result<()> {
    // Minimal stdlib shims needed for compiling rivus via nanus-rs.
    // These are not meant to be a full target runtime.

    let norma_dir = dir.join("norma");
    let hal_dir = norma_dir.join("hal");
    fs::create_dir_all(&hal_dir)?;

    fs::write(norma_dir.join("mod.rs"), "pub mod hal;\n")?;

    fs::write(
        hal_dir.join("mod.rs"),
        "pub mod solum;\npub mod processus;\n",
    )?;

    fs::write(
        hal_dir.join("processus.rs"),
        "use std::env;\n\npub struct processus;\n\nimpl processus {\n    pub fn env(key: &str) -> Option<String> {\n        env::var(key).ok()\n    }\n}\n",
    )?;

    fs::write(
        hal_dir.join("solum.rs"),
        "use std::path::PathBuf;\n\npub struct solum;\n\nimpl solum {\n    pub fn domus() -> String {\n        std::env::var(\"HOME\").unwrap_or_else(|_| \"\".to_string())\n    }\n\n    pub fn iunge(parts: Vec<String>) -> String {\n        let mut p = PathBuf::new();\n        for part in parts {\n            p.push(part);\n        }\n        p.to_string_lossy().to_string()\n    }\n\n    pub fn exstat(path: String) -> bool {\n        std::path::Path::new(&path).exists()\n    }\n}\n",
    )?;

    Ok(())
}
