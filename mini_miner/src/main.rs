#[tokio::main]
async fn main() {
    let problem = mini_miner::fetch_problem().await.unwrap();
    println!("{:?}", problem);
    let solution = problem.solve().expect("no solution found");
    let resp = solution.submit().await;
    println!("{}", resp);
}
