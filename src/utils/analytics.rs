use std::net::SocketAddr;

use axum::http::HeaderMap;
use diesel::PgConnection;
use woothee::parser::Parser;

use crate::{models::url_analytics::NewEntry, repository::url_analytics::create};

pub fn create_analytics(addr: SocketAddr, header: &HeaderMap, conn: &mut PgConnection, short_code: String) -> bool {
    let get_header =
        |name: &str| -> String { header.get(name).and_then(|v| v.to_str().ok()).unwrap_or("").to_string() };
    let ip_address = get_header("X-Forwarded-For")
        .split(',')
        .next_back()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| addr.ip().to_string());

    let user_agent = get_header("user-agent");
    let referer = get_header("referer");

    let parser = Parser::new();
    let default = ("", "");
    let (browser, device) = match parser.parse(&user_agent) {
        Some(result) => (result.name, result.category),
        None => default,
    };
    let country_code = get_header("X-Country-Code");
    let new_entry = NewEntry {
        device: Some(device.to_string()),
        browser: Some(browser.to_string()),
        referer: Some(referer),
        short_code: Some(short_code),
        ip_address,
        user_agent: Some(user_agent),
        country_code: Some(country_code),
    };

    match create(new_entry, conn) {
        Ok(_) => true,
        Err(e) => {
            eprintln!("{e}");
            false
        },
    }
}
