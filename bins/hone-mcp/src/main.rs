#[tokio::main]
async fn main() {
    if let Err(err) = hone_channels::mcp_bridge::run_hone_mcp_stdio().await {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
