
mod google;
mod hetzner;


#[tokio::main]
async fn main() {
    virtualbox::start(1).await.expect("machine");

    if true {
        let result = hetzner::config().await;
        println!("{:?}", result)
    }
}
