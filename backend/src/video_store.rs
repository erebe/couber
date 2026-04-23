use fs_extra::dir::CopyOptions;
use std::path::{Path, PathBuf};

pub struct VideoStore {
    videos_path: PathBuf,
}

impl VideoStore {
    pub fn new(videos_path: PathBuf) -> Self {
        Self { videos_path }
    }

    pub fn add(&self, name: &str, src_path: &Path) -> eyre::Result<PathBuf> {
        if !src_path.is_dir() {
            return Err(eyre::eyre!("Source path is not directory"));
        }
        let dest_dir = self.videos_path.join(name).to_path_buf();
        let opts = CopyOptions::new().copy_inside(true);
        fs_extra::dir::move_dir(src_path, &dest_dir, &opts)?;
        Ok(dest_dir)
    }

    pub fn delete(&self, name: &str) -> eyre::Result<()> {
        let video_dir = self.videos_path.join(name);
        if video_dir.exists() {
            std::fs::remove_dir_all(&video_dir)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup() -> (tempfile::TempDir, tempfile::TempDir, VideoStore) {
        let store_dir = tempfile::tempdir().expect("failed to create store temp dir");
        let src_dir = tempfile::tempdir().expect("failed to create src temp dir");
        let store = VideoStore::new(store_dir.path().to_path_buf());
        (store_dir, src_dir, store)
    }

    #[test]
    fn test_add_copies_file() {
        let (store_dir, src_dir, store) = setup();
        let src = src_dir.path().join("video.mp4");
        fs::write(&src, b"fake video content").unwrap();

        let dest = store.add("my-video", src_dir.path()).expect("add failed");

        assert!(dest.exists());
        assert!(store_dir.path().join("my-video").join("video.mp4").exists());
        assert_eq!(
            fs::read(&dest.join("video.mp4")).unwrap(),
            b"fake video content"
        );
    }

    #[test]
    fn test_delete_removes_directory() {
        let (store_dir, src_dir, store) = setup();
        let src = src_dir.path().join("clip.mp4");
        fs::write(&src, b"data").unwrap();

        store.add("to-delete", src_dir.path()).expect("add failed");
        assert!(store_dir.path().join("to-delete").exists());

        store.delete("to-delete").expect("delete failed");
        assert!(!store_dir.path().join("to-delete").exists());
    }
}
