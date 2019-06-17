// Modified from pbo.rs from armake2 by KoffeinFlummi

use linked_hash_map::{LinkedHashMap};
use openssl::hash::{Hasher, MessageDigest};

use std::collections::{HashMap};
use std::io::{Read, Write, Error, Cursor};

use crate::io::*;
use crate::header::PBOHeader;

pub struct PBO {
    pub files: LinkedHashMap<String, Cursor<Box<[u8]>>>,
    pub extensions: HashMap<String, String>,
    headers: Vec<PBOHeader>,
    pub checksum: Option<Vec<u8>>,
}

impl PBO {
    pub fn new() -> Self {
        Self {
            files: LinkedHashMap::new(),
            extensions: HashMap::new(),
            headers: Vec::new(),
            checksum: None,
        }
    }

    pub fn read<I: Read>(input: &mut I) -> Result<PBO, Error> {
        let mut pbo = PBO::new();
        loop {
            let header = PBOHeader::read(input)?;

            if header.method == 0x5665_7273 {
                loop {
                    let s = input.read_cstring()?;
                    if s.is_empty() { break; }
                    pbo.extensions.insert(s, input.read_cstring()?);
                }
            } else if header.filename.is_empty() {
                break;
            } else {
                pbo.headers.push(header);
            }
        }

        for header in &pbo.headers {
            let mut buffer: Box<[u8]> = vec![0; header.size as usize].into_boxed_slice();
            input.read_exact(&mut buffer)?;
            pbo.files.insert(header.filename.clone(), Cursor::new(buffer));
        }

        input.bytes().next();
        let mut checksum = vec![0; 20];
        input.read_exact(&mut checksum)?;

        Ok(pbo)
    }

    pub fn write<O: Write>(&self, output: &mut O) -> Result<(), Error> {
        let mut headers: Cursor<Vec<u8>> = Cursor::new(Vec::new());

        let ext_header = PBOHeader {
            filename: String::new(),
            method: 0x5665_7273,
            original: 0,
            reserved: 0,
            timestamp: 0,
            size: 0
        };
        ext_header.write(&mut headers)?;

        if let Some(prefix) = self.extensions.get("prefix") {
            headers.write_all(b"prefix\0")?;
            headers.write_cstring(prefix)?;
        }

        for (key, value) in self.extensions.iter() {
            if key == "prefix" { continue; }

            headers.write_cstring(key)?;
            headers.write_cstring(value)?;
        }
        headers.write_cstring(String::new())?;

        let mut files_sorted: Vec<(String,&Cursor<Box<[u8]>>)> = self.files.iter().map(|(a,b)| (a.clone(),b)).collect();
        files_sorted.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));

        for (name, cursor) in &files_sorted {
            let header = PBOHeader {
                filename: name.clone(),
                method: 0,
                original: cursor.get_ref().len() as u32,
                reserved: 0,
                timestamp: 0,
                size: cursor.get_ref().len() as u32
            };

            header.write(&mut headers)?;
        }

        let header = PBOHeader {
            method: 0,
            ..ext_header
        };
        header.write(&mut headers)?;

        let mut h = Hasher::new(MessageDigest::sha1()).unwrap();

        output.write_all(headers.get_ref())?;
        h.update(headers.get_ref()).unwrap();

        for (_, cursor) in &files_sorted {
            output.write_all(cursor.get_ref())?;
            h.update(cursor.get_ref()).unwrap();
        }

        output.write_all(&[0])?;
        output.write_all(&*h.finish().unwrap())?;

        Ok(())
    }
}
