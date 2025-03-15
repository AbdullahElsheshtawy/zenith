use std::process::Command;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=shaders");

    const SHADERS_PATH: &str = "./shaders";
    for entry in std::fs::read_dir(SHADERS_PATH)? {
        let path = entry?.path();

        match path.extension() {
            Some(ext) => match ext.to_str().unwrap() {
                "spv" => continue,
                _ => {}
            },
            None => continue,
        }
        let path_string = String::from(path.to_str().unwrap());
        let output_path = format!("{path_string}.spv");

        println!("cargo:rerun-if-changed={path_string}");
        let output = Command::new("glslangValidator")
            .args(["-V", &path_string, "-o", &output_path])
            .output()?;

        if !output.status.success() {
            return Err(format!("{}", String::from_utf8(output.stdout)?).into());
        }
    }

    Ok(())
}
