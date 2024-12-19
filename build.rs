use std::{ffi::OsStr, fs, io, path::Path};

fn main() {
    println!("cargo:rustc-link-arg-bins=-Tlinkall.x");
    include_website().unwrap();
}

const STATIC_WEBSITE_DIR: &str = "website";
type Result<T, E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

fn include_website() -> Result<()> {
    println!("cargo::rerun-if-changed=build.rs");
    let file = Path::new(&std::env::var_os("OUT_DIR").ok_or("Missing env var OUT_DIR")?)
        .join("website_directory.part.rs");
    let base_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(STATIC_WEBSITE_DIR);
    let files = fs::read_dir(&base_dir)
        .map_err(|e| format!("Failed to read dir {}: {e}", base_dir.display()))?
        .filter_map(|entry| -> Option<Result<fs::DirEntry>> {
            fn inner(entry: io::Result<fs::DirEntry>) -> Result<(bool, fs::DirEntry)> {
                let entry = entry.map_err(|e| format!("Failed to read dir entry: {e}"))?;
                let ft = entry
                    .file_type()
                    .map_err(|e| format!("Failed to determine file type: {e}"))?;
                Ok((ft.is_file(), entry))
            }
            match inner(entry) {
                Ok((true, entry)) => Some(Ok(entry)),
                Ok((false, _)) => None,
                Err(e) => Some(Err(e)),
            }
        })
        .map(|entry| {
            let entry = entry?;
            let path = entry.path();
            let relative_path = path.strip_prefix(&base_dir)?;
            let mime_type = match relative_path
                .extension()
                .and_then(OsStr::to_str)
                .unwrap_or("")
            {
                "css" => "text/css",
                "html" => "text/html; charset=utf-8",
                "js" => "application/javascript; charset=utf-8",
                "svg" => "image/svg+xml",
                other => return Err(format!("Unknown file extension {other}").into()),
            };
            Ok(format!(
                r#"("{}", picoserve::response::File::with_content_type("{mime_type}", include_bytes!("{}")))"#,
                relative_path.display(),
                path.display()
            ))
        })
        .collect::<Result<Vec<_>>>()?
        .join(",\n");

    let code = format!(
        r#"picoserve::response::Directory {{ files: &const {{ [{files}] }}, ..Default::default() }}"#
    );
    fs::write(file, format_code(&code)?)?;
    Ok(())
}

fn format_code(expression: &str) -> Result<String> {
    let code = format!("fn foo() {{ {expression} }}");
    let formatted_code = rustfmt_wrapper::rustfmt(code)?;
    let formatted = formatted_code
        .strip_prefix("fn foo() {\n")
        .ok_or("Missing formatting prefix")?
        .strip_suffix("\n}\n")
        .ok_or("Missing formatting suffix")?;
    Ok(unindent::unindent(formatted))
}
