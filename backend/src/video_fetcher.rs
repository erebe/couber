use crate::database::Video;
use sha2::Digest;
use std::fs::File;
use std::path::PathBuf;
use std::process::Command;
use tracing::info;

pub struct VideoFetcher {
    scripts_path: PathBuf,
}

impl VideoFetcher {
    pub fn new(scripts_path: PathBuf) -> Self {
        Self { scripts_path }
    }

    pub fn fetch_coub(&self, coub_name: &str) -> eyre::Result<(Video, PathBuf)> {
        let mut cmd = Command::new("./coub.sh");
        cmd.args([coub_name]).current_dir(&self.scripts_path);
        info!("Coub fetched: {:?}", cmd);
        cmd.output()?;
        let video_file = File::open(
            self.scripts_path
                .join(coub_name)
                .join(format!("{}.js", coub_name)),
        )?;
        let mut video: Video = serde_json::from_reader(video_file)?;
        video.tags = video
            .tags
            .iter()
            .map(|t| urlencoding::decode(&t).unwrap_or_default().to_lowercase())
            .collect::<Vec<_>>();
        Ok((video, self.scripts_path.join(coub_name)))
    }

    pub fn fetch_video(&self, video_url: &str) -> eyre::Result<(Video, PathBuf)> {
        fn calculate_hash(t: &str) -> String {
            let hash = sha2::Sha256::digest(t.as_bytes());
            let mut hash = hex::encode(hash);
            hash.truncate(10);
            hash
        }
        let video_name = calculate_hash(video_url);
        let mut cmd = Command::new("./generic_vids.sh");
        cmd.arg(video_url)
            .arg(&video_name)
            .current_dir(&self.scripts_path);
        info!("Video fetched: {:?}", cmd);
        cmd.output()?;
        let video_file = File::open(
            self.scripts_path
                .join(&video_name)
                .join(format!("{}.js", video_name)),
        )?;
        let video: Video = serde_json::from_reader(video_file)?;
        Ok((video, self.scripts_path.join(&video_name)))
    }
}
