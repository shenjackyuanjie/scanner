use std::rc::Rc;

use tracing::{Level, event};

#[derive(Debug)]
pub struct CoreDb {
    db: Rc<rusqlite::Connection>,
}

/// 将 bool 转换为 i
///
/// true -> 1
/// false -> 0
pub fn bool_2_str(b: bool) -> &str {
    if b {
        "TRUE"
    } else {
        "FALSE"
    }
}

/// 将 i 转换为 bool
///
/// i == 0 -> false
/// else -> true
pub fn int_2_bool(i: i32) -> bool {
    !(i == 0)
}

impl CoreDb {
    pub fn new(db_path: &str) -> rusqlite::Result<Self> {
        let db = rusqlite::Connection::open(db_path)?;

        event!(Level::INFO, "已经连接到 {} 数据库", db_path);
        let slf = Self { db: Rc::new(db) };

        slf.check_table()?;

        Ok(slf)
    }

    /// table 定义:
    ///
    /// src table: 存储可能的 ip 信息
    /// ip: ip 地址 (主键) (TEXT)
    ///
    /// faild table: 存储失败的 ip 信息 (80, 443 端口不可用)
    /// ip: ip 地址 (主键) (TEXT)
    ///
    /// success table: 存储成功的 ip 信息
    /// ip: ip 地址 (主键) (TEXT)
    /// http_ok: http 请求是否成功 (INTEGER) (80 端口)
    /// https_ok: https 请求是否成功 (INTEGER) (443 端口)
    pub fn check_table(&self) -> rusqlite::Result<()> {
        self.db.execute(
            "CREATE TABLE IF NOT EXISTS src (
                ip TEXT PRIMARY KEY
            )",
            [],
        )?;

        self.db.execute(
            "CREATE TABLE IF NOT EXISTS faild (
                ip TEXT PRIMARY KEY
            )",
            [],
        )?;

        self.db.execute(
            "CREATE TABLE IF NOT EXISTS success (
                ip TEXT PRIMARY KEY,
                http_ok INTEGER,
                https_ok INTEGER
            )",
            [],
        )?;

        event!(Level::INFO, "数据库表检查完毕");

        Ok(())
    }

    /// 清理数据表
    ///
    /// 把 src 中的数据检查一遍, 如果在 faild 或者 success 表中存在, 则删除
    pub fn check_src(&self) -> rusqlite::Result<()> {
        self.db.execute(
            "
            DELETE FROM src
            WHERE ip IN (
                SELECT ip FROM faild
                UNION
                SELECT ip FROM success
            )",
            [],
        )?;

        Ok(())
    }

    /// 获取 n 个待检测的 ip
    ///
    /// 从 src 表中获取 n 个 ip, 并将这 n 个 ip 从 src 表中删除
    pub fn get_n_ip(&self, n: usize) -> rusqlite::Result<Vec<String>> {
        let mut stmt = self.db.prepare("SELECT ip FROM src LIMIT ?")?;
        let mut rows = stmt.query([&n])?;

        let mut ips = Vec::new();
        while let Some(row) = rows.next()? {
            ips.push(row.get(0)?);
        }

        event!(Level::DEBUG, "获取到 {} 个 ip", ips.len());

        Ok(ips)
    }

    /// 添加一些失败的 ip
    pub fn add_faild_ip(&self, ips: Vec<String>) -> rusqlite::Result<()> {
        let mut stmt = self.db.prepare("INSERT INTO faild (ip) VALUES (?)")?;

        for ip in ips.iter() {
            stmt.execute([&ip])?;
        }

        event!(Level::DEBUG, "添加了 {} 个失败的 ip", ips.len());

        Ok(())
    }

    pub fn add_success_ip(&self, ips: Vec<(String, bool, bool)>) -> rusqlite::Result<()> {
        let mut stmt = self.db.prepare("INSERT INTO success (ip, http_ok, https_ok) VALUES (?, ?, ?)")?;

        for ip in ips.iter() {
            stmt.execute([&ip.0, bool_2_str(ip.1), bool_2_str(ip.2)])?;
        }

        event!(Level::DEBUG, "添加了 {} 个成功的 ip", ips.len());

        Ok(())
    }
}
