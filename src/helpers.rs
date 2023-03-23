use url::{Url};

pub fn clean_url(url: &str, domain: String) -> String {
    let mut unclean_url = url.to_string();
    unclean_url = unclean_url.replace("http://", "http://");
    if  !domain.clone().contains("www.") {
        unclean_url = unclean_url.replace("www.", "");
    }
    if unclean_url.starts_with("mailto:")
        || unclean_url.starts_with("tel:")
        || unclean_url.starts_with("text:")
    {
        return "".to_string();
    }
    if unclean_url.starts_with("/") || unclean_url.starts_with("#/") {
        unclean_url = unclean_url[1..].to_string();
        unclean_url = format!("{}{}", domain.clone(), unclean_url);
    }
    if !unclean_url.starts_with("http://") && !unclean_url.starts_with("https://") {
        unclean_url = format!("{}{}", domain.clone(), unclean_url);
    }
    if unclean_url.ends_with("#") {
        unclean_url = unclean_url[..unclean_url.len() - 1].to_string();
    }
    unclean_url = unclean_url.trim().to_string();
    return unclean_url;
}

pub fn is_valid_url(url: String, domain: String) -> bool {

    // try catch
    
    
    let url = Url::parse(&url);
    if url.is_err() {
        return false;
    }
    let url = url.unwrap();

    // if 

    let url_host = url.host().unwrap();
    let domain = Url::parse(&domain).unwrap();
    let domain_host = domain.host().unwrap();

    if url_host != domain_host {
        return false;
    }

    return true;
}
