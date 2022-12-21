//https://kerkour.com/rust-fast-port-scanner

async fn scan(target: IpAddr, full: bool, concurrency: usize, timeout: u64) {
    let ports = stream::iter(get_ports(full));

    ports
        .for_each_concurrent(concurrency, |port| scan_port(target, port, timeout))
        .await;
}

async fn scan_port(target: IpAddr, port: u16, timeout: u64) -> Result<Option<IpAddr>> {
    let timeout = Duration::from_secs(timeout);
    let socket_address = SocketAddr::new(target.clone(), port);

    match tokio::time::timeout(timeout, TcpStream::connect(&socket_address)).await {
        Ok(Ok(_)) => println!("{}", port),
        _ => {}
    }
}