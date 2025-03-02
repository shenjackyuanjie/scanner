use std::{net::SocketAddr, path::Path, time::Duration};

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
) -> anyhow::Result<(bool, bool)> {
    let mut port_80 = ip;
    port_80.set_port(80);
    let mut port_443 = ip;
    port_443.set_port(443);

    let client = reqwest::ClientBuilder::new()
        .resolve_to_addrs(&src, &[port_80, port_443])
        .timeout(Duration::from_secs(10))
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

pub async fn work(args: CliArg) -> anyhow::Result<()> {
    let db = dbs::CoreDb::new(&args.db_path)?;

    let before_add_count = db.count_src()?;

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

    let check_path = Path::new(&args.compare);
    if !check_path.exists() {
        event!(Level::INFO, "对比文件不存在, 开始下载");
        let client = reqwest::Client::new();
        let res = client.get(&args.url).send().await?;
        let text = res.text().await?;
    } else if check_path.is_dir() {
        event!(Level::ERROR, "对比文件不能是目录");
        return Ok(());
    }

    let right_hash = {
      let mut hasher = blake3::Hasher::new();
        let _ = hasher.update(std::fs::read(check_path)?);
        hasher.finalize()
    };

    db.check_src()?;

    let batch_size = args.max_ip_count;

    let todo_count = db.count_src()?;

    event!(
        Level::INFO,
        "数据库内待扫ip: {}, 新增: {}",
        todo_count,
        todo_count - before_add_count
    );

    let (root_url, path) = match args.url.split_once('/') {
        Some((root, path)) => {
            // 处理掉 http:// 或者 https://
            let root = root.split_once("//").unwrap().1;
            (root.to_string(), path.to_string())
        },
        None => (args.url.clone(), "".to_string()),
    };

    if batch_size > todo_count {
        for ip in db.get_n_ip(todo_count)? {
            let ip: SocketAddr = match ip.parse() {
                Ok(ip) => ip,
                Err(e) => {
                    event!(Level::ERROR, "解析 ip 地址失败: {:?}", e);
                    continue;
                }
            }
            match scan_ip(
                ip,
                root_url.clone(),
                path.clone(),
                right_hash,
            ) {

            }
        }
    }

    db.close();

    Ok(())
}
