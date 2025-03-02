use clap::Parser;

mod cli;
mod dbs;

fn main() {
    println!("Hello, world!");

    let args = cli::CliArg::parse();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(args.threads)
        .build()
        .expect("tokio rt 构建失败");

    rt.block_on(a_main(args)).unwrap();
}

async fn a_main(args: cli::CliArg) -> anyhow::Result<()> {
    let db = dbs::CoreDb::new(&args.db_path)?;
    Ok(())
}
