use clap::Parser;

#[derive(Parser, Debug)]
pub struct CliArg {
    /// 线程数量
    #[arg(short = 't', long = "threads", default_value_t = 4)]
    pub threads: usize,
    /// 源站
    #[arg(short = 'u', long = "url")]
    pub url: String,
    /// 可能 ip 文件夹
    #[arg(short = 'i', long = "ip_path")]
    pub ip_path: String,
    /// 数据库文件路径
    #[arg(short = 'd', long = "db_path", default_value = "main.sqlite")]
    pub db_path: String,
    /// 是否开启 debug
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,
}
