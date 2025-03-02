use tracing::{Level, event};

#[derive(Debug)]
pub struct CoreDb {
    db: rusqlite::Connection,
}

/// 将 bool 转换为 i
///
/// true -> 1
/// false -> 0
pub fn bool_2_str(b: bool) -> &'static str {
    if b { "TRUE" } else { "FALSE" }
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
        let slf = Self { db };

        slf.check_table()?;
        slf.check_src()?;

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
    /// http_ok: http 请求是否成功 (BOOLEAN) (80 端口)
    /// https_ok: https 请求是否成功 (BOOLEAN) (443 端口)
    pub fn check_table(&self) -> rusqlite::Result<()> {
        // src
        self.db.execute(
            "CREATE TABLE IF NOT EXISTS src (
                ip TEXT PRIMARY KEY ON CONFLICT REPLACE
            )",
            [],
        )?;
        // index for src
        self.db.execute(
            "
            CREATE UNIQUE INDEX IF NOT EXISTS src_ip_index
            ON src (ip)",
            [],
        )?;

        // faild
        self.db.execute(
            "CREATE TABLE IF NOT EXISTS faild (
                ip TEXT PRIMARY KEY ON CONFLICT REPLACE
            )",
            [],
        )?;
        // index for faild
        self.db.execute(
            "CREATE UNIQUE INDEX IF NOT EXISTS faild_ip_index
            ON faild (ip)",
            [],
        )?;

        self.db.execute(
            "CREATE TABLE IF NOT EXISTS success (
                ip TEXT PRIMARY KEY ON CONFLICT REPLACE,
                http_ok BOOLEAN NOT NULL,
                https_ok BOOLEAN NOT NULL
            )",
            [],
        )?;
        // index for success
        self.db.execute(
            "CREATE UNIQUE INDEX IF NOT EXISTS success_ip_index
            ON success (ip)",
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

    /// 获取所有的 ip
    pub fn get_all_ip(&self) -> rusqlite::Result<Vec<String>> {
        let mut stmt = self.db.prepare("SELECT ip FROM src")?;
        let mut rows = stmt.query([])?;

        let mut ips = Vec::new();
        while let Some(row) = rows.next()? {
            ips.push(row.get(0)?);
        }

        event!(Level::DEBUG, "获取到 {} 个 ip", ips.len());

        Ok(ips)
    }

    /// 添加一些失败的 ip
    pub fn add_faild_ip(&self, ips: &[String]) -> rusqlite::Result<()> {
        let mut stmt = self.db.prepare("INSERT INTO faild (ip) VALUES (?)")?;

        for ip in ips.iter() {
            stmt.execute([&ip])?;
        }

        event!(Level::DEBUG, "添加了 {} 个失败的 ip", ips.len());

        Ok(())
    }

    /// 添加一些成功的 ip
    pub fn add_success_ip(&self, ips: &[(String, bool, bool)]) -> rusqlite::Result<()> {
        let mut stmt = self
            .db
            .prepare("INSERT INTO success (ip, http_ok, https_ok) VALUES (?, ?, ?)")?;

        for ip in ips.iter() {
            stmt.execute([&ip.0, bool_2_str(ip.1), bool_2_str(ip.2)])?;
        }

        event!(Level::DEBUG, "添加了 {} 个成功的 ip", ips.len());

        Ok(())
    }

    /// 更新 ip
    pub fn update_ip(&self, ip: &str, http_ok: bool, https_ok: bool) -> rusqlite::Result<()> {
        if http_ok || https_ok {
            self.add_success_ip(&[(ip.to_string(), http_ok, https_ok)])?;
        } else {
            self.add_faild_ip(&[ip.to_string()])?;
        }
        Ok(())
    }

    /// 更新一大堆 ip
    pub fn update_ips(&self, ips: &[(String, bool, bool)]) -> rusqlite::Result<()> {
        let success = ips
            .iter()
            .filter(|(_, http, https)| *http || *https)
            .map(|(ip, a, b)| (ip.to_string(), *a, *b))
            .collect::<Vec<_>>();
        let faild = ips
            .iter()
            .filter(|(_, http, https)| !(*http || *https))
            .map(|(ip, _, _)| ip.to_string())
            .collect::<Vec<_>>();
        self.add_faild_ip(&faild)?;
        self.add_success_ip(&success)?;
        Ok(())
    }

    /// 导入 ip
    pub fn import_ips(&self, ips: Vec<String>) -> rusqlite::Result<()> {
        let mut stmt = self.db.prepare("INSERT INTO src (ip) VALUES (?) ")?;

        for ip in ips.iter() {
            if let Err(e) = stmt.execute([&ip]) {
                event!(Level::WARN, "插入 ip 失败: {:?}", e);
            }
        }

        event!(Level::INFO, "添加了 {} 个 ip", ips.len());

        Ok(())
    }

    /// 导出成功的 ip
    pub fn export_success(&self) -> rusqlite::Result<(Vec<String>, Vec<String>)> {
        let mut stmt = self
            .db
            .prepare("SELECT ip, http_ok, https_ok FROM success")?;
        let mut rows = stmt.query([])?;

        let mut http_ips = Vec::new();
        let mut https_ips = Vec::new();

        while let Some(row) = rows.next()? {
            let ip: String = row.get(0)?;
            let http_ok: bool = int_2_bool(row.get(1)?);
            let https_ok: bool = int_2_bool(row.get(2)?);

            if http_ok {
                http_ips.push(ip.clone());
            }
            if https_ok {
                https_ips.push(ip.clone());
            }
        }

        event!(Level::DEBUG, "导出了 {} 个 http 成功的 ip", http_ips.len());

        Ok((http_ips, https_ips))
    }

    pub fn count_src(&self) -> rusqlite::Result<usize> {
        let mut stmt = self.db.prepare("SELECT COUNT(*) FROM src")?;
        let mut rows = stmt.query([])?;

        let count: i64 = rows.next()?.unwrap().get(0)?;

        Ok(count as usize)
    }

    pub fn close(self) {
        self.db.close().expect("db 关闭失败?");
    }
}
