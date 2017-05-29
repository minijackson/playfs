use fuse::{self, FileAttr, FileType, ReplyAttr, ReplyDirectory, ReplyEntry, Request};

use rusqlite::Connection;

use time::Timespec;

use libc::ENOENT;

use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
struct FSEntry {
    name: OsString,
    inode: u64,
    time: Timespec,
    kind: FileType,
}

pub struct Filesystem {
    conn: Connection,
    cache: Vec<(PathBuf, Vec<FSEntry>)>,
}

impl Filesystem {
    pub fn new(db: &Path) -> Self {
        info!("Opening database: {}", db.display());

        let conn = Connection::open(db).expect("Could not open database");

        let mut cache;

        {
            let mut req = conn.prepare("SELECT DISTINCT albumartist FROM albums")
                .unwrap();

            let mut count = 0;

            let artists = req.query_map(&[], |row| {
                    count += 1;
                    FSEntry {
                        name: OsString::from((row.get(0): String).replace("/", "_")),
                        inode: count,
                        time: Timespec::new(1, 0),
                        kind: FileType::Directory,
                    }
                })
                .unwrap()
                .collect::<Result<_, _>>()
                .unwrap();

            cache = vec![(PathBuf::from("/"), artists)];
            cache.reserve(count as usize + 1);
        }

        Self {
            conn: conn,
            cache: cache,
        }
    }

    pub fn get_path_content(&mut self, path: &Path) -> Option<&Vec<FSEntry>> {
        if path == Path::new("/") {
            self.cache.get(0).map(|&(_, ref entries)| entries)
        } else {
            None
        }
    }

    pub fn get_ino_content(&mut self, ino: u64) -> Option<&Vec<FSEntry>> {
        self.cache
            .get((ino - 1) as usize)
            .map(|&(_, ref entries)| entries)
    }

    pub fn path_from_ino(&self, ino: u64) -> Option<&Path> {
        self.cache
            .get(ino as usize)
            .map(|&(ref path, _)| path.as_path())
    }
}

impl fuse::Filesystem for Filesystem {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        if parent == 1 {
            match self.get_ino_content(parent)
                      .and_then(|entries| entries.iter().find(|entry| entry.name == name))
                      .map(|entry| entry.inode) {
                None => reply.error(ENOENT),
                Some(inode) => {
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
                                },
                                0)
                }
            }
        } else {
            reply.error(ENOENT)
        }
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        if ino == 1 || ino == 2 {
            reply.attr(&Timespec::new(1, 0),
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
                       })
        } else {
            reply.error(ENOENT)
        }
    }

    fn readdir(&mut self,
               _req: &Request,
               ino: u64,
               _fh: u64,
               mut offset: u64,
               mut reply: ReplyDirectory) {

        offset += 1;

        match self.get_ino_content(ino) {
            None => {
                info!("Trying to list nonexistant directory with inode: {:?}", ino);
                reply.error(ENOENT)
            }
            Some(entries) => {
                for entry in &entries[offset as usize..] {
                    if !reply.add(entry.inode, offset, entry.kind, &entry.name) {
                        debug!("Added at offset {}: {:?}", offset, entry.name);
                        offset += 1;
                    } else {
                        debug!("Fill buffer full");
                        break;
                    }
                }

                reply.ok();
            }
        }
    }
}
