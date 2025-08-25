use indicatif::{ProgressBar, ProgressStyle};

/// Creates a progress bar for visual feedback during synchronization
pub fn create_progress_bar(total_steps: u64, info: String) -> ProgressBar {
    let pb = ProgressBar::new(total_steps);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(&format!(
                "  {{elapsed_precise}} {} {{bar:40.cyan/blue}} {{pos}}/{{len}} ({{percent}}%) {{msg}}",
                info
            ))
            .unwrap()
            .progress_chars("█▉▊▋▌▍▎▏ "),
    );
    pb.tick();
    pb
}

/// Creates a simpler progress bar without elapsed time for sub-tasks
pub fn create_simple_progress_bar(total_steps: u64, info: String) -> ProgressBar {
    let pb = ProgressBar::new(total_steps);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(&format!(
                "  └─ {} {{bar:30.green/blue}} {{pos}}/{{len}} ({{percent}}%)",
                info
            ))
            .unwrap()
            .progress_chars("█▉▊▋▌▍▎▏ "),
    );
    pb.tick();
    pb
}
