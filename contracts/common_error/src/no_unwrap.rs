#![cfg(test)]

#[test]
fn test_no_unwrap_in_production_code() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("contracts");

    let mut found = Vec::new();
    for entry in walkdir::WalkDir::new(&src_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file() && e.path().extension().map_or(false, |ext| ext == "rs"))
    {
        let content = std::fs::read_to_string(entry.path()).unwrap();
        for (i, line) in content.lines().enumerate() {
            if line.contains(".unwrap()") && !line.trim_start().starts_with("//") && !line.trim_start().starts_with("#[") {
                found.push(format!("{}:{}: {}", entry.path().display(), i + 1, line.trim()));
            }
        }
    }

    if !found.is_empty() {
        println!("Found unwrap() calls:\n{}", found.join("\n"));
    }
}