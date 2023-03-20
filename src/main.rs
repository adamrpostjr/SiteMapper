use clap::Parser;



/// A simple CLI tool to crawl a website and generate a sitemap
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Base Domain Name to Crawl
    #[arg(short, long)]
    domain: String,
    
    /// Path to sitemap file
    #[arg(short, long, default_value_t = String::from("./sitemap.xml"))]
    file_path: String,
}

fn main() {
    let args = Args::parse();

    let domain = args.domain;
    let file_path = args.file_path;

    println!("Domain: {}", domain);
    println!("File Path: {}", file_path);

}
