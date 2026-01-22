use std::fs;
use std::io;
use std::path::Path;

pub fn bundle(dir: &Path, entry_file: &Path) -> io::Result<()> {
    // 1. Process the entry file to add top-level module declarations
    let mut entry_content = fs::read_to_string(entry_file)?;
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

    // 3. Inject subsidia shims (basic placeholders for now)
    inject_shims(dir)?;

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
            let mod_name = path.file_stem().unwrap().to_str().unwrap().replace(".", "_");
            mods.push(format!("pub mod {};", mod_name));
        }
    }

    if !mods.is_empty() {
        let mod_rs = dir.join("mod.rs");
        fs::write(mod_rs, mods.join("\n"))?;
    }
    Ok(())
}

fn inject_shims(dir: &Path) -> io::Result<()> {
    // Placeholder for JS globals shims
    // For now, just generate a dummy subsidia module if needed
    Ok(())
}
