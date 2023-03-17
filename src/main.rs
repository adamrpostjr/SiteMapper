use reqwest::blocking::ClientBuilder;
use scraper::{Html, Selector};
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::time::Duration;

static mut VURLS: Vec<String> = Vec::new();
static mut UURLS: Vec<String> = Vec::new();
static mut DOMAIN: String = String::new();
static mut FILE_PATH: String = String::new();

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
        "Unprocessed: {}   |   Processed: {}",
        unsafe { UURLS.clone().len() },
        unsafe { VURLS.clone().len() }
    );
    println!(" ");
    println!("Crawling URL: {}", cur_url);
}

fn clean_url(url: &str) -> String {
    let mut unclean_url = url.to_string();
    unclean_url = unclean_url.replace("http://", "http://");
    if !unclean_url.ends_with("/") {
        unclean_url.push_str("/");
    }

    if unclean_url.starts_with("mailto:") || unclean_url.starts_with("tel:") {
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
    let url = url.replace("www.", "");
    if url.starts_with(&_domain.as_str()) {
        return true;
    } else {
        return false;
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("No domain given");
        std::process::exit(1);
    }
    let base_domain = args[1].clone();
    let clean_domain = clean_url(&base_domain);
    unsafe { DOMAIN = clean_domain.clone() };

    if args.len() == 3 {
        unsafe { FILE_PATH = args[2].clone() };
    } else {
        unsafe { FILE_PATH = "sitemap.xml".to_string() };
    }

    create_file(&mut unsafe { FILE_PATH.clone().as_str() });
    start_sitemap(&mut unsafe { FILE_PATH.clone().as_str() });

    crawl(&clean_domain);
    watch();

    while unsafe { UURLS.len() } > 0 {
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
    end_sitemap(&mut unsafe { FILE_PATH.clone().as_str() });
}

fn crawl(url: &str) {
    let _clean_url = clean_url(url);
    print_status(&_clean_url);

    let client = ClientBuilder::new()
        .user_agent("BOT/Rust Site Mapper")
        .timeout(Duration::from_secs(10))
        .build()
        .expect("Unable to build client");

    let res = match client.get(&_clean_url).send() {
        Ok(res) => res,
        Err(_) => return,
    };

    if res.status().is_client_error() || res.status().is_server_error() {
        return;
    }
    if !res
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap()
        .contains("text/html")
    {
        return;
    }
    let body = res.text().expect("Unable to get response body");
    let document = Html::parse_document(&body);
    let links = Selector::parse("a").expect("Unable to parse selector");

    for link in document.select(&links) {
        let link = link.value().attr("href").unwrap_or("");
        let _clean_link = clean_url(link);
        unsafe { UURLS.push(_clean_link.clone()) };
    }
}

fn watch() {
    loop {
        if unsafe { UURLS.len() } > 0 {
           let url = unsafe { UURLS.remove(0) };
            if unsafe { VURLS.contains(&url) } {
                continue;
            } else if is_valid_url(&url) {
                unsafe { VURLS.push(url.clone()) };
                add_url(&mut unsafe { FILE_PATH.clone().as_str() }, &url);
                crawl(&url)
            }
        }
    }
}

// function to create a file at a given path
fn create_file(path: &str) {
    if path == "" {
        File::create("sitemap.xml").expect("Unable to create file");
    }
    File::create(path).expect("Unable to create file");
}

// start sitemap file
fn start_sitemap(path: &mut &str) {
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

// fn to add a url to the sitemap
fn add_url(path: &mut &str, url: &str) {
    let mut file = OpenOptions::new()
        .append(true)
        .open(path)
        .expect("Unable to open file");
    file.write_all(b"    <url>\n")
        .expect("Unable to write to file");
    file.write_all(b"        <loc>")
        .expect("Unable to write to file");
    file.write_all(url.as_bytes())
        .expect("Unable to write to file");
    file.write_all(b"</loc>\n")
        .expect("Unable to write to file");
    file.write_all(b"    </url>\n")
        .expect("Unable to write to file");
    drop(file);
}

// fn to end the sitemap
fn end_sitemap(path: &mut &str) {
    // open the file for appending
    let mut file = OpenOptions::new()
        .append(true)
        .open(path)
        .expect("Unable to open file");
    file.write_all(b"</urlset>")
        .expect("Unable to write to file");
    drop(file);
}
