mod scanner;

use clap::Parser;
use reqwest::blocking::Client;
use std::fs::{self, File, create_dir_all};
use std::io::{self, Write, BufRead, BufReader};
use std::path::Path;
use std::time::Duration;
use indicatif::{ProgressBar, ProgressStyle};

struct Data {
    data: Vec<u8>,
    html_content: String,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The URL to scan
    #[arg()]
    url: String,

    /// Path to the wordlist file
    #[arg(short = 'd', long = "wordlist")]
    wordlist: String,
}

//constants for the output and intel directory
const OUTPUT_DIR: &str = "found_html";
const INTEL_DIR_DISPLAY: &str = "intel";

fn sanitize_filename(url: &str) -> String {
    url.trim()
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url)
        .trim_end_matches('/')
        .replace('/', "_")
        .replace(':', "_")
        .replace('\\', "_")
        .replace('*', "_")
        .replace('?', "_")
        .replace('"', "_")
        .replace('|', "_")
        .replace('<', "_")
        .replace('>', "_")
        .replace(' ', "_")
}

fn create_file_and_scan(
    client: &Client,
    base_url: &str,
    normalized_path: &str,
    html_content: String,
) -> Result<(), Box<dyn std::error::Error>> {
    // Sanitize path for filename (handle slashes, etc.)
    let sanitized_path = normalized_path.trim_start_matches('/').replace('/', "_");
    let filename = if sanitized_path.is_empty() {
        format!(
            "{}/{}_root_data.txt",
            OUTPUT_DIR,
            sanitize_filename(base_url)
        )
    } else {
        format!(
            "{}/{}_{}_data.txt",
            OUTPUT_DIR,
            sanitize_filename(base_url),
            sanitized_path
        )
    };

    create_dir_all(OUTPUT_DIR)?;

    let mut file = File::create(&filename)?;
    file.write_all(html_content.as_bytes())?;
    println!("HTML saved in {}", filename);

    // Scan the saved file using the scanner module
    match scanner::scan_for_information(filename.clone()) {
        Ok(results) => {
            println!(
                "[+] Scan results for {}: Techs: {}, Emails: {}, Scripts: {}, Comments: {}",
                normalized_path,
                results.technologies.len(),
                results.emails.len(),
                results.scripts.len(),
                results.comments.len()
            );
        }
        Err(e) => eprintln!("Fehler beim Scannen der Datei {}: {}", filename, e),
    }
    Ok(())
}

fn download_robots_txt(client: &Client, base_url: &str) -> Result<(), Box<dyn std::error::Error>> {

    create_dir_all(OUTPUT_DIR)?;

    let robots_url = format!("{}robots.txt", base_url);
    match client.get(&robots_url).timeout(Duration::from_secs(10)).send() {
        Ok(resp) => {
            if resp.status().is_success() {
                match resp.text() {
                    Ok(content) => {
                        let robots_file_path = format!("{}/robots.txt", OUTPUT_DIR);
                        let mut file = File::create(&robots_file_path)?;
                        file.write_all(content.as_bytes())?;
                        println!("robots.txt saved in {}", robots_file_path);
                    }
                    Err(e) => eprintln!("Failed to get text from {}: {}", robots_url, e),
                }
            } else {
                println!(
                    "robots.txt not found or not accessible (Status: {}).",
                    resp.status()
                );
            }
        }
        Err(e) => {
            println!("Request failed for {}: {}", robots_url, e);
        }
    }
    Ok(())
}

fn parse_robots_txt(robots_txt_path: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut paths = Vec::new();

    if !Path::new(robots_txt_path).exists() {
        return Ok(paths);
    }

    let file = File::open(robots_txt_path)?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        let trimmed_line = line.trim_start();

        if trimmed_line.starts_with("Disallow:") || trimmed_line.starts_with("Allow:") {
            let parts: Vec<&str> = trimmed_line.splitn(2, ':').collect();
            if parts.len() == 2 {
                let path = parts[1].trim();
                if !path.is_empty() && path != "*" {
                    //make sure the path starts with a slash "/"
                    let normalized_path = if path.starts_with('/') {
                        path.to_string()
                    } else {
                        format!("/{}", path)
                    };
                    paths.push(normalized_path);
                }
            }
        }
        //User agent comments are ignored
    }
    Ok(paths)
}

fn download_and_save(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let selected_url = &args.url;
    let base_url = format!("{}/", selected_url.trim_end_matches('/'));

    create_dir_all(OUTPUT_DIR)?;

    let client = Client::new();
    let response = client.get(&base_url).send()?.error_for_status()?;
    let html_content = response.text()?;

    let data = Data {
        data: vec![],
        html_content: html_content.clone(),
    };

    println!("Should the initial page data be printed to the console? (y/n): ");
    let input = get_user_input();
    if input.eq_ignore_ascii_case("y") {
        println!("{}", data.html_content);
    }

    create_file_and_scan(&client, &base_url, "", data.html_content)?;

    download_robots_txt(&client, &base_url)?;
    let robots_txt_path = format!("{}/robots.txt", OUTPUT_DIR);
    let robots_paths = parse_robots_txt(&robots_txt_path)?;
    println!("Loaded {} paths from robots.txt.", robots_paths.len());

    let wordlist_path = &args.wordlist;
    let wordlist_paths = if Path::new(wordlist_path).exists() {
        let file = File::open(wordlist_path)?;
        let reader = BufReader::new(file);

        let lines: Result<Vec<String>, _> = reader.lines().collect();
        let paths: Vec<String> = lines?
            .into_iter()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .collect();
        println!("Loaded {} paths from {}.", paths.len(), wordlist_path);
        paths
    } else {
        eprintln!("Warning: Wordlist file '{}' not found. Continuing with predefined and robots.txt paths only.", wordlist_path);
        vec![]
    };

    //Predefined Paths
    let predefined_test_paths = vec![
        "/etc/passwd",
        "/etc/shadow",
        "/proc/self/environ",
        "/proc/version",
        "/.git/config",
        "/config.php",
        "/wp-config.php",
        "/sitemap.xml",
        "/.env",
        "/server-status",
        "/server-info",
        "/phpinfo.php",
        "/backup.sql",
        "/config/database.yml",
        "/WEB-INF/web.xml",
        "/web.config",
    ];

    let all_paths_to_test: Vec<String> = predefined_test_paths
        .into_iter()
        .map(String::from)
        .chain(robots_paths.into_iter())
        .chain(wordlist_paths.into_iter())
        .collect();

    let total_count = all_paths_to_test.len();
    let pb = ProgressBar::new(total_count as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")?
            .progress_chars("#>-"),
    );

    println!(
        "Starting scan with {} total paths (predefined + robots.txt + wordlist)...",
        total_count
    );

    //Scan Loop
    for path in all_paths_to_test {
        pb.inc(1);

        let normalized_path = path;

        let url = format!("{}{}", base_url.trim_end_matches('/'), normalized_path);

        match client.get(&url).timeout(Duration::from_secs(10)).send() {
            Ok(resp) => {
                if resp.status().is_success() {
                    let content_type = resp
                        .headers()
                        .get("content-type")
                        .and_then(|ct| ct.to_str().ok())
                        .unwrap_or("");

                    if content_type.contains("text/html") {
                        match resp.text() {
                            Ok(html) => {
                                //Saves a found HTML file and scans it.
                                if let Err(e) = create_file_and_scan(
                                    &client,
                                    &base_url,
                                    &normalized_path,
                                    html,
                                ) {
                                    eprintln!("Error processing {}: {}", url, e);
                                }
                            }
                            Err(e) => eprintln!("Failed to get text from {}: {}", url, e),
                        }
                    } else {
                        println!(
                            "Found non-HTML resource: {} (Content-Type: {})",
                            url, content_type
                        );
                        // Optional: Saving of non HTML resources can be added here.
                    }
                } else {
                    println!("Status {}: {}", resp.status(), url);
                }
            }
            Err(e) => {
                println!("Request failed for {}: {}", url, e);
            }
        }
    }
    pb.finish_with_message("Scan completed.");

    Ok(())
}


fn get_user_input() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Basis-URL-Validierung
    if !args.url.starts_with("http://") && !args.url.starts_with("https://") {
        eprintln!("Error: Please enter a full URL starting with http:// or https://");
        std::process::exit(1);
    }

    // --- Primärer Scan-Prozess ---
    match download_and_save(args) {
        Ok(()) => {
            println!("\n--- Scanning Phase Finished ---");

            println!("Creating intelligence summary in '{}' directory...", INTEL_DIR_DISPLAY);
            match scanner::scan_all_html_files() {
                Ok(()) => println!("Intelligence summary created successfully in '{}'.", INTEL_DIR_DISPLAY),
                Err(e) => eprintln!("Error creating intelligence summary: {}", e),
            }
        }
        Err(e) => {
            eprintln!("An unrecoverable error occurred during scanning: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}