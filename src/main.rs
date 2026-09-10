#![allow(dead_code)]

mod corpus;

use anyhow::{Context, Result};
use memchr::memmem;
use memmap2::MmapOptions;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::Instant;

const MB: usize = 1024 * 1024;
const GB: usize = 1024 * MB;
const DEFAULT_MIN: usize = 16;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Special {
    None, Riff, Zip, Mp4, Tiff, Sqlite, Jpeg, Gz, Aac, Bmp,
    // added from book synthesis
    Regf, Dex, Pf, Thumbcache,
}

// Static signature table — zero-cost, embedded in binary.
struct Sig {
    name:     &'static str,
    ext:      &'static str,
    magic:    &'static [u8],
    moff:     usize,
    em:       Option<&'static [u8]>,
    em_last:  bool,
    em_trail: usize,
    special:  Special,
    max:      usize,
    min:      usize,
    desc:     &'static str,
}

static SIGS: &[Sig] = &[
    // ── Images ───────────────────────────────────────────────────────────────
    Sig { name:"jpeg",       ext:"jpg",    magic:b"\xFF\xD8\xFF",                 moff:0, em:Some(b"\xFF\xD9"),        em_last:true,  em_trail:0, special:Special::Jpeg,       max:50*MB,   min:512,  desc:"JPEG Image" },
    Sig { name:"png",        ext:"png",    magic:b"\x89PNG\r\n\x1a\n",            moff:0, em:Some(b"IEND\xaeB`\x82"),  em_last:false, em_trail:0, special:Special::None,       max:100*MB,  min:0,    desc:"PNG Image" },
    Sig { name:"gif87a",     ext:"gif",    magic:b"GIF87a",                       moff:0, em:Some(b"\x00;"),           em_last:false, em_trail:0, special:Special::None,       max:20*MB,   min:0,    desc:"GIF Image (87a)" },
    Sig { name:"gif89a",     ext:"gif",    magic:b"GIF89a",                       moff:0, em:Some(b"\x00;"),           em_last:false, em_trail:0, special:Special::None,       max:20*MB,   min:0,    desc:"GIF Image (89a)" },
    Sig { name:"bmp",        ext:"bmp",    magic:b"BM",                           moff:0, em:None,                     em_last:false, em_trail:0, special:Special::Bmp,        max:100*MB,  min:54,   desc:"BMP Image" },
    Sig { name:"tiff_le",    ext:"tif",    magic:b"II*\x00",                      moff:0, em:None,                     em_last:false, em_trail:0, special:Special::Tiff,       max:500*MB,  min:0,    desc:"TIFF Image (LE)" },
    Sig { name:"tiff_be",    ext:"tif",    magic:b"MM\x00*",                      moff:0, em:None,                     em_last:false, em_trail:0, special:Special::Tiff,       max:500*MB,  min:0,    desc:"TIFF Image (BE)" },
    Sig { name:"psd",        ext:"psd",    magic:b"8BPS",                         moff:0, em:None,                     em_last:false, em_trail:0, special:Special::None,       max:2*GB,    min:0,    desc:"Photoshop Document" },
    // ── Audio / Video ─────────────────────────────────────────────────────────
    Sig { name:"riff",       ext:"riff",   magic:b"RIFF",                         moff:0, em:None,                     em_last:false, em_trail:0, special:Special::Riff,       max:4*GB,    min:0,    desc:"RIFF Container (WAV/AVI/WEBP)" },
    Sig { name:"mkv",        ext:"mkv",    magic:b"\x1a\x45\xdf\xa3",             moff:0, em:None,                     em_last:false, em_trail:0, special:Special::None,       max:8*GB,    min:0,    desc:"MKV/WebM Video" },
    Sig { name:"mp4",        ext:"mp4",    magic:b"ftyp",                         moff:4, em:None,                     em_last:false, em_trail:0, special:Special::Mp4,        max:8*GB,    min:0,    desc:"MP4/MOV Video" },
    Sig { name:"mp3_id3",    ext:"mp3",    magic:b"ID3",                          moff:0, em:None,                     em_last:false, em_trail:0, special:Special::None,       max:300*MB,  min:0,    desc:"MP3 Audio (ID3)" },
    Sig { name:"flac",       ext:"flac",   magic:b"fLaC",                         moff:0, em:None,                     em_last:false, em_trail:0, special:Special::None,       max:500*MB,  min:0,    desc:"FLAC Audio" },
    Sig { name:"aac",        ext:"aac",    magic:b"\xFF\xF1",                     moff:0, em:None,                     em_last:false, em_trail:0, special:Special::Aac,        max:200*MB,  min:0,    desc:"AAC Audio (ADTS)" },
    // ── Documents ─────────────────────────────────────────────────────────────
    Sig { name:"pdf",        ext:"pdf",    magic:b"%PDF-",                        moff:0, em:Some(b"%%EOF"),           em_last:true,  em_trail:2, special:Special::None,       max:500*MB,  min:0,    desc:"PDF Document" },
    Sig { name:"rtf",        ext:"rtf",    magic:b"{\\rtf",                       moff:0, em:Some(b"}"),               em_last:true,  em_trail:0, special:Special::None,       max:100*MB,  min:0,    desc:"RTF Document" },
    // ── Archives / containers ─────────────────────────────────────────────────
    Sig { name:"zip",        ext:"zip",    magic:b"PK\x03\x04",                   moff:0, em:None,                     em_last:false, em_trail:0, special:Special::Zip,        max:500*MB,  min:0,    desc:"ZIP / Office Open XML" },
    Sig { name:"ole2",       ext:"doc",    magic:b"\xD0\xCF\x11\xE0\xA1\xB1\x1A\xE1", moff:0, em:None,               em_last:false, em_trail:0, special:Special::None,       max:500*MB,  min:0,    desc:"OLE2 Document" },
    Sig { name:"gz",         ext:"gz",     magic:b"\x1f\x8b",                     moff:0, em:None,                     em_last:false, em_trail:0, special:Special::Gz,         max:2*GB,    min:0,    desc:"Gzip Archive" },
    Sig { name:"7z",         ext:"7z",     magic:b"7z\xBC\xAF'\x1C",             moff:0, em:None,                     em_last:false, em_trail:0, special:Special::None,       max:4*GB,    min:0,    desc:"7-Zip Archive" },
    Sig { name:"rar",        ext:"rar",    magic:b"Rar!\x1a\x07",                 moff:0, em:None,                     em_last:false, em_trail:0, special:Special::None,       max:4*GB,    min:0,    desc:"RAR Archive" },
    // ── Databases ─────────────────────────────────────────────────────────────
    Sig { name:"sqlite",     ext:"db",     magic:b"SQLite format 3\x00",          moff:0, em:None,                     em_last:false, em_trail:0, special:Special::Sqlite,     max:2*GB,    min:0,    desc:"SQLite Database" },
    Sig { name:"sqlite_wal", ext:"db-wal", magic:b"\x37\x7F\x06\x82",            moff:0, em:None,                     em_last:false, em_trail:0, special:Special::None,       max:2*GB,    min:32,   desc:"SQLite Write-Ahead Log" },
    // ── Email ─────────────────────────────────────────────────────────────────
    Sig { name:"eml",        ext:"eml",    magic:b"From:",                        moff:0, em:None,                     em_last:false, em_trail:0, special:Special::None,       max:50*MB,   min:64,   desc:"Email (EML)" },
    // ── Windows artifacts (from forensics book synthesis) ────────────────────
    Sig { name:"evtx",       ext:"evtx",   magic:b"ElfFile\x00",                  moff:0, em:None,                     em_last:false, em_trail:0, special:Special::None,       max:200*MB,  min:128,  desc:"Windows Event Log" },
    Sig { name:"regf",       ext:"dat",    magic:b"regf",                         moff:0, em:None,                     em_last:false, em_trail:0, special:Special::Regf,       max:512*MB,  min:4096, desc:"Windows Registry Hive" },
    Sig { name:"lnk",        ext:"lnk",    magic:b"\x4C\x00\x00\x00\x01\x14\x02\x00\x00\x00\x00\x00\xC0\x00\x00\x00\x00\x00\x00\x46", moff:0, em:None, em_last:false, em_trail:0, special:Special::None, max:50*MB, min:76, desc:"Windows Shell Link" },
    Sig { name:"pf",         ext:"pf",     magic:b"SCCA",                         moff:4, em:None,                     em_last:false, em_trail:0, special:Special::Pf,         max:10*MB,   min:76,   desc:"Windows Prefetch" },
    Sig { name:"thumbcache", ext:"db",     magic:b"CMMM",                         moff:0, em:None,                     em_last:false, em_trail:0, special:Special::Thumbcache, max:100*MB,  min:24,   desc:"Windows Thumbcache" },
    Sig { name:"hibr",       ext:"bin",    magic:b"hibr",                         moff:0, em:None,                     em_last:false, em_trail:0, special:Special::None,       max:8*GB,    min:4096, desc:"Windows Hibernate File" },
    Sig { name:"pagedump",   ext:"dmp",    magic:b"PAGEDUMP",                     moff:0, em:None,                     em_last:false, em_trail:0, special:Special::None,       max:64*GB,   min:4096, desc:"Windows Memory Dump (BSOD)" },
    // ── Mobile / cross-platform ───────────────────────────────────────────────
    Sig { name:"bplist",     ext:"plist",  magic:b"bplist00",                     moff:0, em:None,                     em_last:false, em_trail:0, special:Special::None,       max:64*MB,   min:8,    desc:"Apple Binary Property List" },
    Sig { name:"dex",        ext:"dex",    magic:b"dex\n",                        moff:0, em:None,                     em_last:false, em_trail:0, special:Special::Dex,        max:100*MB,  min:112,  desc:"Android Dalvik Executable" },
];

// ─── Runtime signature (static or corpus-loaded) ──────────────────────────────

pub struct DynSig {
    pub name:     String,
    pub ext:      String,
    pub magic:    Vec<u8>,
    pub moff:     usize,
    pub em:       Option<Vec<u8>>,
    pub em_last:  bool,
    pub em_trail: usize,
    pub special:  Special,
    pub max:      usize,
    pub min:      usize,
    pub desc:     String,
}

fn make_dyn(s: &Sig) -> DynSig {
    DynSig {
        name: s.name.to_string(), ext: s.ext.to_string(),
        magic: s.magic.to_vec(), moff: s.moff,
        em: s.em.map(|b| b.to_vec()), em_last: s.em_last, em_trail: s.em_trail,
        special: s.special, max: s.max, min: s.min, desc: s.desc.to_string(),
    }
}

fn corpus_dyn(s: &corpus::Signature) -> DynSig {
    let special = if s.special_riff      { Special::Riff }
                  else if s.special_zip  { Special::Zip  }
                  else if s.special_mp4  { Special::Mp4  }
                  else                   { Special::None };
    DynSig {
        name: s.name.clone(), ext: s.extension.clone(),
        magic: s.magic.clone(), moff: s.magic_offset as usize,
        em: s.end_magic.clone(), em_last: s.end_magic_last,
        em_trail: s.end_magic_trail as usize,
        special, max: s.max_size as usize, min: s.min_size as usize,
        desc: s.description.clone(),
    }
}

// ─── Size/end algorithms ──────────────────────────────────────────────────────

fn jpeg_length(data: &[u8]) -> Option<usize> {
    let mut i = 2usize;
    loop {
        if i + 1 >= data.len() { return None; }
        if data[i] != 0xFF { return None; }
        while i < data.len() && data[i] == 0xFF { i += 1; }
        if i >= data.len() { return None; }
        let marker = data[i];
        i += 1;
        if marker == 0xD9 { return Some(i); }
        if marker == 0xD8 || marker == 0x01 || (0xD0..=0xD7).contains(&marker) { continue; }
        if i + 2 > data.len() { return None; }
        let seg_len = ((data[i] as usize) << 8) | data[i + 1] as usize;
        if seg_len < 2 { return None; }
        if marker == 0xDA {
            i += seg_len;
            loop {
                if i + 1 >= data.len() { return None; }
                if data[i] == 0xFF {
                    match data[i + 1] {
                        0x00 => i += 2,
                        0xD0..=0xD7 => i += 2,
                        0xD9 => return Some(i + 2),
                        _ => break,
                    }
                } else {
                    i += 1;
                }
            }
        } else {
            i += seg_len;
        }
    }
}

fn riff_size(data: &[u8]) -> Option<usize> {
    if data.len() < 8 { return None; }
    let inner = u32::from_le_bytes(data[4..8].try_into().ok()?) as usize;
    inner.checked_add(8)
}

fn bmp_size(data: &[u8]) -> Option<usize> {
    if data.len() < 6 { return None; }
    let sz = u32::from_le_bytes(data[2..6].try_into().ok()?) as usize;
    if sz > 0 { Some(sz) } else { None }
}

fn sqlite_size(data: &[u8]) -> Option<usize> {
    if data.len() < 100 { return None; }
    let raw = u16::from_be_bytes([data[16], data[17]]) as usize;
    let ps = if raw == 1 { 65536 } else { raw };
    if ps < 512 || ps > 65536 || (ps & (ps - 1)) != 0 { return None; }
    let pc = u32::from_be_bytes(data[28..32].try_into().ok()?) as usize;
    ps.checked_mul(pc)
}

fn tiff_size(data: &[u8]) -> Option<usize> {
    if data.len() < 8 { return None; }
    let le = data[0..2] == *b"II";
    macro_rules! u16at { ($o:expr) => {{
        let o=$o; if o+2>data.len(){return None;}
        if le {u16::from_le_bytes([data[o],data[o+1]])} else {u16::from_be_bytes([data[o],data[o+1]])}
    }};}
    macro_rules! u32at { ($o:expr) => {{
        let o=$o; if o+4>data.len(){return None;}
        if le {u32::from_le_bytes(data[o..o+4].try_into().unwrap())}
        else  {u32::from_be_bytes(data[o..o+4].try_into().unwrap())}
    }};}
    macro_rules! u32raw { ($b:expr) => {{
        let b:[u8;4]=$b;
        if le {u32::from_le_bytes(b)} else {u32::from_be_bytes(b)}
    }};}
    if u16at!(2) != 42 { return None; }

    const OTAGS: &[u16] = &[273, 324, 519, 520, 521];
    const CTAGS: &[u16] = &[279, 325, 522, 523, 524];
    let tsz = |t: u16| -> usize { match t { 1|2|6|7=>1, 3|8=>2, 4|9|11=>4, 5|10|12=>8, _=>1 } };

    let mut mx: usize = 8;
    let mut vis: HashSet<usize> = HashSet::new();
    let mut ifd = u32at!(4) as usize;

    loop {
        if ifd == 0 || !vis.insert(ifd) || ifd + 2 > data.len() { break; }
        let n = u16at!(ifd) as usize;
        let mut offs: Vec<usize> = Vec::new();
        let mut cnts: Vec<usize> = Vec::new();

        for i in 0..n {
            let ep = ifd + 2 + i * 12;
            if ep + 12 > data.len() { break; }
            let tag   = u16at!(ep);
            let dtype = u16at!(ep + 2);
            let count = u32at!(ep + 4) as usize;
            let raw: [u8; 4] = data[ep+8..ep+12].try_into().unwrap();
            let ts = tsz(dtype);
            let dsz = ts.saturating_mul(count);

            if dsz <= 4 {
                mx = mx.max(ep + 12);
                let v = u32raw!(raw) as usize;
                if OTAGS.contains(&tag) { offs.push(v); }
                else if CTAGS.contains(&tag) { cnts.push(v); }
            } else {
                let doff = u32raw!(raw) as usize;
                mx = mx.max(doff.saturating_add(dsz));
                if OTAGS.contains(&tag) || CTAGS.contains(&tag) {
                    let is_off = OTAGS.contains(&tag);
                    let isz = if ts >= 4 { 4usize } else { 2usize };
                    for j in 0..count {
                        let ip = doff + j * ts;
                        if ip + isz > data.len() { break; }
                        let v: usize = if isz == 4 {
                            if le { u32::from_le_bytes(data[ip..ip+4].try_into().unwrap()) as usize }
                            else  { u32::from_be_bytes(data[ip..ip+4].try_into().unwrap()) as usize }
                        } else {
                            if le { u16::from_le_bytes(data[ip..ip+2].try_into().unwrap()) as usize }
                            else  { u16::from_be_bytes(data[ip..ip+2].try_into().unwrap()) as usize }
                        };
                        if is_off { offs.push(v); } else { cnts.push(v); }
                    }
                }
            }
        }

        for (o, c) in offs.iter().zip(cnts.iter()) { mx = mx.max(o.saturating_add(*c)); }
        for o in offs.get(cnts.len()..).unwrap_or(&[]) { mx = mx.max(*o); }

        let np = ifd + 2 + n * 12;
        if np + 4 > data.len() { break; }
        mx = mx.max(np + 4);
        ifd = u32at!(np) as usize;
    }
    if mx > 8 { Some(mx) } else { None }
}

fn zip_eocd_end(data: &[u8]) -> Option<usize> {
    let p = memmem::find(data, b"PK\x05\x06")?;
    if p + 22 > data.len() { return Some(p + 4); }
    let clen = u16::from_le_bytes([data[p+20], data[p+21]]) as usize;
    Some(p + 22 + clen)
}

// hive_bins_size (LE u32 at offset 40) + 4096 base header = total hive size
fn regf_size(data: &[u8]) -> Option<usize> {
    if data.len() < 44 { return None; }
    let hbs = u32::from_le_bytes(data[40..44].try_into().ok()?) as usize;
    hbs.checked_add(4096)
}

// file_size at offset 12 (LE u32) gives exact prefetch file length
fn pf_size(data: &[u8]) -> Option<usize> {
    if data.len() < 16 { return None; }
    let sz = u32::from_le_bytes(data[12..16].try_into().ok()?) as usize;
    if sz >= 76 { Some(sz) } else { None }
}

// file_size at offset 32 (LE u32) gives exact DEX file length
fn dex_size(data: &[u8]) -> Option<usize> {
    if data.len() < 36 { return None; }
    let sz = u32::from_le_bytes(data[32..36].try_into().ok()?) as usize;
    if sz >= 112 { Some(sz) } else { None }
}

// ─── Subtype detection ────────────────────────────────────────────────────────

fn riff_sub(data: &[u8]) -> (&'static str, &'static str) {
    if data.len() >= 12 {
        match &data[8..12] {
            b"WAVE" => return ("wav",  "WAV Audio"),
            b"AVI " => return ("avi",  "AVI Video"),
            b"WEBP" => return ("webp", "WebP Image"),
            _ => {}
        }
    }
    ("riff", "RIFF Container")
}

fn zip_sub(data: &[u8]) -> (&'static str, &'static str) {
    let eocd = match memmem::find(data, b"PK\x05\x06") {
        Some(p) => p, None => return ("zip", "ZIP Archive"),
    };
    if eocd + 22 > data.len() { return ("zip", "ZIP Archive"); }
    let cdsz  = u32::from_le_bytes(data[eocd+12..eocd+16].try_into().unwrap_or([0;4])) as usize;
    let cdoff = u32::from_le_bytes(data[eocd+16..eocd+20].try_into().unwrap_or([0;4])) as usize;
    if cdoff >= data.len() { return ("zip", "ZIP Archive"); }
    let cd = &data[cdoff..(cdoff + cdsz).min(data.len())];

    const PFX: &[(&[u8], &str, &str)] = &[
        (b"word/",     "docx", "Word Document"),
        (b"xl/",       "xlsx", "Excel Spreadsheet"),
        (b"ppt/",      "pptx", "PowerPoint Presentation"),
        (b"META-INF/", "odt",  "OpenDocument"),
    ];

    let mut pos = 0;
    while pos + 46 <= cd.len() {
        if &cd[pos..pos+4] != b"PK\x01\x02" { break; }
        let fnl = u16::from_le_bytes([cd[pos+28], cd[pos+29]]) as usize;
        let exl = u16::from_le_bytes([cd[pos+30], cd[pos+31]]) as usize;
        let cml = u16::from_le_bytes([cd[pos+32], cd[pos+33]]) as usize;
        let fns = pos + 46;
        let fne = (fns + fnl).min(cd.len());
        let fname = &cd[fns..fne];
        for (p, ext, desc) in PFX { if fname.starts_with(p) { return (ext, desc); } }
        pos += 46 + fnl + exl + cml;
    }
    ("zip", "ZIP Archive")
}

// ─── Validators ───────────────────────────────────────────────────────────────

fn jpeg_ok(d: &[u8]) -> bool {
    if d.len() < 4 { return false; }
    let m = d[3];
    if m == 0x00 || m == 0x01 || m == 0xFF || (0xF8..=0xFD).contains(&m) { return false; }
    if d.len() >= 8 {
        let sl = ((d[4] as usize) << 8) | d[5] as usize;
        let np = 4 + sl;
        if sl < 2 || np >= d.len() || d[np] != 0xFF { return false; }
    }
    true
}

fn gz_ok(d: &[u8])  -> bool { d.len() >= 3 && d[2] == 0x08 }

fn aac_ok(d: &[u8]) -> bool {
    if d.len() < 7 || (d[1] & 0x06) != 0x00 { return false; }
    let fl = (((d[3] & 0x03) as usize) << 11) | ((d[4] as usize) << 3) | ((d[5] >> 5) as usize);
    (8..=8192).contains(&fl)
}

fn bmp_ok(d: &[u8]) -> bool {
    if d.len() < 18 { return true; }
    const VALID: &[u32] = &[12, 40, 52, 56, 64, 108, 124];
    VALID.contains(&u32::from_le_bytes(d[14..18].try_into().unwrap()))
}

// version byte at offset 0 must be a known Prefetch format version
fn pf_ok(d: &[u8]) -> bool {
    d.len() >= 13 && matches!(d[0], 0x11 | 0x17 | 0x1A | 0x1E)
}

// bytes 4-6 must be "03" + ASCII digit; byte 7 must be NUL
fn dex_ok(d: &[u8]) -> bool {
    d.len() >= 8 && d[4] == b'0' && d[5] == b'3' && d[6].is_ascii_digit() && d[7] == 0
}

// version LE u32 at offset 4 must be a known Thumbcache format version
fn thumbcache_ok(d: &[u8]) -> bool {
    if d.len() < 8 { return false; }
    let ver = u32::from_le_bytes(d[4..8].try_into().unwrap_or([0;4]));
    ver > 0 && ver < 0x30
}

// ─── Utilities ────────────────────────────────────────────────────────────────

fn human_size(n: usize) -> String {
    if n >= 1024 * 1024 { format!("{:.1}MB", n as f64 / (1024.0 * 1024.0)) }
    else if n >= 1024   { format!("{:.1}KB", n as f64 / 1024.0) }
    else                { format!("{}B", n) }
}

#[cfg(unix)]
fn same_device(a: &Path, b: &Path) -> bool {
    use std::os::unix::fs::MetadataExt;
    match (std::fs::metadata(a), std::fs::metadata(b)) {
        (Ok(ma), Ok(mb)) => ma.dev() == mb.dev(),
        _ => false,
    }
}

// ─── Carving engine ───────────────────────────────────────────────────────────

pub struct Finding {
    pub offset: usize,
    pub name:   String,
    pub ext:    String,
    pub desc:   String,
    pub size:   usize,
    pub path:   PathBuf,
    pub trunc:  bool,
    pub sha256: String,
}

pub fn carve(src: &[u8], outdir: &Path, sigs: &[DynSig], quiet: bool) -> Result<Vec<Finding>> {
    fs::create_dir_all(outdir)?;

    let mut out:  Vec<Finding>          = Vec::new();
    let mut seen: HashSet<usize>        = HashSet::new();
    let mut ctr:  HashMap<String, usize> = HashMap::new();
    let total = sigs.len();

    for (si, sig) in sigs.iter().enumerate() {
        if !quiet {
            eprintln!("  [{:>2}/{}] {}...", si + 1, total, sig.name);
        }
        let finder = memmem::Finder::new(&sig.magic);
        let min = sig.min.max(DEFAULT_MIN);
        let mut ss = 0usize;

        while let Some(rel) = finder.find(&src[ss..]) {
            let idx = ss + rel;
            ss = idx + 1;

            if idx < sig.moff { continue; }
            let start = idx - sig.moff;
            if seen.contains(&start) { continue; }

            let wend = (start + sig.max).min(src.len());
            let data = &src[start..wend];
            if data.len() < min { continue; }

            // ── end-finding ──────────────────────────────────────────────────
            let (carved, mut trunc): (&[u8], bool) = match sig.special {
                Special::Jpeg => match jpeg_length(data) {
                    Some(n) => (&data[..n], false),
                    None    => match memmem::find(data, b"\xFF\xD9") {
                        Some(p) => (&data[..p+2], false),
                        None    => (data, true),
                    },
                },
                Special::Zip => match zip_eocd_end(data) {
                    Some(e) => (&data[..e.min(data.len())], false),
                    None    => (data, true),
                },
                _ => {
                    if let Some(ref em) = sig.em {
                        let p = if sig.em_last {
                            memmem::rfind(data, em)
                        } else {
                            memmem::find(data, em)
                        };
                        match p {
                            Some(p) => {
                                let mut cut = p + em.len();
                                let mut tr = sig.em_trail;
                                while tr > 0 && cut < data.len() && matches!(data[cut], 0x0A|0x0D) {
                                    cut += 1; tr -= 1;
                                }
                                (&data[..cut], false)
                            }
                            None => (data, true),
                        }
                    } else {
                        (data, true)
                    }
                }
            };

            if carved.len() < min { continue; }

            // ── validation ───────────────────────────────────────────────────
            let valid = match sig.special {
                Special::Jpeg       => jpeg_ok(carved),
                Special::Gz         => gz_ok(carved),
                Special::Aac        => aac_ok(carved),
                Special::Bmp        => bmp_ok(carved),
                Special::Pf         => pf_ok(carved),
                Special::Dex        => dex_ok(carved),
                Special::Thumbcache => thumbcache_ok(carved),
                _                   => true,
            };
            if !valid { continue; }

            // ── subtype + size-field trimming ─────────────────────────────────
            let (fext, fdesc, fdata): (&str, &str, &[u8]) = match sig.special {
                Special::Riff => {
                    let (e, d) = riff_sub(carved);
                    let trim = match riff_size(carved) {
                        Some(n) if n <= carved.len() => { trunc = false; &carved[..n] }
                        _ => carved,
                    };
                    (e, d, trim)
                }
                Special::Zip => {
                    let (e, d) = zip_sub(carved);
                    (e, d, carved)
                }
                Special::Bmp => {
                    let trim = match bmp_size(carved) {
                        Some(n) if n < carved.len() => &carved[..n],
                        _ => carved,
                    };
                    (&sig.ext, &sig.desc, trim)
                }
                Special::Tiff => {
                    let trim = match tiff_size(carved) {
                        Some(n) if n <= carved.len() => { trunc = false; &carved[..n] }
                        _ => carved,
                    };
                    (&sig.ext, &sig.desc, trim)
                }
                Special::Sqlite => {
                    let trim = match sqlite_size(carved) {
                        Some(n) if n <= carved.len() => { trunc = false; &carved[..n] }
                        _ => carved,
                    };
                    (&sig.ext, &sig.desc, trim)
                }
                Special::Regf => {
                    let trim = match regf_size(carved) {
                        Some(n) if n <= carved.len() => { trunc = false; &carved[..n] }
                        _ => carved,
                    };
                    (&sig.ext, &sig.desc, trim)
                }
                Special::Pf => {
                    let trim = match pf_size(carved) {
                        Some(n) if n <= carved.len() => { trunc = false; &carved[..n] }
                        _ => carved,
                    };
                    (&sig.ext, &sig.desc, trim)
                }
                Special::Dex => {
                    let trim = match dex_size(carved) {
                        Some(n) if n <= carved.len() => { trunc = false; &carved[..n] }
                        _ => carved,
                    };
                    (&sig.ext, &sig.desc, trim)
                }
                _ => (&sig.ext, &sig.desc, carved),
            };

            // ── write ─────────────────────────────────────────────────────────
            // JPEG files that hit max_size without finding EOI get a _partial
            // suffix so the user knows the recovered file is incomplete.
            let n = {
                let c = ctr.entry(fext.to_string()).and_modify(|x| *x += 1).or_insert(1);
                *c
            };
            let partial = trunc && sig.special == Special::Jpeg;
            let fname = if partial {
                format!("{fext}_{n:04}_partial.{fext}")
            } else {
                format!("{fext}_{n:04}.{fext}")
            };
            let out_path = outdir.join(&fname);
            fs::write(&out_path, fdata)
                .with_context(|| format!("writing {}", out_path.display()))?;

            let mut h = Sha256::new();
            h.update(fdata);
            let sha = format!("{:x}", h.finalize());

            if !quiet {
                let trunc_mark = if trunc { " [partial]" } else { "" };
                eprintln!("         {:<5}  {:>8}  offset=0x{:08x}  {}{}", fext, human_size(fdata.len()), start, fname, trunc_mark);
            }

            seen.insert(start);
            out.push(Finding {
                offset: start,
                name:   sig.name.clone(),
                ext:    fext.to_string(),
                desc:   fdesc.to_string(),
                size:   fdata.len(),
                path:   out_path,
                trunc,
                sha256: sha,
            });
        }
    }
    Ok(out)
}

// ─── CLI ──────────────────────────────────────────────────────────────────────

fn usage() {
    eprintln!("pala — filesystem-agnostic file carver\n");
    eprintln!("Usage:  pala <source> <outdir> [OPTIONS]\n");
    eprintln!("Arguments:");
    eprintln!("  source    Disk image file or block device to scan");
    eprintln!("  outdir    Directory to write recovered files into");
    eprintln!("            MUST be on a different drive than <source>\n");
    eprintln!("Options:");
    eprintln!("  -t, --types <csv>    File types to recover (default: all)");
    eprintln!("  -c, --corpus <path>  Load extra signatures from a PALA corpus file");
    eprintln!("  -l, --list           List available types and exit");
    eprintln!("  -q, --quiet          Suppress all output (use with --json)");
    eprintln!("      --json           Write structured JSON summary to stdout");
    eprintln!("  -h, --help           Show this help\n");
    eprintln!("Examples:");
    eprintln!("  pala disk.img /mnt/usb/recovered/");
    eprintln!("  pala /dev/sdb /mnt/usb/recovered/ -t jpeg,png,pdf");
    eprintln!("  pala disk.img out/ --json | jq '.findings[] | {{ext,size}}'");
    eprintln!("  pala disk.img out/ -c custom.pala     # with extra signatures");
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let mut src:         Option<String>      = None;
    let mut out:         Option<String>      = None;
    let mut types:       Option<Vec<String>> = None;
    let mut corpus_path: Option<String>      = None;
    let mut quiet  = false;
    let mut json   = false;
    let mut list   = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-h"|"--help"   => { usage(); return Ok(()); }
            "-l"|"--list"   => list  = true,
            "-q"|"--quiet"  => quiet = true,
            "--json"        => json  = true,
            "-t"|"--types"  => {
                i += 1;
                if i >= args.len() { anyhow::bail!("--types requires a value"); }
                types = Some(args[i].split(',').map(|s| s.trim().to_string()).collect());
            }
            a if a.starts_with("--types=") => {
                types = Some(a[8..].split(',').map(|s| s.trim().to_string()).collect());
            }
            "-c"|"--corpus" => {
                i += 1;
                if i >= args.len() { anyhow::bail!("--corpus requires a path"); }
                corpus_path = Some(args[i].clone());
            }
            a if a.starts_with("--corpus=") => {
                corpus_path = Some(a[9..].to_string());
            }
            a => {
                if src.is_none()      { src = Some(a.to_string()); }
                else if out.is_none() { out = Some(a.to_string()); }
                else { anyhow::bail!("unexpected argument: {a}"); }
            }
        }
        i += 1;
    }

    // ── Build signature list ───────────────────────────────────────────────────
    let mut all_sigs: Vec<DynSig> = SIGS.iter().map(make_dyn).collect();

    if let Some(ref cp) = corpus_path {
        let bytes = std::fs::read(cp).with_context(|| format!("reading corpus {cp}"))?;
        let extra = corpus::load_corpus(&bytes)
            .with_context(|| format!("parsing corpus {cp}"))?;
        eprintln!("pala: loaded {} extra signature(s) from {cp}", extra.len());
        all_sigs.extend(extra.iter().map(corpus_dyn));
    }

    // ── Filter by type names ───────────────────────────────────────────────────
    let sigs: Vec<DynSig> = if let Some(ref type_names) = types {
        let filtered: Vec<DynSig> = all_sigs
            .into_iter()
            .filter(|s| type_names.iter().any(|n| n == &s.name))
            .collect();
        if filtered.is_empty() {
            anyhow::bail!("no matching types (run `pala --list` to see available types)");
        }
        filtered
    } else {
        all_sigs
    };

    // ── List mode ─────────────────────────────────────────────────────────────
    if list {
        println!("{:<12}  {:<6}  {}", "NAME", "EXT", "DESCRIPTION");
        println!("{}", "-".repeat(48));
        for s in &sigs { println!("{:<12}  {:<6}  {}", s.name, s.ext, s.desc); }
        return Ok(());
    }

    // ── Require source + output ───────────────────────────────────────────────
    let src_path = src.ok_or_else(|| anyhow::anyhow!("source path required\n\nRun `pala --help` for usage."))?;
    let out_path = out.ok_or_else(|| anyhow::anyhow!("output directory required\n\nRun `pala --help` for usage."))?;

    let file = File::open(&src_path).with_context(|| format!("opening {src_path}"))?;
    let size = {
        let m = file.metadata()?;
        if m.len() > 0 { m.len() as usize }
        else {
            let mut f = File::open(&src_path)?;
            f.seek(SeekFrom::End(0))? as usize
        }
    };
    if size == 0 { anyhow::bail!("source is empty"); }

    let outdir = PathBuf::from(&out_path);
    fs::create_dir_all(&outdir)?;

    // Warn if source and output are on the same device.
    #[cfg(unix)]
    if same_device(std::path::Path::new(&src_path), &outdir) {
        eprintln!("WARNING: output directory is on the SAME device as the source.");
        eprintln!("         Writing recovered files here risks overwriting the data you are");
        eprintln!("         trying to recover.  Use a different drive for --output.");
        eprintln!("         (e.g. a USB stick, an external drive, or a network share)");
        eprintln!();
    }

    if !quiet {
        eprintln!("pala: source  = {src_path} ({})", human_size(size));
        eprintln!("pala: output  = {out_path}");
        eprintln!();
    }

    let mmap = unsafe { MmapOptions::new().len(size).map(&file) }
        .with_context(|| format!("mmap {src_path}"))?;

    let t0 = Instant::now();
    let findings = carve(&mmap, &outdir, &sigs, quiet)?;
    let elapsed = t0.elapsed();

    // ── JSON output ───────────────────────────────────────────────────────────
    if json {
        let src_j = serde_json::to_string(&src_path).unwrap();
        let out_j = serde_json::to_string(&out_path).unwrap();
        print!(r#"{{"source":{src_j},"output":{out_j},"source_bytes":{size},"elapsed_ms":{},"found":{},"findings":["#,
               elapsed.as_millis(), findings.len());
        for (idx, f) in findings.iter().enumerate() {
            if idx > 0 { print!(","); }
            let path_j = serde_json::to_string(&f.path.to_string_lossy().as_ref()).unwrap();
            let name_j = serde_json::to_string(&f.name).unwrap();
            let ext_j  = serde_json::to_string(&f.ext).unwrap();
            let desc_j = serde_json::to_string(&f.desc).unwrap();
            let sha_j  = serde_json::to_string(&f.sha256).unwrap();
            print!(r#"{{"offset":{},"type":{name_j},"extension":{ext_j},"description":{desc_j},"size":{},"path":{path_j},"truncated":{},"sha256":{sha_j}}}"#,
                   f.offset, f.size, f.trunc);
        }
        println!("]}}");
    }

    // ── Human-readable summary ────────────────────────────────────────────────
    if !quiet {
        eprintln!();
        if findings.is_empty() {
            eprintln!("pala: no files recovered. ({:.1}s)", elapsed.as_secs_f32());
            if size > 100 * MB {
                eprintln!("pala: hint: if the drive was wiped (all zeros) or encrypted, no data can be recovered by carving.");
                eprintln!("pala: hint: try scanning the raw device (/dev/sdX) if you scanned a partition image.");
            }
        } else {
            let mut by_ext: HashMap<&str, usize> = HashMap::new();
            for f in &findings { *by_ext.entry(f.ext.as_str()).or_insert(0) += 1; }
            let mut groups: Vec<_> = by_ext.iter().collect();
            groups.sort_by_key(|(ext, _)| *ext);
            let summary = groups.iter()
                .map(|(ext, n)| format!("{n} {ext}"))
                .collect::<Vec<_>>()
                .join(", ");
            eprintln!("pala: recovered {} file(s) in {:.1}s → {out_path}", findings.len(), elapsed.as_secs_f32());
            eprintln!("pala: {summary}");
        }
    }
    Ok(())
}
