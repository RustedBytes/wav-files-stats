# wav-files-stats

A lightweight Rust CLI tool to recursively scan a directory for WAV audio files, calculate their durations, and display pretty statistics like total duration, average, min, and max.

## Features

- **Recursive Directory Scanning**: Processes WAV files in subfolders using efficient traversal.
- **Duration Calculation**: Reads WAV headers to compute precise durations without full file loading.
- **Robust Error Handling**: Skips invalid files gracefully and reports warnings.
- **Pretty Output**: Formatted stats for quick insights.
- **Idiomatic Rust**: Built with safety, concurrency primitives, and minimal dependencies.

## Installation

Add as a binary crate or install via Cargo:

```bash
cargo install --git https://github.com/RustedBytes/wav-files-stats
```

For development, clone the repo and build:

```bash
git clone https://github.com/RustedBytes/wav-files-stats
cd wav-duration-stats
cargo build --release
```

## Usage

```bash
wav-duration-stats /path/to/audio/folder
```

### Example Output

```
WAV File Statistics:
====================
Total files processed: 5
Total duration: 12m 34s
Average duration: 2m 28s
Shortest file: 45s
Longest file: 4m 12s

Warnings:
  - Failed to read WAV file subfolder/invalid.wav: Invalid format
```

## Testing

Run the test suite:

```bash
cargo test
```

Tests cover duration calculation (valid/empty/invalid files) and stats printing (with/without files). Uses `tempfile` for isolated fixtures.

## Dependencies

- `clap`: Argument parsing.
- `hound`: WAV file reading.
- `walkdir`: Recursive directory traversal.
- `anyhow`: Error handling.

See `Cargo.toml` for versions.

## Contributing

1. Fork the repo.
2. Create a feature branch (`git checkout -b feature/AmazingFeature`).
3. Commit changes (`git commit -m 'Add some AmazingFeature'`).
4. Push to the branch (`git push origin feature/AmazingFeature`).
5. Open a Pull Request.


## Cite

```
@software{Smoliakov_Wav_Files_Toolkit,
  author = {Smoliakov, Yehor},
  month = oct,
  title = {{WAV Files Toolkit: A suite of command-line tools for common WAV audio processing tasks, including conversion from other formats, data augmentation, loudness normalization, spectrogram generation, and validation.}},
  url = {https://github.com/RustedBytes/wav-files-toolkit},
  version = {0.4.0},
  year = {2025}
}
```
