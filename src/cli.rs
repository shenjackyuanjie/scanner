use clap::Parser;

#[derive(Parser, Debug)]
pub struct CliArg {
    /// 线程数量
    #[arg(short = 't', long = "threads", default_value_t = 4)]
    pub threads: usize,
    /// 源站, 包含 https:// 或 http://
    #[arg(short = 'u', long = "url")]
    pub url: String,
    /// 可能 ip 文件夹/文件
    #[arg(short = 'i', long = "ip_path")]
    pub ip_path: String,
    /// 数据库文件路径
    #[arg(short = 'd', long = "db_path", default_value = "main.sqlite")]
    pub db_path: String,
    /// 是否开启 debug
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,
    /// 单次搜索的最大 ip 数量
    #[arg(short = 'm', long = "max_ip", default_value_t = 100)]
    pub max_ip_count: usize,
    /// 对比文件路径
    #[arg(short = 'c', long = "compare", default_value = "check.src")]
    pub compare: String,
    /// 超时时间 (默认 1s)
    #[arg(short = 'o', long = "timeout", default_value_t = 1)]
    pub timeout: u64,
    /// 每次 搜索 worker 启动间隔 (ms)
    #[arg(short = 'w', long = "worker_interval", default_value_t = 200)]
    pub worker_interval: u64,
}
