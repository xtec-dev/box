mod google;
mod hetzner;
mod vbox;

#[tokio::main]
async fn main() {
    vbox::start(1).await.expect("machine");

    if true {
        let result = hetzner::config().await;
        println!("{:?}", result)
    }
}
