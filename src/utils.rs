use axum::http::header::{self, HeaderMap};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use time::{Duration, OffsetDateTime};

use noteserver::auth;

// Necessary for cookie expiry to persist when passing them back to the client,
// because the client sends no expiry dates with the request
pub fn refresh_cookie_expiry(jar: &CookieJar) -> CookieJar {
    let mut res = jar.clone();
    for cookie in jar.iter() {
        res = res.add(make_expiring_cookie(cookie.name(), cookie.value()));
    }
    res
}

pub fn make_expiring_cookie<'a>(name: &str, val: &str) -> Cookie<'a> {
    let expires_at = OffsetDateTime::now_utc() + Duration::days(30);
    Cookie::build((name.to_string(), val.to_string()))
        .expires(expires_at)
        .http_only(true)
        .path("/")
        .build()
}

pub fn get_cookie_from_jar(jar: &CookieJar, cookie_name: &str) -> Option<String> {
    let cookie_gotten = jar.get(cookie_name).cloned();
    cookie_gotten.map(|cookie| cookie.value().to_string())
}

pub fn get_token_cookie_name(dir: &str) -> String {
    format!("tok-{}", dir)
}

pub fn is_dark_theme(jar: &CookieJar) -> bool {
    let darktheme = get_cookie_from_jar(jar, "theme").unwrap_or(String::from("light"));
    darktheme == "dark"
}

pub fn valid_auth(headers: &HeaderMap) -> bool {
    match headers.get(header::AUTHORIZATION).cloned() {
        Some(val) => match val.to_str() {
            Ok(pw) => auth::is_authorized(pw),
            _ => false,
        },
        None => false,
    }
}

pub fn make_redirect_headers(to: String) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(header::LOCATION, to.parse().unwrap());
    headers
}
