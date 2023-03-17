use reqwest::blocking::ClientBuilder;
use scraper::{Html, Selector};
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::time::Duration;

static mut SURLS: Vec<String> = Vec::new();
static mut VURLS: Vec<String> = Vec::new();
static mut UURLS: Vec<String> = Vec::new();
static mut URLS: Vec<String> = Vec::new();
static mut DOMAIN: String = String::new();
static mut FILE_PATH: String = String::new();
static mut CUR_URL: String = String::new();

fn print_status(cur_url: &str) {
    print!("{}[2J", 27 as char);
    print!("{}[1;1H", 27 as char);
    println!("------------------------------------------------------------------------");
    println!(" ");
    println!("Base Domain: {} ", unsafe { DOMAIN.clone() });
    println!("File Location: {}", unsafe { FILE_PATH.clone() });
    println!(" ");
    println!("------------------------------------------------------------------------");
    println!(" ");
    println!(
        "Unprocessed: {}     |     Processed: {}     |     Valid: {}     |     PreProcessing: {}",
        unsafe { UURLS.clone().len() },
        unsafe { VURLS.clone().len() },
        unsafe { URLS.clone().len() },
        unsafe { SURLS.clone().len() },
    );
    println!(" ");
    println!("Crawling URL: {}", cur_url);
}

fn watch() {
    // A Thread will Keep Crawling URLS and append to UURLS
    // Another Thread Will check UURLS for status and validity and appending to URLS
    // Another thread will process URLS and append to VURLS

    let mut handles = vec![];

    handles.push(std::thread::spawn(|| loop {
        if unsafe { UURLS.clone().len() } > 0 {
            let url = unsafe { UURLS.remove(0) };
            let url = clean_url(&url);
            if is_valid_url(&url)
                && check_url(&url)
                && !unsafe { VURLS.clone().contains(&url) && { URLS.clone().contains(&url) } }
            {
                unsafe { URLS.push(url.clone()) };
            }
        } else {
            continue;
        }
    }));

    handles.push(std::thread::spawn(|| loop {
        if unsafe { URLS.clone().len() } > 0 {
            let url = unsafe { URLS.remove(0) };
            unsafe { SURLS.push(url.clone()) };
            crawl_page(&url);
        } else {
            continue;
        }
    }));

    handles.push(std::thread::spawn(|| loop {
        if unsafe { SURLS.clone().len() } > 0 {
            let url = unsafe { SURLS.remove(0) };
            unsafe { VURLS.push(url.clone()) };
            add_url(&mut unsafe { FILE_PATH.clone().as_str() }, &url);
        } else {
            continue;
        }
    }));

    loop {
        std::thread::sleep(std::time::Duration::from_millis(100));
        print_status(unsafe { &CUR_URL });
    }
}

fn main() {
    let domain = std::env::args().nth(1).expect("No domain provided");
    let file_path = std::env::args().nth(2).expect("No file path provided");
    let file_path = if file_path == "" {
        "sitemap.xml"
    } else {
        &file_path
    };
    unsafe {
        DOMAIN = domain;
        FILE_PATH = file_path.to_string();
    }

    create_file(unsafe { FILE_PATH.clone().as_str() });
    start_sitemap(unsafe { FILE_PATH.clone().as_str() });
    unsafe {
        UURLS.push(DOMAIN.clone());
    }
    let url = clean_url(unsafe { &DOMAIN });
    crawl_page(&url);
    watch();

    end_sitemap(&mut unsafe { FILE_PATH.clone().as_str() });
}

fn clean_url(url: &str) -> String {
    let mut unclean_url = url.to_string();
    unclean_url = unclean_url.replace("http://", "http://");
    if !unsafe { DOMAIN.clone() }.contains("www.") {
        unclean_url = unclean_url.replace("www.", "");
    }
    if !unclean_url.ends_with("/") {
        unclean_url.push_str("/");
    }
    if unclean_url.starts_with("mailto:")
        || unclean_url.starts_with("tel:")
        || unclean_url.starts_with("text:")
    {
        return "".to_string();
    }
    if unclean_url.starts_with("/") || unclean_url.starts_with("#/") {
        unclean_url = unclean_url[1..].to_string();
        unclean_url = format!("{}{}", unsafe { DOMAIN.clone() }, unclean_url);
    }
    if !unclean_url.starts_with("http://") && !unclean_url.starts_with("https://") {
        unclean_url = format!("{}{}", unsafe { DOMAIN.clone() }, unclean_url);
    }
    unclean_url = unclean_url.trim().to_string();

    return unclean_url;
}

fn is_valid_url(url: &str) -> bool {
    let _domain = unsafe { DOMAIN.clone() };
    if url.starts_with(&_domain.as_str()) {
        return true;
    }
    return false;
}

fn crawl_page(url: &str) {
    unsafe { CUR_URL = url.clone().to_string() };

    let client = ClientBuilder::new()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("Unable to build client");
    let resp = client.get(url).send().expect("Unable to send request");
    let body = resp.text().expect("Unable to get response body");
    let document = Html::parse_document(&body);
    let selector = Selector::parse("a").expect("Unable to parse selector");
    for element in document.select(&selector) {
        let link = element.value().attr("href").unwrap_or("");
        let link = clean_url(link);
        if link != "" && !unsafe { UURLS.clone().contains(&link) } {
            if is_valid_url(&link.to_string()) {
                unsafe {
                    let uri = clean_url(link.clone().as_str());
                    UURLS.push(uri);
                }
            }
        }
    }
}

fn check_url(url: &str) -> bool {
    let client = ClientBuilder::new()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("Unable to build client");
    let resp = client.get(url).send().expect("Unable to send request");
    if resp.status().is_success() {
        return true;
    }
    return false;
}

fn create_file(path: &str) {
    if path == "" {
        File::create("sitemap.xml").expect("Unable to create file");
    }
    File::create(path).expect("Unable to create file");
}

fn start_sitemap(path: &str) {
    let mut file = OpenOptions::new()
        .append(true)
        .open(path)
        .expect("Unable to open file");
    file.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n")
        .expect("Unable to write to file");
    file.write_all(b"<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n")
        .expect("Unable to write to file");
    drop(file);
}

fn add_url(path: &str, url: &str) {
    let mut file = OpenOptions::new()
        .append(true)
        .open(path)
        .expect("Unable to open file");
    let string = format!("    <url>\n        <loc>{}</loc>\n    </url>\n", url);
    file.write_all(string.as_bytes())
        .expect("Unable to write to file");
}

fn end_sitemap(path: &mut &str) {
    let mut file = OpenOptions::new()
        .append(true)
        .open(path)
        .expect("Unable to open file");
    file.write_all(b"</urlset>")
        .expect("Unable to write to file");
    drop(file);
}
