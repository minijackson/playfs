use fuse;
use fuse::{
    FileAttr,
    FileType,
    ReplyAttr,
    ReplyDirectory,
    ReplyEntry,
    Request,
};

use rusqlite::Connection;

use time::Timespec;

use libc::ENOENT;

use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

pub struct Filesystem {
    conn: Connection,
    //cache: HashMap<PathBuf, Vec<DirectoryEntry>>,
}

impl Filesystem {
    pub fn new(db: &Path) -> Self {
        info!("Opening database: {}", db.display());

        Self {
            conn: Connection::open(db).expect("Could not open database"),
            //cache: HashMap::new(),
        }
    }

    //pub fn get_content(&mut self, path: &Path) -> Vec<DirectoryEntry> {
        //if path == Path::new("/") {
            //self.cache
                //.entry(path.to_owned())
                //.or_insert_with(|| {
                    //let conn = self.conn.lock().unwrap();
                    //let mut req = conn.prepare("SELECT DISTINCT albumartist FROM albums")
                        //.unwrap();
                    //req.query_map(&[], |row| {
                            //DirectoryEntry {
                                //name: OsString::from(row.get(0): String),
                                //kind: FileType::Directory,
                            //}
                        //})
                        //.unwrap()
                        //.collect::<Result<_, _>>()
                        //.unwrap()
                //})
            //.clone()
        //} else {
            //vec![]
        //}
    //}
}

impl fuse::Filesystem for Filesystem {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        if parent == 1 {
            reply.entry(&Timespec::new(0, 0),
            &FileAttr {
                ino: 2,
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
            }, 0)
        } else {
            reply.error(ENOENT)
        }
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        if ino == 1 || ino == 2 {
            reply.attr(&Timespec::new(1, 0),
            &FileAttr {
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
            })
        } else {
            reply.error(ENOENT)
        }
    }

    //fn opendir(&self, _req: RequestInfo, path: &Path, _flags: u32) -> ResultOpen {
        //if path == Path::new("/") {
            //Ok((0, 0))
        //} else {
            //Err(ENOENT)
        //}
    //}

    fn readdir(&mut self, _req: &Request, ino: u64, _fh: u64, offset: u64, mut reply: ReplyDirectory) {

        if ino == 1 && offset == 0 {
            let mut req = self.conn.prepare("SELECT DISTINCT albumartist FROM albums")
                .unwrap();
            let mut results = req.query(&[]).unwrap();

            let mut offset = offset;

            while let Some(row) = results.next() {
                let row = row.unwrap();
                reply.add(2, offset, FileType::Directory, OsString::from(row.get(0): String));
                offset += 1;
            }

            //let results = req.query_map(&[], |row| {
                    //DirectoryEntry {
                        //name: OsString::from(row.get(0): String),
                        //kind: FileType::Directory,
                    //}
                //})
                //.unwrap()
                //.collect::<Result<_, _>>()
                //.each(||);
        }

        reply.ok();
    }
}
