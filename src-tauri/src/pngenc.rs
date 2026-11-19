//! A minimal PNG writer for small RGBA images (app icons on Windows), so no image crate is needed.

use std::io::Write;

/// The PNG bytes of a `width` x `height` RGBA image, rows top to bottom.
pub fn encode_rgba(width: u32, height: u32, rgba: &[u8]) -> Option<Vec<u8>> {
    let stride = width as usize * 4;
    if width == 0 || height == 0 || rgba.len() != stride * height as usize {
        return None;
    }
    // Each row starts with filter type 0 (none).
    let mut raw = Vec::with_capacity((stride + 1) * height as usize);
    for row in rgba.chunks_exact(stride) {
        raw.push(0);
        raw.extend_from_slice(row);
    }
    let mut zlib = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    zlib.write_all(&raw).ok()?;
    let data = zlib.finish().ok()?;

    let mut header = Vec::with_capacity(13);
    header.extend_from_slice(&width.to_be_bytes());
    header.extend_from_slice(&height.to_be_bytes());
    header.extend_from_slice(&[8, 6, 0, 0, 0]); // 8-bit, RGBA, deflate, no filter set, no interlace

    let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
    chunk(&mut png, b"IHDR", &header);
    chunk(&mut png, b"IDAT", &data);
    chunk(&mut png, b"IEND", &[]);
    Some(png)
}

fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    out.extend_from_slice(kind);
    out.extend_from_slice(data);
    let mut crc = Crc::new();
    crc.update(kind);
    crc.update(data);
    out.extend_from_slice(&crc.finish().to_be_bytes());
}

/// CRC-32 as PNG uses it (the zlib polynomial).
struct Crc(u32);

impl Crc {
    fn new() -> Self {
        Crc(0xffff_ffff)
    }

    fn update(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            let mut c = (self.0 ^ u32::from(byte)) & 0xff;
            for _ in 0..8 {
                c = if c & 1 != 0 { 0xedb8_8320 ^ (c >> 1) } else { c >> 1 };
            }
            self.0 = (self.0 >> 8) ^ c;
        }
    }

    fn finish(self) -> u32 {
        self.0 ^ 0xffff_ffff
    }
}

#[cfg(test)]
#[path = "tests/pngenc.rs"]
mod tests;
