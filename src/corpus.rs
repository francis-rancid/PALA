/// PALA signature corpus — binary format.
///
/// Layout:
///   [0..4]   magic    "PALA"
///   [4]      version  0x01
///   [5..9]   count    u32 LE — number of signature records
///   [9..]    records  (see SignatureRecord below)
///
/// Each record:
///   u8          name_len
///   [name_len]  name (UTF-8)
///   u8          ext_len
///   [ext_len]   extension (UTF-8, no dot)
///   u8          magic_len
///   [magic_len] magic bytes (header pattern)
///   u8          flags
///                 0x01  has_end_magic
///                 0x02  end_magic_last   (rfind — for types with nested instances)
///                 0x04  has_end_magic_trail (include trailing \r\n after end marker)
///                 0x08  special_riff
///                 0x10  special_zip
///                 0x20  special_mp4
///   if has_end_magic:
///     u8          end_magic_len
///     [end_magic_len] end_magic bytes
///   if has_end_magic_trail:
///     u8          trail_max  (max bytes of \r/\n to include)
///   u8          magic_offset  (bytes before magic in file; 4 for MP4 ftyp, else 0)
///   u64 LE      max_size
///   u32 LE      min_size
///   u8          desc_len
///   [desc_len]  description (UTF-8)

use anyhow::{anyhow, Result};

pub const MAGIC: &[u8; 4] = b"PALA";
pub const VERSION: u8 = 0x01;

pub const FLAG_HAS_END_MAGIC:     u8 = 0x01;
pub const FLAG_END_MAGIC_LAST:    u8 = 0x02;
pub const FLAG_END_MAGIC_TRAIL:   u8 = 0x04;
pub const FLAG_SPECIAL_RIFF:      u8 = 0x08;
pub const FLAG_SPECIAL_ZIP:       u8 = 0x10;
pub const FLAG_SPECIAL_MP4:       u8 = 0x20;

#[derive(Debug, Clone)]
pub struct Signature {
    pub name:            String,
    pub extension:       String,
    pub magic:           Vec<u8>,
    pub magic_offset:    u8,
    pub end_magic:       Option<Vec<u8>>,
    pub end_magic_last:  bool,
    pub end_magic_trail: u8,
    pub special_riff:    bool,
    pub special_zip:     bool,
    pub special_mp4:     bool,
    pub max_size:        u64,
    pub min_size:        u32,
    pub description:     String,
}

pub fn load_corpus(bytes: &[u8]) -> Result<Vec<Signature>> {
    if bytes.len() < 9 {
        return Err(anyhow!("corpus too short"));
    }
    if &bytes[0..4] != MAGIC {
        return Err(anyhow!("bad magic: expected PALA, got {:?}", &bytes[0..4]));
    }
    if bytes[4] != VERSION {
        return Err(anyhow!("unsupported version 0x{:02x}", bytes[4]));
    }
    let count = u32::from_le_bytes([bytes[5], bytes[6], bytes[7], bytes[8]]) as usize;

    let mut sigs = Vec::with_capacity(count);
    let mut pos = 9usize;

    macro_rules! read_bytes {
        ($n:expr, $label:literal) => {{
            if pos + $n > bytes.len() {
                return Err(anyhow!("truncated reading {}", $label));
            }
            let slice = &bytes[pos..pos + $n];
            pos += $n;
            slice
        }};
    }

    macro_rules! read_u8 {
        ($label:literal) => {{
            if pos >= bytes.len() {
                return Err(anyhow!("truncated reading {}", $label));
            }
            let v = bytes[pos];
            pos += 1;
            v
        }};
    }

    for _ in 0..count {
        let name_len = read_u8!("name_len") as usize;
        let name = std::str::from_utf8(read_bytes!(name_len, "name"))?.to_string();

        let ext_len = read_u8!("ext_len") as usize;
        let extension = std::str::from_utf8(read_bytes!(ext_len, "extension"))?.to_string();

        let magic_len = read_u8!("magic_len") as usize;
        let magic = read_bytes!(magic_len, "magic").to_vec();

        let flags = read_u8!("flags");
        let has_end_magic    = flags & FLAG_HAS_END_MAGIC   != 0;
        let end_magic_last   = flags & FLAG_END_MAGIC_LAST  != 0;
        let has_trail        = flags & FLAG_END_MAGIC_TRAIL != 0;
        let special_riff     = flags & FLAG_SPECIAL_RIFF    != 0;
        let special_zip      = flags & FLAG_SPECIAL_ZIP     != 0;
        let special_mp4      = flags & FLAG_SPECIAL_MP4     != 0;

        let end_magic = if has_end_magic {
            let em_len = read_u8!("end_magic_len") as usize;
            Some(read_bytes!(em_len, "end_magic").to_vec())
        } else {
            None
        };

        let end_magic_trail = if has_trail { read_u8!("trail_max") } else { 0 };
        let magic_offset = read_u8!("magic_offset");

        let max_size = u64::from_le_bytes(read_bytes!(8, "max_size").try_into().unwrap());
        let min_size = u32::from_le_bytes(read_bytes!(4, "min_size").try_into().unwrap());

        let desc_len = read_u8!("desc_len") as usize;
        let description = std::str::from_utf8(read_bytes!(desc_len, "description"))?.to_string();

        sigs.push(Signature {
            name, extension, magic, magic_offset,
            end_magic, end_magic_last, end_magic_trail,
            special_riff, special_zip, special_mp4,
            max_size, min_size, description,
        });
    }

    Ok(sigs)
}

/// Serialize a slice of Signature structs into the PALA corpus binary format.
pub fn serialize_corpus(sigs: &[Signature]) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(MAGIC);
    buf.push(VERSION);
    buf.extend_from_slice(&(sigs.len() as u32).to_le_bytes());

    for s in sigs {
        buf.push(s.name.len() as u8);
        buf.extend_from_slice(s.name.as_bytes());
        buf.push(s.extension.len() as u8);
        buf.extend_from_slice(s.extension.as_bytes());
        buf.push(s.magic.len() as u8);
        buf.extend_from_slice(&s.magic);

        let mut flags: u8 = 0;
        if s.end_magic.is_some()  { flags |= FLAG_HAS_END_MAGIC; }
        if s.end_magic_last       { flags |= FLAG_END_MAGIC_LAST; }
        if s.end_magic_trail > 0  { flags |= FLAG_END_MAGIC_TRAIL; }
        if s.special_riff         { flags |= FLAG_SPECIAL_RIFF; }
        if s.special_zip          { flags |= FLAG_SPECIAL_ZIP; }
        if s.special_mp4          { flags |= FLAG_SPECIAL_MP4; }
        buf.push(flags);

        if let Some(em) = &s.end_magic {
            buf.push(em.len() as u8);
            buf.extend_from_slice(em);
        }
        if s.end_magic_trail > 0 {
            buf.push(s.end_magic_trail);
        }

        buf.push(s.magic_offset);
        buf.extend_from_slice(&s.max_size.to_le_bytes());
        buf.extend_from_slice(&s.min_size.to_le_bytes());
        buf.push(s.description.len() as u8);
        buf.extend_from_slice(s.description.as_bytes());
    }

    buf
}
