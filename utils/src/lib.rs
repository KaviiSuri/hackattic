use dotenv::dotenv;
use std::fmt;

fn get_access_token() -> String {
    std::env::var("HACKATTIC_ACCESS_TOKEN").expect("HACKATTIC_ACCESS_TOKEN not set")
}

pub enum UrlType {
    Problem,
    Solution,
}

impl fmt::Display for UrlType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            UrlType::Problem => write!(f, "problem"),
            UrlType::Solution => write!(f, "solve"),
        }
    }
}

pub fn get_formatted_url(problem_name: &str, url_type: UrlType) -> String {
    dotenv().ok();
    let access_token = get_access_token();
    format!(
        "https://hackattic.com/challenges/{problem_name}/{url_type}?access_token={access_token}"
    )
}
