use std::rc::Rc;

#[derive(Debug)]
pub struct CoreDb {
    db: Rc<rusqlite::Connection>,
}

impl CoreDb {
    pub fn new(db_path: &str) -> rusqlite::Result<Self> {
        let db = rusqlite::Connection::open(db_path)?;
        Ok(Self { db: Rc::new(db) })
    }

    pub fn check_table(&self) -> rusqlite::Result<()> {
        self.db.execute(
            "CREATE TABLE IF NOT EXISTS ip_list (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                ip TEXT NOT NULL,
                port INTEGER NOT NULL,
                type TEXT NOT NULL,
                source TEXT NOT NULL,
                check_time INTEGER NOT NULL
            )",
            [],
        )?;
        Ok(())
    }
}
