use std::fs::{remove_file, OpenOptions};
use std::io::Write;


pub fn start_sitemap(file_path: String) {
    // delete file if it exists
    let _ = remove_file(file_path.clone());


    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(file_path)
        .unwrap();

    file.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\r\n")
        .unwrap();
    file.write_all(b"<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\r\n")
        .unwrap();
}


pub fn add_url(file_path: String, url: String) {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(file_path)
        .unwrap();
    let string = format!("    <url>\n        <loc>{}</loc>\n    </url>\n", url);
    file.write_all(string.as_bytes())
        .expect("Unable to write to file");
}

pub fn end_sitemap(file_path: String) {
    let mut file = OpenOptions::new()
        .append(true)
        .open(file_path)
        .expect("Unable to open file");
    file.write_all(b"</urlset>")
        .expect("Unable to write to file");
    drop(file);
}