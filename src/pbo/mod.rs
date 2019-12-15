use std::collections::HashMap;
use std::io::{Cursor, Read, Seek, SeekFrom};

use crate::PBOHeader;

mod read;
mod write;

#[derive(Default)]
pub struct PBO<I: Seek + Read> {
    pub extensions: HashMap<String, String>,
    pub extension_order: Vec<String>,
    pub headers: Vec<PBOHeader>,
    checksum: Option<Vec<u8>>,
    files: HashMap<String, Cursor<Box<[u8]>>>,

    input: Option<I>,
    blob_start: u64,
    read_cache: bool,
}

impl<I: Seek + Read> PBO<I> {
    pub fn new() -> Self {
        Self {
            extensions: HashMap::new(),
            extension_order: Vec::new(),
            headers: Vec::new(),
            checksum: None,
            files: HashMap::new(),

            input: None,
            blob_start: 0,
            read_cache: false,
        }
    }

    /// Enable or disable read caching
    ///
    /// When enabled, files will be stored in RAM after being read
    /// future reads will be pulled from RAM
    pub fn set_cache_enabled(&mut self, enable: bool) -> bool {
        let ret = self.read_cache;
        self.read_cache = enable;
        ret
    }

    /// Return true if the read cache is enabled
    pub fn cache_enabled(&self) -> bool {
        self.read_cache
    }

    /// Clears the read cache in RAM
    pub fn clear_cache(&mut self) {
        let files = self
            .files(true)
            .iter()
            .map(|h| h.filename.clone())
            .collect::<Vec<String>>();
        let mut remove = Vec::new();
        for n in self.files.keys() {
            if !files.contains(&n) {
                remove.push(n.to_string());
            }
        }
        for n in remove {
            self.files.remove(&n);
        }
    }

    /// A list of filenames in the PBO
    pub fn files(&self, disk_only: bool) -> Vec<PBOHeader> {
        let mut filenames = Vec::new();
        for h in &self.headers {
            filenames.push(h.clone());
        }
        if !disk_only {
            for (n, c) in &self.files {
                filenames.push(PBOHeader {
                    filename: n.clone(),
                    method: 0,
                    original: c.get_ref().len() as u32,
                    reserved: 0,
                    timestamp: 0,
                    size: c.get_ref().len() as u32,
                });
            }
        }
        filenames
    }

    /// Get files in alphabetical order
    pub fn files_sorted(&self, disk_only: bool) -> Vec<PBOHeader> {
        let mut sorted = self.files(disk_only);
        sorted.sort_by(|a, b| a.filename.to_lowercase().cmp(&b.filename.to_lowercase()));
        sorted
    }

    /// Finds a header if it exists
    pub fn header(&mut self, filename: &str) -> Option<PBOHeader> {
        for header in &self.headers {
            if header.filename == filename {
                return Some(header.clone());
            }
        }
        None
    }

    /// Retreives a file from a PBO
    pub fn retrieve(&mut self, filename: &str) -> Option<Cursor<Box<[u8]>>> {
        if let Some(f) = self.files.get(filename) {
            return Some(f.clone());
        } else {
            let input = self.input.as_mut().unwrap();
            (*input)
                .seek(SeekFrom::Start(self.blob_start))
                .unwrap();
            for h in &self.headers {
                if h.filename == filename {
                    let mut buffer: Box<[u8]> = vec![0; h.size as usize].into_boxed_slice();
                    (*input).read_exact(&mut buffer).unwrap();
                    if self.read_cache {
                        self.files
                            .insert(filename.to_string(), Cursor::new(buffer.clone()));
                    }
                    return Some(Cursor::new(buffer));
                } else {
                    (*input).seek(SeekFrom::Current(h.size as i64)).unwrap();
                }
            }
        }
        None
    }

    /// Removes a file, returning it if it existed
    pub fn remove(&mut self, filename: &str) -> Option<Cursor<Box<[u8]>>> {
        self.files.remove(filename)
    }

    /// Adds or updates a file to the PBO, returns the old file if it existed
    pub fn add(&mut self, filename: &str, file: Cursor<Box<[u8]>>) -> Option<Cursor<Box<[u8]>>> {
        self.files.insert(filename.to_string(), file)
    }
}
