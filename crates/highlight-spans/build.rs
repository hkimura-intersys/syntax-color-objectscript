fn main() {
    compile_vendor_grammar(
        "vendor/tree-sitter-sql",
        "tree-sitter-sql-vendor",
        &["queries/highlights.scm"],
    );
}

fn compile_vendor_grammar(base_dir: &str, lib_name: &str, extra_paths: &[&str]) {
    let src_dir = std::path::Path::new(base_dir).join("src");
    let parser_path = src_dir.join("parser.c");
    let scanner_path = src_dir.join("scanner.c");
    let header_parser_path = src_dir.join("tree_sitter/parser.h");
    let header_alloc_path = src_dir.join("tree_sitter/alloc.h");
    let header_array_path = src_dir.join("tree_sitter/array.h");

    let mut c_config = cc::Build::new();
    c_config
        .std("c11")
        .include(&src_dir)
        .flag_if_supported("-Wno-unused-parameter")
        .flag_if_supported("-Wno-unused-but-set-variable")
        .flag_if_supported("-Wno-sign-compare")
        .flag_if_supported("-Wno-trigraphs")
        .file(&parser_path);

    if scanner_path.exists() {
        c_config.file(&scanner_path);
    }

    c_config.compile(lib_name);

    println!("cargo:rerun-if-changed={}", parser_path.display());
    println!("cargo:rerun-if-changed={}", header_parser_path.display());
    println!("cargo:rerun-if-changed={}", header_alloc_path.display());
    println!("cargo:rerun-if-changed={}", header_array_path.display());
    if scanner_path.exists() {
        println!("cargo:rerun-if-changed={}", scanner_path.display());
    }
    for rel_path in extra_paths {
        println!(
            "cargo:rerun-if-changed={}",
            std::path::Path::new(base_dir).join(rel_path).display()
        );
    }
}
