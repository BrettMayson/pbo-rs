use std::io::{Cursor, Error, Read, Seek, Write};

use openssl::hash::{Hasher, MessageDigest};

use crate::io::*;
use crate::{PBOHeader, PBO};

impl<I: Seek + Read> PBO<I> {
    /// Write the PBO file
    pub fn write<O: Write>(&mut self, output: &mut O) -> Result<(), Error> {
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

        for key in &self.extension_order {
            if key == "prefix" {
                continue;
            }

            headers.write_cstring(key)?;
            headers.write_cstring(self.extensions.get(key).unwrap())?;
        }
        headers.write_cstring(String::new())?;

        let files_sorted = self.files_sorted(false);

        for header in &files_sorted {
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

        for header in &files_sorted {
            let cursor = self.retrieve(&header.filename).unwrap();
            output.write_all(cursor.get_ref())?;
            h.update(cursor.get_ref()).unwrap();
        }

        output.write_all(&[0])?;
        output.write_all(&*h.finish().unwrap())?;

        Ok(())
    }

    /// Generate a checksum of the PBO
    pub fn checksum(&mut self) -> Result<Vec<u8>, Error> {

        if let Some(checksum) = &self.checksum {
            return Ok(checksum.to_vec());
        }

        self.gen_checksum()
    }

    pub fn gen_checksum(&mut self) -> Result<Vec<u8>, Error> {
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

        for key in &self.extension_order {
            if key == "prefix" {
                continue;
            }

            headers.write_cstring(key)?;
            headers.write_cstring(self.extensions.get(key).unwrap())?;
        }
        headers.write_cstring(String::new())?;

        let files_sorted = self.files_sorted(false);

        for header in &files_sorted {
            let header = PBOHeader {
                filename: header.filename.clone(),
                method: 0,
                original: header.original,
                reserved: 0,
                timestamp: 0,
                size: header.size,
            };
            header.write(&mut headers)?;
        }

        let header = PBOHeader {
            method: 0,
            ..ext_header
        };
        header.write(&mut headers)?;

        let mut h = Hasher::new(MessageDigest::sha1()).unwrap();

        h.update(headers.get_ref()).unwrap();

        for header in &files_sorted {
            let cursor = self.retrieve(&header.filename).unwrap();
            h.update(cursor.get_ref()).unwrap();
        }

        Ok(h.finish().unwrap().to_vec())
    }
}
