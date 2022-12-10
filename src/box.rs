mod google;
mod hetzner;

#[tokio::main]
async fn main() {
    if true {
        let result = hetzner::config().await;
        println!("{:?}", result)
    }
}
