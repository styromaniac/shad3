use anyhow::Result;
use futures::future::join_all;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use regex::Regex;
use reqwest::Client;
use sha3::{Digest, Sha3_512};
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::env;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const CHECKSUMS_DIR: &str = "checksums";
const OUTPUT_FILE: &str = "checksums/all_checksums.txt";

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        anyhow::bail!("Usage: {} <blocklist-url> [--expect <prefix>]", args[0]);
    }

    let url = &args[1];
    let expect = args
        .iter()
        .position(|arg| arg == "--expect")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.to_string());

    let start_time = Instant::now();

    println!("Downloading file(s)...");
    let filename = download_files(url).await?;

    println!("Processing file...");
    let (checksums, hashing_time, total_lines) = process_file_parallel(&filename, expect.as_deref())?;

    // Calculate and display hashing rate
    let hashing_rate = total_lines as f64 / hashing_time.as_secs_f64();
    println!("Hashing rate: {:.2} hashes/second", hashing_rate);

    println!("Writing checksums...");
    fs::create_dir_all(CHECKSUMS_DIR)?;
    write_sorted_checksums_parallel(&checksums, OUTPUT_FILE)?;

    fs::remove_file(filename)?;

    let duration = start_time.elapsed();
    println!("Done! Total execution time: {:.2?}", duration);
    Ok(())
}

async fn download_files(base_url: &str) -> Result<String> {
    let client = Client::new();
    let mut filenames_to_download = Vec::new();

    let original_filename = base_url.split('/').last().unwrap_or("block.txt").to_string();
    let (prefix, number_part) = original_filename.split_at(
        original_filename
            .find(char::is_numeric)
            .unwrap_or(original_filename.len()),
    );
    let padding_length = number_part.chars().take_while(|c| *c == '0').count();

    let mut current_number = extract_number(&original_filename);

    // Generate all filenames to download
    loop {
        let new_filename = if let Some(num) = current_number {
            format!("{}{:0width$}", prefix, num, width = padding_length + 1)
        } else {
            original_filename.clone()
        };

        filenames_to_download.push(new_filename);

        if current_number.is_none() || current_number == Some(0) {
            break;
        }
        current_number = current_number.map(|n| n - 1);
    }

    // Parallel download
    let original_filename_clone = original_filename.clone();
    let download_futures = filenames_to_download.into_iter().map(|filename| {
        let client = client.clone();
        let base_url = base_url.to_string();
        let original_filename = original_filename_clone.clone();
        async move {
            let current_url = base_url.replace(&original_filename, &filename);
            println!("Attempting to download: {}", current_url);

            let response = client.get(&current_url).send().await?;
            if !response.status().is_success() {
                println!("File not found: {}", current_url);
                return Ok(None);
            }

            println!("Downloading: {}", current_url);
            let content = response.bytes().await?;
            Ok(Some((filename, content.to_vec())))
        }
    });

    let download_results: Vec<Result<Option<(String, Vec<u8>)>>> = join_all(download_futures).await;
    let mut downloaded_files: Vec<(String, Vec<u8>)> = download_results
        .into_iter()
        .filter_map(|r| r.ok().flatten())
        .collect();

    if downloaded_files.is_empty() {
        return Err(anyhow::anyhow!("No files were downloaded"));
    }

    // Sort files in descending order based on extracted number
    downloaded_files.sort_by(|(a, _), (b, _)| extract_number(b).cmp(&extract_number(a)));

    // Combine all downloaded files
    let combined_filename = "combined_block.txt";
    let mut combined_file = File::create(combined_filename)?;
    for (_, content) in downloaded_files.iter() {
        combined_file.write_all(content)?;
    }

    Ok(combined_filename.to_string())
}

fn extract_number(filename: &str) -> Option<u32> {
    let path = Path::new(filename);
    let stem = path.file_stem()?.to_str()?;
    let re = Regex::new(r"(\d+)$").unwrap();
    re.captures(stem)
        .and_then(|cap| cap.get(1))
        .and_then(|m| m.as_str().parse().ok())
}

fn process_file_parallel(
    filename: &str,
    expect: Option<&str>,
) -> Result<(Vec<(Vec<u8>, Vec<u8>)>, Duration, usize)> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let total_lines = reader.lines().count();
    let progress_bar = ProgressBar::new(total_lines as u64);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_bar()),
    );

    let start_time = Instant::now();

    let chunk_size = 10000; // Adjust this value based on your available memory
    let mut checksums = Vec::new();
    let mut processed_lines = 0;

    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    while processed_lines < total_lines {
        let chunk: Vec<_> = lines.by_ref().take(chunk_size).collect::<Result<_, _>>()?;

        let chunk_results: Vec<(Vec<u8>, Vec<u8>)> = chunk
            .par_iter()
            .filter_map(|line| {
                let should_process = if let Some(prefix) = expect {
                    line.starts_with(prefix)
                } else {
                    true
                };

                if should_process {
                    let processed_line = if let Some(prefix) = expect {
                        line.strip_prefix(prefix).unwrap_or(line)
                    } else {
                        line
                    };

                    let line_bytes = processed_line.as_bytes().to_vec();
                    let mut hasher = Sha3_512::new();
                    hasher.update(&line_bytes);
                    let hash = hasher.finalize().to_vec();
                    Some((line_bytes, hash))
                } else {
                    None
                }
            })
            .collect();

        checksums.extend(chunk_results);
        processed_lines += chunk.len();
        progress_bar.set_position(processed_lines as u64);
    }

    progress_bar.finish_with_message("Processing complete");

    let hashing_time = start_time.elapsed();

    Ok((checksums, hashing_time, total_lines))
}

fn write_sorted_checksums_parallel(
    checksums: &[(Vec<u8>, Vec<u8>)],
    output_file: &str,
) -> Result<()> {
    println!("Sorting and writing checksums...");
    let total_checksums = checksums.len();
    let chunk_size = 1_000_000; // Adjust this based on available memory

    let writer = Arc::new(Mutex::new(BufWriter::new(File::create(output_file)?)));
    let progress_bar = ProgressBar::new(total_checksums as u64);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_bar()),
    );

    // Create sorted temp files
    let temp_files = create_sorted_temp_files(checksums, chunk_size, &progress_bar)?;

    // Merge sorted temp files
    merge_sorted_files(&temp_files, &writer, &progress_bar)?;

    writer.lock().unwrap().flush()?;
    progress_bar.finish_with_message("Finished writing checksums");
    println!("\nFinished writing checksums to {}", output_file);

    // Clean up temp files
    for temp_file in temp_files {
        fs::remove_file(temp_file)?;
    }

    Ok(())
}

fn create_sorted_temp_files(
    checksums: &[(Vec<u8>, Vec<u8>)],
    chunk_size: usize,
    progress_bar: &ProgressBar,
) -> Result<Vec<String>> {
    let mut temp_files = Vec::new();

    for (i, chunk) in checksums.chunks(chunk_size).enumerate() {
        let mut sorted_chunk = chunk.to_vec();
        sorted_chunk.par_sort_by(|a, b| a.1.cmp(&b.1));

        let temp_filename = format!("temp_sorted_{}.txt", i);
        let temp_file = File::create(&temp_filename)?;
        let mut temp_writer = BufWriter::new(temp_file);

        for (_, checksum) in sorted_chunk.iter() {
            writeln!(temp_writer, "{}", hex::encode(checksum))?;
        }
        temp_writer.flush()?;

        temp_files.push(temp_filename);
        progress_bar.inc(chunk.len() as u64);
    }

    Ok(temp_files)
}

fn merge_sorted_files(
    temp_files: &[String],
    writer: &Arc<Mutex<BufWriter<File>>>,
    progress_bar: &ProgressBar,
) -> Result<()> {
    let mut readers: Vec<BufReader<File>> = temp_files
        .iter()
        .map(|filename| BufReader::new(File::open(filename).unwrap()))
        .collect();

    let mut heap = BinaryHeap::new();

    // Initialize the heap with the first line from each file
    for (i, reader) in readers.iter_mut().enumerate() {
        let mut line = String::new();
        if reader.read_line(&mut line)? > 0 {
            heap.push(Reverse((line.trim().to_string(), i)));
        }
    }

    let mut buffer = String::new();
    let buffer_size = 10000; // Adjust based on memory constraints

    while let Some(Reverse((checksum, file_index))) = heap.pop() {
        buffer.push_str(&checksum);
        buffer.push('\n');

        if buffer.len() >= buffer_size {
            writer.lock().unwrap().write_all(buffer.as_bytes())?;
            progress_bar.inc(buffer.lines().count() as u64);
            buffer.clear();
        }

        let mut next_line = String::new();
        if readers[file_index].read_line(&mut next_line)? > 0 {
            heap.push(Reverse((next_line.trim().to_string(), file_index)));
        }
    }

    // Write any remaining buffer content
    if !buffer.is_empty() {
        writer.lock().unwrap().write_all(buffer.as_bytes())?;
        progress_bar.inc(buffer.lines().count() as u64);
    }

    Ok(())
}
