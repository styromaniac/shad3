use anyhow::Result;
use futures::future::join_all;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use regex::Regex;
use reqwest::Client;
use sha3::{Digest, Sha3_512};
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};
use std::env;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const DEFAULT_OUTPUT_FILE: &str = "checksums.txt";

type DownloadResult = Result<Option<(String, Vec<u8>)>>;
type ProcessResult = Result<(Vec<(Vec<u8>, Vec<u8>)>, Duration, usize)>;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 || args.len() > 3 {
        anyhow::bail!("Usage: {} <blocklist-url> [output-path]", args[0]);
    }

    let url = &args[1];
    let output_path = args
        .get(2)
        .map(|s| s.as_str())
        .unwrap_or(DEFAULT_OUTPUT_FILE);

    let start_time = Instant::now();

    println!("Downloading file(s)...");
    let filenames = download_files(url).await?;

    println!("Determining most common prefix across all files...");
    let prefix = find_most_common_ip_prefix_across_files(&filenames)?;
    match &prefix {
        Some(p) => {
            println!(
                "Detected IP prefix: '{}' ({} parts)",
                p,
                p.split('.').count()
            );
            println!("This prefix will be removed from matching lines before hashing.");
        }
        None => println!("No common IP prefix detected. Processing all lines without removal."),
    }

    let mut all_checksums = Vec::new();
    let mut total_hashing_time = Duration::new(0, 0);
    let mut total_lines = 0;

    for filename in &filenames {
        println!("Processing file: {}", filename);
        let (checksums, hashing_time, lines) = process_file_parallel(filename, prefix.as_deref())?;

        all_checksums.extend(checksums);
        total_hashing_time += hashing_time;
        total_lines += lines;
    }

    let hashing_rate = total_lines as f64 / total_hashing_time.as_secs_f64();
    println!("Overall hashing rate: {:.2} hashes/second", hashing_rate);

    println!("Writing checksums...");
    write_sorted_checksums_parallel(&all_checksums, output_path)?;

    // Clean up downloaded files
    for filename in filenames {
        fs::remove_file(filename)?;
    }

    let duration = start_time.elapsed();
    println!("Done! Total execution time: {:.2?}", duration);
    Ok(())
}

async fn download_files(base_url: &str) -> Result<Vec<String>> {
    let client = Client::new();
    let mut filenames_to_download = Vec::new();

    let original_filename = base_url
        .split('/')
        .last()
        .unwrap_or("block.txt")
        .to_string();
    let (prefix, number_part) = original_filename.split_at(
        original_filename
            .find(char::is_numeric)
            .unwrap_or(original_filename.len()),
    );
    let padding_length = number_part.chars().take_while(|c| *c == '0').count();

    let original_number = extract_number(&original_filename);

    if let Some(start_num) = original_number {
        for num in (0..=start_num).rev() {
            let new_filename = format!("{}{:0width$}", prefix, num, width = padding_length + 1);
            filenames_to_download.push(new_filename);
        }

        let mut current_num = start_num + 1;
        loop {
            let new_filename = format!(
                "{}{:0width$}",
                prefix,
                current_num,
                width = padding_length + 1
            );
            let current_url = base_url.replace(&original_filename, &new_filename);

            if let Ok(response) = Client::new().head(&current_url).send().await {
                if response.status().is_success() {
                    filenames_to_download.push(new_filename);
                    current_num += 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    } else {
        filenames_to_download.push(original_filename.clone());
    }

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

    let download_results: Vec<DownloadResult> = join_all(download_futures).await;
    let mut downloaded_files = Vec::new();
    for result in download_results {
        if let Some((filename, content)) = result? {
            let file = File::create(&filename)?;
            let mut writer = BufWriter::new(file);
            writer.write_all(&content)?;
            writer.flush()?;
            downloaded_files.push(filename);
        }
    }

    if downloaded_files.is_empty() {
        return Err(anyhow::anyhow!("No files were downloaded"));
    }

    downloaded_files.sort_by_key(|a| extract_number(a));

    Ok(downloaded_files)
}

fn extract_number(filename: &str) -> Option<u32> {
    let path = Path::new(filename);
    let stem = path.file_stem()?.to_str()?;
    let re = Regex::new(r"(\d+)$").unwrap();
    re.captures(stem)
        .and_then(|cap| cap.get(1))
        .and_then(|m| m.as_str().parse().ok())
}

fn find_most_common_ip_prefix_across_files(filenames: &[String]) -> Result<Option<String>> {
    let mut prefix_counts = HashMap::new();
    let mut total_lines = 0;

    let ip_regex = Regex::new(r"^(\d{1,3}\.){1,3}\d{1,3}")?;

    for filename in filenames {
        let file = File::open(filename)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            total_lines += 1;
            if let Some(ip_match) = ip_regex.find(&line) {
                let full_ip = ip_match.as_str();
                *prefix_counts.entry(full_ip.to_string()).or_insert(0) += 1;
                for len in 1..=3 {
                    let prefix = full_ip.split('.').take(len).collect::<Vec<_>>().join(".");
                    *prefix_counts.entry(prefix).or_insert(0) += 1;
                }
            }
        }
    }

    Ok(prefix_counts
        .into_iter()
        .filter(|&(_, count)| count >= total_lines / 2)
        .max_by(|(a, _), (b, _)| {
            let a_parts = a.split('.').count();
            let b_parts = b.split('.').count();
            a_parts.cmp(&b_parts).then(a.len().cmp(&b.len()))
        })
        .map(|(prefix, _)| prefix))
}

fn process_file_parallel(filename: &str, prefix: Option<&str>) -> ProcessResult {
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

    let chunk_size = 10000;
    let mut checksums = Vec::new();
    let processed_lines = Arc::new(AtomicUsize::new(0));
    let skipped_lines = Arc::new(AtomicUsize::new(0));

    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    while processed_lines.load(Ordering::Relaxed) < total_lines {
        let chunk: Vec<_> = lines.by_ref().take(chunk_size).collect::<Result<_, _>>()?;

        let chunk_results: Vec<(Vec<u8>, Vec<u8>)> = chunk
            .par_iter()
            .filter_map(|line| {
                if line.trim().is_empty() {
                    skipped_lines.fetch_add(1, Ordering::Relaxed);
                    return None;
                }
                match prefix {
                    Some(p) if line.starts_with(p) => {
                        let processed_line = &line[p.len()..].trim_start();
                        let line_bytes = processed_line.as_bytes().to_vec();
                        let mut hasher = Sha3_512::new();
                        hasher.update(&line_bytes);
                        let hash = hasher.finalize().to_vec();
                        Some((line_bytes, hash))
                    }
                    Some(_) => {
                        skipped_lines.fetch_add(1, Ordering::Relaxed);
                        None
                    }
                    None => {
                        let line_bytes = line.trim().as_bytes().to_vec();
                        let mut hasher = Sha3_512::new();
                        hasher.update(&line_bytes);
                        let hash = hasher.finalize().to_vec();
                        Some((line_bytes, hash))
                    }
                }
            })
            .collect();

        checksums.extend(chunk_results);
        let new_processed = processed_lines.fetch_add(chunk.len(), Ordering::Relaxed) + chunk.len();
        progress_bar.set_position(new_processed as u64);
    }

    progress_bar.finish_with_message("Processing complete");

    let hashing_time = start_time.elapsed();

    println!("Total lines: {}", total_lines);
    println!("Processed lines: {}", checksums.len());
    println!("Skipped lines: {}", skipped_lines.load(Ordering::Relaxed));

    Ok((checksums, hashing_time, total_lines))
}

fn write_sorted_checksums_parallel(
    checksums: &[(Vec<u8>, Vec<u8>)],
    output_file: &str,
) -> Result<()> {
    println!("Sorting and writing checksums...");
    let total_checksums = checksums.len();
    let chunk_size = 1_000_000;

    let writer = Arc::new(Mutex::new(BufWriter::new(File::create(output_file)?)));
    let progress_bar = ProgressBar::new(total_checksums as u64);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_bar()),
    );

    let temp_files = create_sorted_temp_files(checksums, chunk_size, &progress_bar)?;

    merge_sorted_files(&temp_files, &writer, &progress_bar)?;

    writer.lock().unwrap().flush()?;
    progress_bar.finish_with_message("Finished writing checksums");
    println!("\nFinished writing checksums to {}", output_file);

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

    for (i, reader) in readers.iter_mut().enumerate() {
        let mut line = String::new();
        if reader.read_line(&mut line)? > 0 {
            heap.push(Reverse((line.trim().to_string(), i)));
        }
    }

    let mut buffer = String::new();
    let buffer_size = 10000;

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

    if !buffer.is_empty() {
        writer.lock().unwrap().write_all(buffer.as_bytes())?;
        progress_bar.inc(buffer.lines().count() as u64);
    }

    Ok(())
}
