use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::Arc;

pub struct ProgressTracker {
    multi_progress: Arc<MultiProgress>,
}

impl ProgressTracker {
    pub fn new() -> Self {
        Self {
            multi_progress: Arc::new(MultiProgress::new()),
        }
    }

    pub fn create_progress_bar(&self, total: u64, message: &str) -> ProgressBar {
        let pb = self.multi_progress.add(ProgressBar::new(total));
        pb.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}",
                )
                .unwrap()
                .progress_chars("#>-"),
        );
        pb.set_message(message.to_string());
        pb.enable_steady_tick(std::time::Duration::from_millis(100));
        pb
    }

    pub fn create_download_progress_bar(&self, total: u64, filename: &str) -> ProgressBar {
        let pb = self.multi_progress.add(ProgressBar::new(total));
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta}) {msg}")
                .unwrap()
                .progress_chars("#>-"),
        );
        pb.set_message(format!("Downloading: {}", filename));
        pb.enable_steady_tick(std::time::Duration::from_millis(100));
        pb
    }

    pub fn remove_progress_bar(&self, pb: &ProgressBar) {
        self.multi_progress.remove(pb);
    }

    pub fn println(&self, message: &str) {
        self.multi_progress.println(message).unwrap_or(());
    }

    pub fn set_pinned_message(&self, message: &str) {
        self.println(&format!(">>> {}", message));
    }
}
