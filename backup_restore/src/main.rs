#[tokio::main]
async fn main() {
    let problem = backup_restore::fetch_problem().await.unwrap();
    let solution = problem.solve().await;
    if let Ok(solution) = solution {
        println!("Solution: {:?}", serde_json::to_string(&solution).unwrap());
        let resp = solution.submit().await;
        println!("Response: {:?}", resp);
    } else {
        println!("Error: {:?}", solution.err());
    }
}
