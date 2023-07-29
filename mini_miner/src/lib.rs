use std::future::Future;

use crypto_hash::{hex_digest, Algorithm};
use serde::{Deserialize, Serialize};
use utils::{get_formatted_url, UrlType};

#[derive(Debug, Deserialize, Serialize)]
pub struct Block {
    data: Vec<(String, isize)>,
    nonce: Option<u64>,
}

impl Block {
    fn hash(&self) -> String {
        let block_json = serde_json::to_string(&self).unwrap();
        let hash = hex_digest(Algorithm::SHA256, block_json.as_bytes());
        let hash_binary = hex_to_binary(&hash);
        hash_binary
    }
}

fn hex_to_binary(hex_string: &str) -> String {
    let mut binary_string = String::new();
    for c in hex_string.chars() {
        let binary = match c {
            '0' => "0000",
            '1' => "0001",
            '2' => "0010",
            '3' => "0011",
            '4' => "0100",
            '5' => "0101",
            '6' => "0110",
            '7' => "0111",
            '8' => "1000",
            '9' => "1001",
            'a' => "1010",
            'b' => "1011",
            'c' => "1100",
            'd' => "1101",
            'e' => "1110",
            'f' => "1111",
            _ => panic!("Invalid hex character"),
        };
        binary_string.push_str(binary);
    }

    binary_string
}

#[derive(Debug, Deserialize)]
pub struct Problem {
    block: Block,
    difficulty: u64,
}

pub async fn fetch_problem() -> Result<Problem, anyhow::Error> {
    let problem_url = get_formatted_url(env!("CARGO_PKG_NAME"), UrlType::Problem);
    let resp = reqwest::get(problem_url).await?.json::<Problem>().await?;
    Ok(resp)
}

impl Problem {
    pub fn solve(self) -> Option<Solution> {
        let mut block = self.block;
        let mut nonce = 0;
        loop {
            block.nonce = Some(nonce);
            if block
                .hash()
                .starts_with(&"0".repeat(self.difficulty as usize))
            {
                return Some(Solution { nonce });
            }
            nonce += 1;
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Solution {
    nonce: u64,
}

impl Solution {
    pub fn submit(&self) -> impl Future<Output = String> + '_ {
        async move {
            let solution_url = get_formatted_url(env!("CARGO_PKG_NAME"), UrlType::Solution);
            let client = reqwest::Client::new();
            let resp = client
                .post(solution_url)
                .body(serde_json::to_string(&self).unwrap())
                .send()
                .await
                .unwrap()
                .text()
                .await
                .unwrap();
            resp
        }
    }
}
