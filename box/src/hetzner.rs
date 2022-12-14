use anyhow::Result;
use hcloud::apis::configuration::Configuration;
use hcloud::apis::servers_api;

pub async fn _config() -> Result<()> {
    let mut configuration = Configuration::new();
    configuration.bearer_access_token = Some("YOUR_HCLOUD_API_TOKEN".to_string());

    let servers = servers_api::list_servers(&configuration, Default::default())
        .await?
        .servers;

    for server in servers {
        println!("{:?}", server);
    }

    Ok(())
}
