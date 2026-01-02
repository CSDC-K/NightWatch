fn main() {
    let config = slint_build::CompilerConfiguration::default()
        .with_include_paths(vec![
            // Sleek-ui klasörünün bulunduğu ana dizin
            std::path::PathBuf::from("node_modules"), 
        ]);

    slint_build::compile_with_config("ui/main.slint", config).expect("Slint build failed");
}