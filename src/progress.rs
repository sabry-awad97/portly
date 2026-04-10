// Progress indicator module for long-running operations
// Provides spinners and progress bars with JSON mode and quiet flag support

use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// Wrapper for progress indicators that respects JSON mode and quiet flags
pub struct ProgressIndicator {
    spinner: Option<ProgressBar>,
}

/// Progress bar for determinate operations
pub struct ProgressCounter {
    bar: Option<ProgressBar>,
}

impl ProgressCounter {
    /// Create a new progress bar
    pub fn new(total: usize, message: &str, json_mode: bool, quiet: bool) -> Self {
        let should_show = !json_mode && !quiet;

        let bar = if should_show && total > 0 {
            let pb = ProgressBar::new(total as u64);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{msg} [{bar:40.cyan/blue}] {pos}/{len}")
                    .expect("Invalid progress template"),
            );
            pb.set_message(message.to_string());
            Some(pb)
        } else {
            None
        };

        Self { bar }
    }

    /// Increment the progress bar
    pub fn inc(&self) {
        if let Some(bar) = &self.bar {
            bar.inc(1);
        }
    }

    /// Finish the progress bar
    pub fn finish(self) {
        if let Some(bar) = self.bar {
            bar.finish_and_clear();
        }
    }
}

impl ProgressIndicator {
    /// Create a new progress indicator
    ///
    /// # Arguments
    /// * `message` - The message to display
    /// * `json_mode` - If true, suppresses all progress output
    /// * `quiet` - If true, suppresses all progress output
    pub fn new(message: &str, json_mode: bool, quiet: bool) -> Self {
        let should_show = !json_mode && !quiet;

        let spinner = if should_show {
            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.green} {msg}")
                    .expect("Invalid progress template"),
            );
            pb.set_message(message.to_string());
            pb.enable_steady_tick(Duration::from_millis(100));
            Some(pb)
        } else {
            None
        };

        Self { spinner }
    }

    /// Finish and clear the progress indicator
    pub fn finish(self) {
        if let Some(spinner) = self.spinner {
            spinner.finish_and_clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_indicator_can_be_created_and_started() {
        // Arrange: Create a progress indicator in quiet mode (to avoid terminal output in tests)
        let indicator = ProgressIndicator::new("Testing...", false, true);

        // Assert: Indicator should be created successfully (no spinner in quiet mode)
        assert!(indicator.spinner.is_none());
    }

    #[test]
    fn test_progress_indicator_shows_in_interactive_mode() {
        // Arrange: Create a progress indicator in interactive mode (not JSON, not quiet)
        let indicator = ProgressIndicator::new("Scanning...", false, false);

        // Assert: Indicator should have a spinner
        assert!(indicator.spinner.is_some());
    }

    #[test]
    fn test_progress_indicator_suppressed_in_json_mode() {
        // Arrange: Create a progress indicator with JSON mode enabled
        let indicator = ProgressIndicator::new("Scanning...", true, false);

        // Assert: Indicator should not show in JSON mode
        assert!(indicator.spinner.is_none());
    }

    #[test]
    fn test_progress_indicator_can_be_finished() {
        // Arrange: Create a progress indicator
        let indicator = ProgressIndicator::new("Processing...", false, true);

        // Act: Finish the indicator (should not panic)
        indicator.finish();

        // Assert: Test passes if no panic occurs
    }

    #[test]
    fn test_progress_indicator_finish_clears_spinner() {
        // Arrange: Create an indicator that shows
        let indicator = ProgressIndicator::new("Working...", false, false);

        // Act: Finish should clear the spinner
        indicator.finish();

        // Assert: Test passes if terminal is properly restored (no panic)
    }

    #[test]
    fn test_progress_indicator_respects_quiet_flag() {
        // Arrange: Create indicator with quiet flag
        let indicator = ProgressIndicator::new("Processing...", false, true);

        // Assert: Should not show when quiet is enabled
        assert!(indicator.spinner.is_none());
    }
}
