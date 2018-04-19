// use fuse::{self, FileAttr, FileType, ReplyAttr, ReplyDirectory, ReplyEntry, Request};

use fuse_mt::{self, DirectoryEntry, FileAttr, FileType, RequestInfo, ResultEmpty, ResultEntry,
              ResultOpen, ResultReaddir};

use rusqlite::{types::ToSql, Connection};

use time::Timespec;

use libc::{c_int, ENOENT};

use std::ffi::OsString;
use std::path::{self, Path};

use std::sync::Mutex;

const DEFAULT_FILE_ATTR: FileAttr = FileAttr {
    size: 0,
    blocks: 0,

    atime: Timespec { sec: 1, nsec: 0 },
    mtime: Timespec { sec: 1, nsec: 0 },
    ctime: Timespec { sec: 1, nsec: 0 },
    crtime: Timespec { sec: 1, nsec: 0 },

    kind: FileType::RegularFile,

    perm: 0o755,
    nlink: 0,
    uid: 1000,
    gid: 1000,
    rdev: 0,
    flags: 0,
};

pub struct Filesystem {
    conn: Mutex<Connection>,
}

pub enum PlayPath {
    Root,
    Artist(String),
    Album {
        artist: String,
        album: String,
    },
    Song {
        artist: String,
        album: String,
        song: String,
    },
}

fn component_to_string(component: &path::Component) -> String {
    String::from(
        match component {
            path::Component::Normal(comp) => comp,
            _ => panic!("Wut"),
        }.to_str()
            .unwrap(),
    )
}

fn decompose_path(path: &Path) -> Result<PlayPath, ()> {
    let components = path.components().collect::<Vec<_>>();

    match components.len() {
        1 => Ok(PlayPath::Root),

        2 => Ok(PlayPath::Artist(component_to_string(&components[1]))),

        3 => Ok(PlayPath::Album {
            artist: component_to_string(&components[1]),
            album: component_to_string(&components[2]),
        }),

        4 => Ok(PlayPath::Song {
            artist: component_to_string(&components[1]),
            album: component_to_string(&components[2]),
            song: component_to_string(&components[3]),
        }),

        _ => Err(()),
    }
}

impl Filesystem {
    pub fn new(db: &Path) -> Self {
        info!("Opening database: {}", db.display());
        Filesystem {
            conn: Mutex::new(Connection::open(db).expect("Could not open database")),
        }
    }

    fn has_thing(&self, request: &'static str, params: &[&ToSql]) -> bool {
        let conn = self.conn.lock().unwrap();

        let mut req = conn.prepare(request).unwrap();

        req.query_row(params, |row| row.get(0): bool).unwrap()
    }

    fn has_artist(&self, artist: &str) -> bool {
        self.has_thing(
            "SELECT EXISTS(SELECT 1 FROM albums WHERE albumartist = ?);",
            &[&artist],
        )
    }

    fn has_album(&self, artist: &str, album: &str) -> bool {
        self.has_thing(
            "SELECT EXISTS(SELECT 1 FROM albums WHERE albumartist = ? AND album = ?);",
            &[&artist, &album],
        )
    }

    fn get_things(
        &self,
        request: &'static str,
        params: &[&ToSql],
        kind: FileType,
    ) -> Result<Vec<DirectoryEntry>, c_int> {
        let conn = self.conn.lock().unwrap();
        let mut req = conn.prepare(request).unwrap();

        let results = req.query_map(params, |row| DirectoryEntry {
            name: OsString::from((row.get(0): String).replace("/", "_")),
            kind,
        }).unwrap()
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| 0);

        results
    }

    fn get_artists(&self) -> Result<Vec<DirectoryEntry>, c_int> {
        self.get_things(
            "SELECT DISTINCT albumartist FROM albums",
            &[],
            FileType::Directory,
        )
    }

    fn get_albums(&self, artist: &str) -> Result<Vec<DirectoryEntry>, c_int> {
        self.get_things(
            "SELECT DISTINCT album FROM albums WHERE albumartist = ?",
            &[&artist],
            FileType::Directory,
        )
    }

    fn get_songs(&self, artist: &str, album: &str) -> Result<Vec<DirectoryEntry>, c_int> {
        self.get_things(
            "SELECT DISTINCT title FROM items WHERE albumartist = ? AND album = ?",
            &[&artist, &album],
            FileType::RegularFile,
        )
    }

}

impl fuse_mt::FilesystemMT for Filesystem {
    fn init(&self, req: RequestInfo) -> ResultEmpty {
        info!("Init: {:?}", req);
        Ok(())
    }

    fn opendir(&self, _req: RequestInfo, path: &Path, _flags: u32) -> ResultOpen {
        match decompose_path(path) {
            Err(_) => Err(ENOENT),

            Ok(PlayPath::Root) => Ok((0, 0)),

            Ok(PlayPath::Artist(artist)) => {
                if self.has_artist(&artist) {
                    Ok((0, 0))
                } else {
                    Err(ENOENT)
                }
            }

            Ok(PlayPath::Album { artist, album }) => {
                if self.has_album(&artist, &album) {
                    Ok((0, 0))
                } else {
                    Err(ENOENT)
                }
            }

            _ => Err(1),
        }
    }

    fn readdir(&self, _req: RequestInfo, path: &Path, _fh: u64) -> ResultReaddir {
        match decompose_path(path) {
            Err(_) => Err(ENOENT),

            Ok(PlayPath::Root) => self.get_artists(),
            Ok(PlayPath::Artist(artist)) => self.get_albums(&artist),
            Ok(PlayPath::Album { artist, album }) => self.get_songs(&artist, &album),

            _ => Err(1),
        }
    }

    fn getattr(&self, _req: RequestInfo, path: &Path, _fh: Option<u64>) -> ResultEntry {
        match decompose_path(path) {
            Err(_) => Err(ENOENT),

            Ok(PlayPath::Root) | Ok(PlayPath::Artist(_)) | Ok(PlayPath::Album { .. }) => Ok((
                Timespec::new(1, 0),
                FileAttr {
                    kind: FileType::Directory,
                    ..DEFAULT_FILE_ATTR
                },
            )),

            Ok(PlayPath::Song { .. }) => Ok((
                Timespec::new(1, 0),
                DEFAULT_FILE_ATTR
                ,
            )),
        }
    }
}
