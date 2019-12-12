use std::io::{Cursor, Error, Read, Seek, Write};

use openssl::hash::{Hasher, MessageDigest};

use crate::io::*;
use crate::{PBOHeader, PBO};

impl<I: Seek + Read> PBO<I> {
    /// Write the PBO file
    pub fn write<O: Write>(&self, output: &mut O) -> Result<(), Error> {
        let mut headers: Cursor<Vec<u8>> = Cursor::new(Vec::new());

        let ext_header = PBOHeader {
            filename: String::new(),
            method: 0x5665_7273,
            original: 0,
            reserved: 0,
            timestamp: 0,
            size: 0,
        };
        ext_header.write(&mut headers)?;

        if let Some(prefix) = self.extensions.get("prefix") {
            headers.write_all(b"prefix\0")?;
            headers.write_cstring(prefix)?;
        }

        for (key, value) in self.extensions.iter() {
            if key == "prefix" {
                continue;
            }

            headers.write_cstring(key)?;
            headers.write_cstring(value)?;
        }
        headers.write_cstring(String::new())?;

        let mut files_sorted: Vec<(String, &Cursor<Box<[u8]>>)> =
            self.files.iter().map(|(a, b)| (a.clone(), b)).collect();
        files_sorted.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));

        for (name, cursor) in &files_sorted {
            let header = PBOHeader {
                filename: name.clone(),
                method: 0,
                original: cursor.get_ref().len() as u32,
                reserved: 0,
                timestamp: 0,
                size: cursor.get_ref().len() as u32,
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
