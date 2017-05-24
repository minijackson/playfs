use fuse_mt::{DirectoryEntry, FileAttr, FilesystemMT, FileType, ResultEmpty, ResultEntry,
              ResultGetattr, ResultOpen, ResultReaddir, RequestInfo};

use rusqlite::Connection;

use time::Timespec;

use libc::ENOENT;

use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::sync::Mutex;

pub struct Filesystem {
    conn: Mutex<Connection>,
}

impl Filesystem {
    pub fn new(db: &Path) -> Self {
        info!("Opening database: {}", db.display());
        Self { conn: Mutex::new(Connection::open(db).expect("Could not open database")) }
    }
}

impl FilesystemMT for Filesystem {
    fn init(&self, _req: RequestInfo) -> ResultEmpty {
        debug!("Init!");
        Ok(())
    }

    fn lookup(&self, _req: RequestInfo, parent: &Path, name: &OsStr) -> ResultEntry {
        if parent == Path::new("/") && name == "hello.txt" {
            Ok((Timespec::new(0, 0),
                FileAttr {
                    ino: 2,
                    size: 0,
                    blocks: 0,
                    atime: Timespec::new(1, 0),
                    mtime: Timespec::new(1, 0),
                    ctime: Timespec::new(1, 0),
                    crtime: Timespec::new(1, 0),
                    kind: FileType::RegularFile,
                    perm: 0o755,
                    nlink: 0,
                    uid: 1000,
                    gid: 1000,
                    rdev: 0,
                    flags: 0,
                }))
        } else {
            Err(ENOENT)
        }
    }

    fn getattr(&self, _req: RequestInfo, path: &Path, _fh: Option<u64>) -> ResultGetattr {
        if path == Path::new("/") {
            Ok((Timespec::new(1, 0),
                FileAttr {
                    ino: 1,
                    size: 0,
                    blocks: 0,
                    atime: Timespec::new(1, 0),
                    mtime: Timespec::new(1, 0),
                    ctime: Timespec::new(1, 0),
                    crtime: Timespec::new(1, 0),
                    kind: FileType::Directory,
                    perm: 0o755,
                    nlink: 0,
                    uid: 1000,
                    gid: 1000,
                    rdev: 0,
                    flags: 0,
                }))
        } else if path == Path::new("/hello.txt") {
            Ok((Timespec::new(0, 0),
                FileAttr {
                    ino: 2,
                    size: 0,
                    blocks: 0,
                    atime: Timespec::new(1, 0),
                    mtime: Timespec::new(1, 0),
                    ctime: Timespec::new(1, 0),
                    crtime: Timespec::new(1, 0),
                    kind: FileType::RegularFile,
                    perm: 0o644,
                    nlink: 0,
                    uid: 1000,
                    gid: 1000,
                    rdev: 0,
                    flags: 0,
                }))
        } else {
            Err(ENOENT)
        }
    }

    fn opendir(&self, _req: RequestInfo, path: &Path, _flags: u32) -> ResultOpen {
        if path == Path::new("/") {
            Ok((0, 0))
        } else {
            Err(ENOENT)
        }
    }

    fn readdir(&self, _req: RequestInfo, path: &Path, _fh: u64) -> ResultReaddir {

        let conn = self.conn.lock().unwrap();
        let mut req = conn.prepare("SELECT albumartist FROM albums").unwrap();
        let results = req.query_map(&[], |row| {
                DirectoryEntry {
                    name: OsString::from(row.get(0): String),
                    kind: FileType::Directory,
                }
            })
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        Ok(results)
    }
}
