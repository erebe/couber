use std::fs::File;
use std::process::Command;
use crate::database::Video;

pub fn fetch_coub(
    coub_name: &str,
    output_path: &std::path::Path,
    scripts_path: &std::path::Path,
) -> eyre::Result<Video> {
    let mut cmd = Command::new("./coub.sh");
    cmd.args([coub_name])
        .arg(output_path)
        .current_dir(scripts_path);
    info!("Coub fetched: {:?}", cmd);
    cmd.output()?;
    let video_file = File::open(
        output_path
            .join(coub_name)
            .join(format!("{}.js", coub_name)),
    )?;
    let video: Video = serde_json::from_reader(video_file)?;
    Ok(video)
}

pub fn fetch_video(
    video_url: &str,
    output_path: &std::path::Path,
    scripts_path: &std::path::Path,
) -> eyre::Result<Video> {
    fn calculate_hash(t: &str) -> String {
        let hash = sha2::Sha256::digest(t.as_bytes());
        let mut hash = hex::encode(hash);
        hash.truncate(10);
        hash
    }
    let video_name = calculate_hash(video_url);
    let mut cmd = Command::new("./generic_vids.sh");
    cmd.arg(video_url)
        .arg(output_path)
        .arg(&video_name)
        .current_dir(scripts_path);
    info!("Video fetched: {:?}", cmd);
    cmd.output()?;
    let video_file = File::open(
        output_path
            .join(&video_name)
            .join(format!("{}.js", video_name)),
    )?;
    let video: Video = serde_json::from_reader(video_file)?;
    Ok(video)
}
