use std::collections::HashSet;
use std::fs::{self, create_dir_all, File, read_dir};
use std::io::{Write, BufWriter};
use std::path::{Path, PathBuf};
use scraper::{Html, Selector};
use regex::Regex;

pub fn get_intel_dir_path() -> PathBuf {
    let found_html_path = Path::new("found_html");

    found_html_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("intel")
}

#[derive(Debug, Default)]
pub struct ScannerInfos {
    pub webserver: Vec<String>,
    pub technologies: HashSet<String>,
    pub emails: HashSet<String>,
    pub scripts: HashSet<String>,
    pub comments: Vec<String>,
    pub meta_tags: Vec<String>,
    pub links: HashSet<String>,
    pub api_endpoints: HashSet<String>,
}

pub fn scan_for_information(filename: String) -> Result<ScannerInfos, Box<dyn std::error::Error>> {
    println!("Debug: Scanning file: {}", filename);

    let mut info = ScannerInfos::default();
    let content = fs::read_to_string(&filename)?;
    let document = Html::parse_document(&content);

    let script_selector = Selector::parse("script[src]").unwrap();
    for element in document.select(&script_selector) {
        if let Some(src) = element.value().attr("src") {
            info.scripts.insert(src.to_string());
        }
    }

    let link_css_selector = Selector::parse("link[rel='stylesheet'][href]").unwrap();
    for element in document.select(&link_css_selector) {
        if let Some(href) = element.value().attr("href") {
            info.scripts.insert(href.to_string());
        }
    }

    let link_href_selector = Selector::parse("a[href]").unwrap();
    for element in document.select(&link_href_selector) {
        if let Some(href) = element.value().attr("href") {
            if href.starts_with("http") {
                info.links.insert(href.to_string());
            }
            //Logic for scanning Internal Links could be added here
        }
    }

    let meta_name_selector = Selector::parse("meta[name][content]").unwrap();
    for element in document.select(&meta_name_selector) {
        let name = element.value().attr("name").unwrap_or("");
        let content_val = element.value().attr("content").unwrap_or("");
        if !name.is_empty() && !content_val.is_empty() {
            info.meta_tags.push(format!("{}: {}", name, content_val));
        }
    }

    let title_selector = Selector::parse("title").unwrap();
    if let Some(title_element) = document.select(&title_selector).next() {
        let title_text = title_element.text().collect::<Vec<_>>().join(" ").trim().to_string();
        if !title_text.is_empty() {
            // Optional: info.technologies.insert(format!("Title: {}", title_text));
        }
    }

    let document_text_lower = document.root_element().text().collect::<String>().to_lowercase();
    let tech_keywords = [
        "google", "gws", "nginx", "apache", "react", "angular", "vue.js", "webpack", "jquery",
        "adservice", "gstatic", "googlesyndication", "analytics", "gtag", "closure library",
        "trustedtypes", "gapi", "material", "lit", "polymer", "bootstrap", "font awesome",
    ];
    for tech in &tech_keywords {
        if document_text_lower.contains(&tech.to_lowercase()) {
            info.technologies.insert(tech.to_string());
        }
    }

    //Regex for Email addresses
    let re_email = Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}")?;
    for cap in re_email.captures_iter(&content) {
        if let Some(email) = cap.get(0) {
            info.emails.insert(email.as_str().to_string());
        }
    }

    let re_comment = Regex::new(r"(?s)<!--(.*?)-->")?;
    for cap in re_comment.captures_iter(&content) {
        if let Some(comment_match) = cap.get(1) {
            let comment_text = comment_match.as_str().trim();
            if !comment_text.is_empty() {
                info.comments.push(comment_text.to_string());
            }
        }
    }

    let re_api = Regex::new(r"/(api|gen_204|client_204|log|og/_/js|_/js|_/ss|graphql|rest)/[^?\s\'<>]*")?;
    for cap in re_api.captures_iter(&content) {
         if let Some(api) = cap.get(0) {
             let api_str = api.as_str();
             if api_str.len() > 4 && !api_str.contains("://") {
            info.api_endpoints.insert(api_str.to_string());
             }
         }
    }

    let data_src_selector = Selector::parse("[data-src]").unwrap();
    for element in document.select(&data_src_selector) {
        if let Some(data_src) = element.value().attr("data-src") {
            info.scripts.insert(data_src.to_string());
        }
    }

    Ok(info)
}

//writes all collected information to files in the intel directory
fn write_summary_to_files(all_info: ScannerInfos) -> Result<(), Box<dyn std::error::Error>> {
    let intel_dir_path = get_intel_dir_path();
    let intel_dir_str = intel_dir_path.to_string_lossy();

    create_dir_all(&intel_dir_path)?;
    println!("Debug: Ensuring intel directory exists at: {}", intel_dir_str);

    fn write_items_to_file<S>(
        base_dir: &Path,
        filename: &str,
        items: impl IntoIterator<Item = S>,
    ) -> Result<(), std::io::Error>
    where
        S: AsRef<str>,
    {
        let path = base_dir.join(filename);
        let file = File::create(&path)?;
        let mut writer = BufWriter::new(file);
        let mut count = 0;
        for item in items {
            writeln!(writer, "{}", item.as_ref())?;
            count += 1;
        }
        writer.flush()?;
        println!("Intel summary written to {} ({} items)", path.display(), count);
        Ok(())
    }

    println!("Debug: Writing to files - Techs: {}, Emails: {}, Scripts: {}, Comments: {}, Meta: {}, Links: {}, APIs: {}",
             all_info.technologies.len(),
             all_info.emails.len(),
             all_info.scripts.len(),
             all_info.comments.len(),
             all_info.meta_tags.len(),
             all_info.links.len(),
             all_info.api_endpoints.len()
    );

    write_items_to_file(&intel_dir_path, "technologies.txt", &all_info.technologies)?;
    write_items_to_file(&intel_dir_path, "emails.txt", &all_info.emails)?;
    write_items_to_file(&intel_dir_path, "scripts.txt", &all_info.scripts)?;
    write_items_to_file(&intel_dir_path, "comments.txt", &all_info.comments)?;
    write_items_to_file(&intel_dir_path, "meta_tags.txt", &all_info.meta_tags)?;
    write_items_to_file(&intel_dir_path, "links.txt", &all_info.links)?;
    write_items_to_file(&intel_dir_path, "api_endpoints.txt", &all_info.api_endpoints)?;

    if !all_info.webserver.is_empty() {
        write_items_to_file(&intel_dir_path, "webserver.txt", &all_info.webserver)?;
    } else {
        let path = intel_dir_path.join("webserver.txt");
        File::create(&path)?;
        println!("No webserver info found, created empty {}", path.display());
    }

    Ok(())
}

//main function for scanning all html files in the found_html directory
pub fn scan_all_html_files() -> Result<(), Box<dyn std::error::Error>> {
    let found_html_path_str = "found_html";
    let found_html_path = Path::new(found_html_path_str);
    if !found_html_path.exists() {
        eprintln!("Directory '{}' not found. Nothing to scan.", found_html_path_str);
        return Ok(());
    }
    if !found_html_path.is_dir() {
        eprintln!("'{}' is not a directory.", found_html_path_str);
        return Ok(());
    }

    let entries = read_dir(found_html_path)?;
    println!("Debug: Looking for HTML files in directory: {}", found_html_path_str);

    let mut all_collected_info = ScannerInfos::default();
    let mut files_scanned = 0;
    let mut files_with_errors = 0;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(extension) = path.extension() {
                if extension == "html" || extension == "htm" || extension == "txt" {
                    let filename_str = path.to_string_lossy().to_string();
                    println!("Scanning {}", filename_str);
                    files_scanned += 1;

                    match scan_for_information(filename_str) {
                        Ok(file_info) => {
                            println!("Debug: Successfully scanned file.");
                            println!("Debug: Found {} techs, {} emails, {} scripts, {} comments, {} meta tags, {} links, {} APIs",
                                     file_info.technologies.len(),
                                     file_info.emails.len(),
                                     file_info.scripts.len(),
                                     file_info.comments.len(),
                                     file_info.meta_tags.len(),
                                     file_info.links.len(),
                                     file_info.api_endpoints.len()
                            );

                            all_collected_info.technologies.extend(file_info.technologies);
                            all_collected_info.emails.extend(file_info.emails);
                            all_collected_info.scripts.extend(file_info.scripts);
                            all_collected_info.comments.extend(file_info.comments);
                            all_collected_info.meta_tags.extend(file_info.meta_tags);
                            all_collected_info.links.extend(file_info.links);
                            all_collected_info.api_endpoints.extend(file_info.api_endpoints);
                            all_collected_info.webserver.extend(file_info.webserver);
                        }
                        Err(e) => {
                            eprintln!("Error scanning {}: {}", path.display(), e);
                            files_with_errors += 1;
                        }
                    }
                } else {
                    println!("Debug: Skipping file with extension: {:?}", extension);
                }
            } else {
                println!("Debug: Skipping file without extension: {:?}", path);
            }
        } else {
            println!("Debug: Skipping directory entry: {:?}", path);
        }
    }

    println!("Finished scanning {} HTML files ({} errors).", files_scanned, files_with_errors);
    println!("Total unique items found across all files:");
    println!("  Technologies: {}", all_collected_info.technologies.len());
    println!("  Emails: {}", all_collected_info.emails.len());
    println!("  Scripts: {}", all_collected_info.scripts.len());
    println!("  Comments: {}", all_collected_info.comments.len());
    println!("  Meta Tags: {}", all_collected_info.meta_tags.len());
    println!("  Links: {}", all_collected_info.links.len());
    println!("  API Endpoints: {}", all_collected_info.api_endpoints.len());

    let intel_dir_path = get_intel_dir_path();
    println!("Creating intelligence summary in '{}' directory...", intel_dir_path.display());
    write_summary_to_files(all_collected_info)?; //make sure this is called after all files are scanned

    println!("All HTML files scanned. Summary written to '{}/' directory.", intel_dir_path.display());
    Ok(())
}