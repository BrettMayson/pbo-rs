use std::io::{Error, Read, Seek, SeekFrom};

use crate::io::ReadExt;
use crate::{PBOHeader, PBO};

impl<I: Seek + Read> PBO<I> {
    /// Create a PBO object by reading a file
    pub fn read(mut input: I) -> Result<Self, Error> {
        let mut pbo = Self::new();
        loop {
            let (header, size) = PBOHeader::read(&mut input)?;
            pbo.blob_start += size as u64;
            if header.method == 0x5665_7273 {
                loop {
                    let s = input.read_cstring()?;
                    pbo.blob_start += s.as_bytes().len() as u64 + 1;
                    if s.is_empty() {
                        break;
                    }
                    let val = input.read_cstring()?;
                    pbo.blob_start += val.as_bytes().len() as u64 + 1;
                    pbo.extensions.insert(s.clone(), val);
                    pbo.extension_order.push(s);
                }
            } else if header.filename.is_empty() {
                break;
            } else {
                pbo.headers.push(header);
            }
        }

        for header in &pbo.headers {
            input
                .seek(SeekFrom::Current(i64::from(header.size)))
                .unwrap();
        }

        input.seek(SeekFrom::Current(1))?;
        let mut checksum = vec![0; 20];
        input.read_exact(&mut checksum)?;
        pbo.checksum = Some(checksum);

        pbo.input = Some(input);

        Ok(pbo)
    }
}
