use clap::Parser;
use reqwest::blocking::ClientBuilder;
use scraper::{Html, Selector};
use std::time::Duration;

// import sitemap.rs
mod sitemap;
use sitemap::{add_url, end_sitemap, start_sitemap};
mod helpers;
use helpers::{clean_url, is_valid_url};

// global variables for found urls
static mut FOUND_URLS: Vec<String> = Vec::new(); // new urls will be tossed here
static mut VISITED_URLS: Vec<String> = Vec::new(); // once a url has been scanned for links it will be placed here
static mut PROCESSED_URLS: Vec<String> = Vec::new(); // once a url from VISITED_URLS has been processed it will be placed here

/// A simple CLI tool to crawl a website and generate a sitemap
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Base Domain Name to Crawl
    #[arg(short, long)]
    domain: String,

    /// Path to sitemap file
    #[arg(short, long, default_value_t = String::from("./maps/sitemap.xml"))]
    file_path: String,

    /// Amount of time in seconds to wait before timing out a request
    #[arg(short, long, default_value_t = 30)]
    timeout: u64,

    /// Number of threads to run on
    #[arg(long, default_value_t = 1)]
    threads: u64,
}

fn main() {
    let args = Args::parse();

    let domain = args.domain;
    let file_path = args.file_path;
    let threads = args.threads;

    let domain = clean_url(&domain, domain.clone());
    crawl_url(domain.clone());
    start_sitemap(file_path.clone());

    let threads = threads + 2;

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads as usize)
        .build()
        .unwrap();

    pool.install(|| {
        rayon::scope(|s| {
            for _ in 0..threads - 2 {
                s.spawn(|_| {
                    crawl_urls();
                });
            }
            s.spawn(|_| {
                std::thread::sleep(Duration::from_secs(30));
                loop {
                    std::thread::sleep(Duration::from_millis(300));
                    process_urls(file_path.clone());
                }
            });
            s.spawn(|_| {
                loop {
                    print_vars();
                    std::thread::sleep(Duration::from_millis(300));
                }
            });
        });
    });
    end_sitemap(file_path);
}

fn crawl_url(url: String) {
    let args = Args::parse();
    let domain = args.domain;

    let url = clean_url(&url, domain.clone());
    if !is_valid_url(url.clone(), domain.clone())
        && unsafe { !VISITED_URLS.contains(&url) }
        && unsafe { !PROCESSED_URLS.contains(&url) }
    {
        return;
    }

    unsafe { VISITED_URLS.push(url.clone()) }
    let version = env!("CARGO_PKG_VERSION");
    let user_agent = format!("BOT/SiteMapper/{}", version);
    let client = ClientBuilder::new()
        .user_agent(user_agent)
        .timeout(Duration::from_secs(10))
        .build();

    if client.is_err() {
        return;
    }

    let client = client.unwrap();

    let res = client.get(&url).send();
    if res.is_err() {
        return;
    }

    let res = res.unwrap();

    let body = res.text().unwrap();
    let document = Html::parse_document(&body);
    let selector = Selector::parse("a").unwrap();
    for element in document.select(&selector) {
        let link = element.value().attr("href").unwrap_or("");
        let link = clean_url(link, domain.clone());
        if !is_valid_url(link.clone(), domain.clone())
            && unsafe { !FOUND_URLS.contains(&link) }
            && unsafe { !VISITED_URLS.contains(&link) }
            && unsafe { !PROCESSED_URLS.contains(&link) }
        {
            continue;
        }
        unsafe { FOUND_URLS.push(link.to_string()) }
    }
    // close out of client
    drop(client);
}

fn process_urls(file_path: String) {
    if unsafe { VISITED_URLS.len() > 0 } {
        let url = unsafe { VISITED_URLS.remove(0) };
        if url.is_empty() {
            drop(url);
            return;
        }
        if unsafe { !PROCESSED_URLS.contains(&url) } {
            unsafe { PROCESSED_URLS.push(url.clone()) }
            add_url(file_path, url)
        } else {
            drop(url);
        }
        return;
    }
}

// watcher function, that watches for FOUND_URLS and crawls them
fn crawl_urls() {
    loop {
        if unsafe { FOUND_URLS.len() > 0 } {
            let url = unsafe { FOUND_URLS.remove(0) };
            crawl_url(url.clone());
        }else{
            std::thread::sleep(Duration::from_secs(1));
            if unsafe { FOUND_URLS.len() == 0 } {
                break;
            }
        }
    }
}

// function to print out the variables for debugging
fn print_vars() {
    // clear out console
    print!("{}[2J", 27 as char);
    // top left corner
    print!("{}[1;1H", 27 as char);

    println!("FOUND_URLS: {:?}", unsafe { FOUND_URLS.len() });
    println!("VISITED_URLS: {:?}", unsafe { VISITED_URLS.len() });
    println!("PROCESSED_URLS: {:?}", unsafe { PROCESSED_URLS.len() });
}
