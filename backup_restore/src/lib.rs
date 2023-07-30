pub mod docker;
use std::{
    collections::{HashSet, VecDeque},
    future::Future,
};

use serde::{Deserialize, Serialize};
use tokio_postgres::NoTls;
use utils::{get_formatted_url, UrlType};

#[derive(Debug, Deserialize)]
pub struct Problem {
    pub dump: String,
}

impl Problem {
    pub async fn solve(self) -> Result<Solution, anyhow::Error> {
        let mut db = docker::DB::new(self.dump).await;
        db.run().await;
        let psql_url = db.get_psql_url().await;
        let alive_ssns = Problem::get_alive_ssns(&psql_url).await?;

        Ok(Solution { alive_ssns })
    }

    async fn get_alive_ssns(db_url: &str) -> Result<HashSet<String>, anyhow::Error> {
        // Connect to the PostgreSQL database
        let (client, connection) = tokio_postgres::connect(db_url, NoTls).await?;

        // Spawn a task to run the connection in the background
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        // SQL query to be executed
        let sql_query = "SELECT ssn FROM criminal_records WHERE status = 'alive';";

        // Prepare the statement
        let statement = client.prepare(sql_query).await?;

        // Execute the statement and retrieve the result set
        let rows = client.query(&statement, &[]).await?;

        // Process the result set and extract 'ssn' values
        let mut ssn_values: VecDeque<String> = VecDeque::new();
        for row in rows {
            let ssn: String = row.get(0); // Assuming the SSN column is the first column (index 0)
            ssn_values.push_back(ssn);
        }

        // Convert VecDeque to Vec
        return Ok(ssn_values.into_iter().collect());
    }
}

pub async fn fetch_problem() -> Result<Problem, anyhow::Error> {
    let problem_url = get_formatted_url(env!("CARGO_PKG_NAME"), UrlType::Problem);
    let resp = reqwest::get(problem_url).await?.json::<Problem>().await?;
    Ok(resp)
}

#[derive(Debug, Serialize)]
pub struct Solution {
    pub alive_ssns: HashSet<String>,
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
