use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

pub async fn read_file(path: PathBuf) -> Result<String, std::io::Error> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let file_path = manifest_dir.join("../../").join(path);
    println!("Reading file: {:?}", file_path);

    let mut file = File::open(file_path).await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;
    Ok(contents)
}
