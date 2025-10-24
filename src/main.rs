use clap::Parser;
use hound::{WavReader, WavSpec};
use std::path::{Path, PathBuf};
use rayon::prelude::*;
use std::time::Duration;
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The root directory to scan for WAV files
    path: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let path = args.path;

    if !path.exists() {
        anyhow::bail!("Provided path does not exist: {}", path.display());
    }

    if !path.is_dir() {
        anyhow::bail!("Provided path is not a directory: {}", path.display());
    }

    let (durations, errors): (Vec<_>, Vec<_>) = WalkDir::new(&path)
        .follow_links(false)
        .into_iter()
        .par_bridge() // Switch to a parallel iterator
        .filter_map(|entry_result| {
            match entry_result {
                Ok(entry) => {
                    let file_path = entry.path();
                    if file_path.is_file() && file_path.extension().and_then(|s| s.to_str()).is_some_and(|ext| ext.eq_ignore_ascii_case("wav")) {
                        Some(match calculate_duration(file_path) {
                            Ok(duration) => Ok(duration),
                            Err(e) => Err(format!("Failed to read WAV file {}: {}", file_path.display(), e)),
                        })
                    } else {
                        None // Not a .wav file, so we skip it.
                    }
                }
                Err(e) => Some(Err(format!("Failed to read entry: {}", e))),
            }
        })
        .partition(Result::is_ok);

    let durations: Vec<Duration> = durations.into_iter().map(Result::unwrap).collect();
    let errors: Vec<String> = errors.into_iter().map(Result::unwrap_err).collect();

    print_stats(durations.len(), &durations, &errors)?;

    if !errors.is_empty() {
        eprintln!("\nWarnings:");
        for error in errors {
            eprintln!("  - {}", error);
        }
    }

    Ok(())
}

fn calculate_duration(path: &Path) -> anyhow::Result<Duration> {
    let reader = WavReader::open(path)?;
    let spec: WavSpec = reader.spec();
    let len = reader.len() as u64;

    if len == 0 {
        anyhow::bail!("Empty audio file");
    }

    let duration_secs = len as f64 / spec.sample_rate as f64;
    let duration = Duration::from_secs_f64(duration_secs);

    Ok(duration)
}

/// Formats a `Duration` into a human-readable string like "1h 2m 3s".
fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();

    if total_seconds == 0 {
        return "0s".to_string();
    }

    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    let mut parts = Vec::new();
    if hours > 0 {
        parts.push(format!("{}h", hours));
    }
    if minutes > 0 {
        parts.push(format!("{}m", minutes));
    }
    if seconds > 0 {
        parts.push(format!("{}s", seconds));
    }

    parts.join(" ")
}

fn print_stats(file_count: usize, durations: &[Duration], errors: &[String]) -> anyhow::Result<()> {
    if file_count == 0 {
        println!("No WAV files found in the directory tree.");
        return Ok(());
    }

    let total_duration = durations.par_iter().sum::<Duration>();
    let average_duration = if file_count > 0 {
        total_duration / file_count as u32
    } else {
        Duration::ZERO
    };

    let min_duration = durations.par_iter().min().unwrap_or(&Duration::ZERO);
    let max_duration = durations.par_iter().max().unwrap_or(&Duration::ZERO);

    println!("\nWAV File Statistics:");
    println!("====================");
    println!("Total files processed: {}", file_count);
    println!("Total duration: {}", format_duration(total_duration));
    println!("Average duration: {}", format_duration(average_duration));
    println!("Shortest file: {}", format_duration(*min_duration));
    println!("Longest file: {}", format_duration(*max_duration));
    println!("===================="); // This line is new, but it matches the README.md example.
    println!("Number of errors/warnings: {}", errors.len());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_calculate_duration_valid_wav() -> anyhow::Result<()> {
        let dir = TempDir::new()?;
        let wav_path = dir.path().join("test.wav");
        let mut file = File::create(&wav_path)?;
        // Write minimal valid WAV header (44 bytes) + 1 second of silence at 44100 Hz, 1 channel, 16-bit
        // Note: This is a simplified header; in practice, use hound to generate.
        let header = include_bytes!("../test_data/minimal_wav_header.bin"); // Assume a test fixture binary
        file.write_all(header)?;
        file.write_all(&[0u8; 88200])?; // 1s of 16-bit samples

        let duration = calculate_duration(&wav_path)?;
        assert_eq!(duration.as_secs_f64(), 1.0);

        Ok(())
    }

    #[test]
    fn test_calculate_duration_empty_wav() {
        let dir = TempDir::new().unwrap();
        let wav_path = dir.path().join("empty.wav");
        File::create(&wav_path).unwrap();

        let result = calculate_duration(&wav_path);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Failed to read enough bytes."
        );
    }

    #[test]
    fn test_calculate_duration_non_wav() {
        let dir = TempDir::new().unwrap();
        let txt_path = dir.path().join("test.txt");
        File::create(&txt_path).unwrap();

        let result = calculate_duration(&txt_path);
        assert!(result.is_err()); // hound::open fails on non-WAV
    }

    #[test]
    fn test_print_stats_no_files() {
        let durations: Vec<Duration> = Vec::new();
        let errors: Vec<String> = Vec::new();
        let result = print_stats(0, &durations, &errors);
        assert!(result.is_ok());
        // Output verification would require output capture
    }

    #[test]
    fn test_print_stats_with_files() {
        let durations = vec![Duration::from_secs(1), Duration::from_secs(2)];
        let errors: Vec<String> = Vec::new();
        let result = print_stats(2, &durations, &errors);
        assert!(result.is_ok());
        // Total: 3s, Avg: 1.5s, Min:1s, Max:2s (verification via expected output capture)
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_secs(0)), "0s");
        assert_eq!(format_duration(Duration::from_secs(45)), "45s");
        assert_eq!(format_duration(Duration::from_secs(148)), "2m 28s");
        assert_eq!(format_duration(Duration::from_secs(252)), "4m 12s");
        assert_eq!(
            format_duration(Duration::from_secs(3600 + 120 + 3)),
            "1h 2m 3s"
        );
        assert_eq!(format_duration(Duration::from_secs(3600)), "1h");
        assert_eq!(format_duration(Duration::from_secs(3603)), "1h 3s");
    }
}
