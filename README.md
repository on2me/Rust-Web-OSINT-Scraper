# Rust Web OSINT Scraper

A simple web scraper and OSINT (Open Source Intelligence) gathering tool written in Rust. This tool downloads HTML pages from a target website, saves them locally, and extracts various pieces of information like emails, scripts, comments, and potential API endpoints for further analysis.

## Features

*   **Website Crawling:** Downloads the main page, `robots.txt`, and recursively scans paths found in `robots.txt` and a user-provided wordlist.
*   **Local Storage:** Saves all downloaded HTML content to a dedicated directory (`found_html`).
*   **Information Extraction:** Parses saved HTML files to find:
    *   Email addresses
    *   Script and stylesheet sources (`<script src=...>`, `<link href=...>`)
    *   External links (`<a href=...>`)
    *   Meta tags (`<meta name=... content=...>`)
    *   HTML comments (`<!-- ... -->`)
    *   Potential API endpoints (using pattern matching)
    *   Technology keywords (e.g., React, Nginx, Google)
*   **Intelligence Summary:** Aggregates all extracted information from the scanned pages and saves each category (emails, scripts, etc.) into separate text files within an `intel` directory for easy review.

## Prerequisites

*   **Rust:** You need the Rust toolchain installed. It's recommended to install it via [rustup](https://www.rust-lang.org/tools/install).
    ```bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    # Follow the instructions, then reload your shell or run:
    source $HOME/.cargo/env
    ```
*   **Cargo:** This is included with the Rust toolchain and is used for building and running the project.
*   **Wordlist:** A text file containing paths/directories to scan on the target website (e.g., `directory-list-2.3-small.txt`, `common.txt`).

## Installation

1.  **Clone the Repository:**
    ```bash
    git clone https://github.com/on2me/Rust-Web-OSINT-Scraper.git
    cd Rust-Web-OSINT-Scraper
    ```
2.  **Build the Project:**
    Use Cargo to compile the project.
    ```bash
    cargo build --release
    ```
    This will create the executable at `target/release/rust-web-osint-scraper`.

## Usage

1.  **Run the Scraper:**
    Execute the program with the target URL and the path to your wordlist.
    ```bash
    ./target/release/rust-web-osint-scraper <TARGET_URL> -d <PATH_TO_WORDLIST>
    ```
    **Example:**
    ```bash
    ./target/release/rust-web-osint-scraper https://example.com -d /usr/share/dirbuster/wordlists/directory-list-2.3-small.txt
    ```
2.  **Follow Prompts:**
    *   You will be asked if you want to print the initial page's HTML content to the console.
3.  **Check Output:**
    *   **Downloaded HTML:** Saved in the `found_html/` directory.
    *   **Intelligence Summary:** After scanning, summary files (`.txt`) for each category will be created in the `intel/` directory.

## Project Structure (After Running)

*   `found_html/`: Contains the raw HTML files downloaded during the scan.
*   `intel/`: Contains the extracted intelligence, organized into files like:
    *   `emails.txt`
    *   `scripts.txt`
    *   `comments.txt`
    *   `meta_tags.txt`
    *   `links.txt`
    *   `api_endpoints.txt`
    *   `technologies.txt`
    *   `webserver.txt` (usually empty as this info comes from HTTP headers, not HTML content in this implementation)

## Dependencies (Crates Used)

*   `reqwest` (with `blocking` feature): For making HTTP requests.
*   `tokio`: As a runtime dependency for `reqwest`.
*   `scraper`: For parsing HTML and extracting data using CSS selectors.
*   `regex`: For pattern matching (emails, comments, API paths).
*   `clap` (with `derive` feature): For parsing command-line arguments.
*   `indicatif`: For displaying a progress bar during scanning.

## Contributing

Contributions are welcome! Feel free to open issues or submit pull requests.

## License

This project is licensed under the MIT License - see the `LICENSE` file for details.
