use fuse;

use rusqlite::Connection;

use std::path::Path;

pub struct Filesystem {
    conn: Connection
}

impl Filesystem {

    pub fn new(db: &Path) -> Self {
        info!("Opening database: {}", db.display());
        Self {
            conn: Connection::open(db).expect("Could not open database")
        }
    }

}

impl fuse::Filesystem for Filesystem {

}
