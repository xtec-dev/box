mod google;
mod hetzner;
mod vbox;

#[tokio::main]
async fn main() {
    vbox::ova_import().await;

    if true {
        let result = hetzner::config().await;
        println!("{:?}", result)
    }
}
