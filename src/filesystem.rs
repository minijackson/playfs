use fuse;
use fuse::{FileAttr, FileType, ReplyAttr, ReplyDirectory, ReplyEntry, Request};

use rusqlite::Connection;

use time::Timespec;

use libc::ENOENT;

use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

struct FSEntry {
    name: OsString,
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

        let artists;

        {
            let mut req = conn.prepare("SELECT DISTINCT albumartist FROM albums")
                .unwrap();

            artists = req.query_map(&[], |row| {
                    FSEntry {
                        name: OsString::from((row.get(0): String).replace("/", "_")),
                        time: Timespec::new(1, 0),
                        kind: FileType::Directory,
                    }
                })
                .unwrap()
                .collect::<Result<_, _>>()
                .unwrap();
        }

        Self {
            conn: conn,
            cache: vec![(PathBuf::from("/"), artists)],
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
            self.cache.get((ino - 1) as usize).map(|&(_, ref entries)| entries)
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

    fn readdir(&mut self,
               _req: &Request,
               ino: u64,
               _fh: u64,
               mut offset: u64,
               mut reply: ReplyDirectory) {

        offset += 1;

        match self.get_ino_content(ino) {
            None => {
                info!("No such directory to list");
                reply.error(ENOENT)
            }
            Some(entries) => {
                for entry in &entries[offset as usize..] {
                    if !reply.add(offset + 2, offset, entry.kind, &entry.name) {
                        info!("Added at offset {}: {:?}", offset, entry.name);
                        offset += 1;
                    } else {
                        info!("Fill buffer full");
                        break;
                    }
                }

                info!("Replying ok");
                reply.ok();
            }
        }
    }
}
