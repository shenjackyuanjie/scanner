use std::{
    net::{Ipv4Addr, SocketAddr},
    path::Path,
    time::Duration,
};

use blake3::Hash;
use tracing::{Level, event};

use crate::{cli::CliArg, dbs};

/// 扫描一个指定的 ip, 返回他的 80, 443 端口是否可以获取到指定的内容
///
/// ip: 要扫描的 ip 地址
///
/// src: 原始地址(不带 http/https)
///
/// path: 要获取的路径
pub async fn scan_ip(
    ip: SocketAddr,
    src: String,
    path: String,
    right_hash: Hash,
    timeout: Duration,
) -> anyhow::Result<(bool, bool)> {
    let mut port_80 = ip;
    port_80.set_port(80);
    let mut port_443 = ip;
    port_443.set_port(443);

    let client = reqwest::ClientBuilder::new()
        .resolve_to_addrs(&src, &[port_80, port_443])
        .timeout(timeout)
        .build()?;

    match client.get(format!("https://{}/{}", src, path)).send().await {
        Ok(res) => {
            if let Ok(text) = res.text().await {
                let mut hasher = blake3::Hasher::new();
                let _ = hasher.update(text.as_bytes());
                let hash = hasher.finalize();
                if hash == right_hash {
                    return Ok((true, false));
                }
            }
        }
        Err(_) => {
            if let Ok(res) = client.get(format!("http://{}/{}", src, path)).send().await {
                if let Ok(text) = res.text().await {
                    let mut hasher = blake3::Hasher::new();
                    let _ = hasher.update(text.as_bytes());
                    let hash = hasher.finalize();
                    if hash == right_hash {
                        return Ok((false, true));
                    }
                }
            }
        }
    }
    Ok((false, false))
}

/// 核心工作函数
pub async fn work(args: CliArg) -> anyhow::Result<()> {
    let db = dbs::CoreDb::new(&args.db_path)?;

    let before_add_count = db.count_src()?;

    let check_path = Path::new(&args.compare);
    if !check_path.exists() {
        event!(Level::INFO, "对比文件不存在, 开始下载");
        let client = reqwest::Client::new();
        let res = client.get(&args.url).send().await?;
        let text = res.text().await?;
        std::fs::write(&args.compare, text)?;
    } else if check_path.is_dir() {
        event!(Level::ERROR, "对比文件不能是目录");
        return Ok(());
    }

    let right_hash = {
        let mut hasher = blake3::Hasher::new();
        let _ = hasher.update(&std::fs::read(check_path)?);
        hasher.finalize()
    };

    // 先把数据加载进来
    let src_path = Path::new(&args.ip_path);
    // 检查是不是目录
    // 如果是目录, 那么就加载目录下的所有文件
    // 如果是文件, 那么就加载文件
    if src_path.is_dir() {
        for file in src_path.read_dir()? {
            let file = file?;
            let path = file.path();
            if path.is_file() {
                match std::fs::read_to_string(path) {
                    Ok(text) => {
                        let datas = text
                            .lines()
                            .map(|d| d.trim().to_string())
                            .filter(|d| !d.is_empty())
                            .collect::<Vec<String>>();
                        event!(Level::INFO, "正在导入 {} 个 ip", datas.len());
                        db.import_ips(datas)?;
                    }
                    Err(e) => {
                        event!(Level::ERROR, "读取文件失败: {:?}", e);
                    }
                }
            }
        }
    } else {
        match std::fs::read_to_string(src_path) {
            Ok(text) => {
                let datas = text
                    .lines()
                    .map(|d| d.trim().to_string())
                    .filter(|d| !d.is_empty())
                    .collect::<Vec<String>>();
                db.import_ips(datas)?;
            }
            Err(e) => {
                event!(Level::ERROR, "读取文件失败: {:?}", e);
            }
        }
    }

    event!(Level::INFO, "导入完成");

    db.check_src()?;

    let batch_size = args.max_ip_count;

    let todo_count = db.count_src()?;

    event!(
        Level::INFO,
        "数据库内待扫ip: {}, 新增: {}",
        todo_count,
        todo_count - before_add_count
    );

    let (root_url, path) = {
        let url = args.url.clone();
        let mut url = url.split_once("://").unwrap().1;
        let mut path = "";
        if let Some((root, p)) = url.split_once('/') {
            url = root;
            path = p;
        }
        (url.to_string(), path.to_string())
    };

    let mut worker_count = 0_usize;
    let timeout = Duration::from_secs(args.timeout);

    if batch_size > todo_count {
        let ips = db.get_all_ip()?;
        let result = worker(&ips, root_url, path, right_hash, timeout).await;
        db.update_ips(&result)?;
    } else {
        let mut pool = Vec::with_capacity(args.threads);
        while db.count_src()? > 0 {
            if pool.len() < args.threads {
                let scan_ip = db.get_n_ip(args.max_ip_count.min(db.count_src()?))?;
                let root_url = root_url.clone();
                let path = path.clone();
                let handle = tokio::spawn(async move {
                    worker(&scan_ip, root_url, path, right_hash, timeout).await
                });
                worker_count += 1;
                pool.push(handle);
                event!(
                    Level::INFO,
                    "开始 worker {} (共计 {}) 进度: {:>2.3}%",
                    worker_count,
                    pool.len(),
                    (worker_count as f64 / (db.count_src()? as f64 / args.max_ip_count as f64))
                        * 100.0
                );
                tokio::time::sleep(Duration::from_millis(args.worker_interval)).await;
            } else {
                let handle = pool.remove(0);
                let result = handle.await?;
                event!(Level::INFO, "扫描了 {} 个 ip", result.len());
                db.update_ips(&result)?;
            }
            let mut i = 0;
            while i < pool.len() {
                if !pool[i].is_finished() {
                    i += 1;
                } else {
                    let handle = pool.remove(i);
                    let result = handle.await?;
                    event!(Level::INFO, "扫描了 {} 个 ip", result.len());
                    db.update_ips(&result)?;
                    i = 0;
                }
            }
        }
    }

    db.close();

    Ok(())
}

/// 工作线程
pub async fn worker(
    ips: &[String],
    root_url: String,
    path: String,
    right_hash: Hash,
    timeout: Duration,
) -> Vec<(String, bool, bool)> {
    let mut result = Vec::new();

    for ip in ips.iter() {
        let socket: Ipv4Addr = match ip.parse() {
            Ok(ip) => ip,
            Err(e) => {
                event!(Level::ERROR, "解析 ip 地址失败: {:?}", e);
                continue;
            }
        };
        let socket = SocketAddr::new(socket.into(), 0);
        match scan_ip(socket, root_url.clone(), path.clone(), right_hash, timeout).await {
            Ok((https, http)) => {
                result.push((ip.clone(), https, http));
            }
            Err(e) => {
                event!(Level::WARN, "扫描 ip 失败: {:?}", e);
            }
        }
    }

    result
}
