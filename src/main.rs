use clap::Parser;

mod cli;
mod dbs;
mod scan;

fn main() {
    println!("Hello, world!");

    let args = cli::CliArg::parse();

    tracing_subscriber::fmt()
        .with_max_level({
            if args.verbose {
                tracing::Level::DEBUG
            } else {
                tracing::Level::INFO
            }
        })
        .init();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(args.threads)
        .build()
        .expect("tokio rt 构建失败");

    rt.block_on(a_main(args)).unwrap();
}

async fn a_main(args: cli::CliArg) -> anyhow::Result<()> {
    scan::work(args).await?;
    Ok(())
}
