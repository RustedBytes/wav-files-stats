use clap::Parser;
use hound::{WavReader, WavSpec};
use std::path::{Path, PathBuf};
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

    let mut durations: Vec<Duration> = Vec::new();
    let mut file_count: usize = 0;
    let mut errors: Vec<String> = Vec::new();

    for entry in WalkDir::new(&path).follow_links(false).into_iter() {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                errors.push(format!("Failed to read entry: {}", e));
                continue;
            }
        };

        let file_path = entry.path();

        if file_path.is_file()
            && file_path
                .extension()
                .and_then(|s| s.to_str())
                .map_or(false, |ext| ext.eq_ignore_ascii_case("wav"))
        {
            match calculate_duration(file_path) {
                Ok(duration) => {
                    durations.push(duration);
                    file_count += 1;
                }
                Err(e) => {
                    errors.push(format!(
                        "Failed to read WAV file {}: {}",
                        file_path.display(),
                        e
                    ));
                }
            }
        }
    }

    print_stats(file_count, &durations, &errors)?;

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

fn print_stats(file_count: usize, durations: &[Duration], errors: &[String]) -> anyhow::Result<()> {
    if file_count == 0 {
        println!("No WAV files found in the directory tree.");
        return Ok(());
    }

    let total_duration = durations.iter().sum::<Duration>();
    let average_duration = if file_count > 0 {
        total_duration / file_count as u32
    } else {
        Duration::ZERO
    };

    let min_duration = durations.iter().min().unwrap_or(&Duration::ZERO);
    let max_duration = durations.iter().max().unwrap_or(&Duration::ZERO);

    println!("\nWAV File Statistics:");
    println!("====================");
    println!("Total files processed: {}", file_count);
    println!("Total duration: {:?}", total_duration);
    println!("Average duration: {:?}", average_duration);
    println!("Shortest file: {:?}", min_duration);
    println!("Longest file: {:?}", max_duration);
    println!("====================");
    println!("Number of errors/warnings: {}", errors.len());

    Ok(())
}
