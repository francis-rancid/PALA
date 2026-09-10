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

const MB: usize = 1024 * 1024;
const GB: usize = 1024 * MB;
const DEFAULT_MIN: usize = 16;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Special {
    None, Riff, Zip, Mp4, Tiff, Sqlite, Jpeg, Gz, Aac, Bmp,
}

struct Sig {
    name:   &'static str,
    ext:    &'static str,
    magic:  &'static [u8],
    moff:   usize,
    em:     Option<&'static [u8]>,
    em_last: bool,
    em_trail: usize,
    special: Special,
    max:    usize,
    min:    usize,
    desc:   &'static str,
}

static SIGS: &[Sig] = &[
    Sig { name:"jpeg",    ext:"jpg",  magic:b"\xFF\xD8\xFF",               moff:0, em:Some(b"\xFF\xD9"),        em_last:true,  em_trail:0, special:Special::Jpeg,   max:50*MB,   min:512, desc:"JPEG Image" },
    Sig { name:"png",     ext:"png",  magic:b"\x89PNG\r\n\x1a\n",          moff:0, em:Some(b"IEND\xaeB`\x82"),  em_last:false, em_trail:0, special:Special::None,   max:100*MB,  min:0,   desc:"PNG Image" },
    Sig { name:"gif87a",  ext:"gif",  magic:b"GIF87a",                     moff:0, em:Some(b"\x00;"),           em_last:false, em_trail:0, special:Special::None,   max:20*MB,   min:0,   desc:"GIF Image (87a)" },
    Sig { name:"gif89a",  ext:"gif",  magic:b"GIF89a",                     moff:0, em:Some(b"\x00;"),           em_last:false, em_trail:0, special:Special::None,   max:20*MB,   min:0,   desc:"GIF Image (89a)" },
    Sig { name:"bmp",     ext:"bmp",  magic:b"BM",                         moff:0, em:None,                     em_last:false, em_trail:0, special:Special::Bmp,    max:100*MB,  min:54,  desc:"BMP Image" },
    Sig { name:"tiff_le", ext:"tif",  magic:b"II*\x00",                    moff:0, em:None,                     em_last:false, em_trail:0, special:Special::Tiff,   max:500*MB,  min:0,   desc:"TIFF Image (LE)" },
    Sig { name:"tiff_be", ext:"tif",  magic:b"MM\x00*",                    moff:0, em:None,                     em_last:false, em_trail:0, special:Special::Tiff,   max:500*MB,  min:0,   desc:"TIFF Image (BE)" },
    Sig { name:"psd",     ext:"psd",  magic:b"8BPS",                       moff:0, em:None,                     em_last:false, em_trail:0, special:Special::None,   max:2*GB,    min:0,   desc:"Photoshop Document" },
    Sig { name:"riff",    ext:"riff", magic:b"RIFF",                       moff:0, em:None,                     em_last:false, em_trail:0, special:Special::Riff,   max:4*GB,    min:0,   desc:"RIFF Container (WAV/AVI/WEBP)" },
    Sig { name:"mkv",     ext:"mkv",  magic:b"\x1a\x45\xdf\xa3",           moff:0, em:None,                     em_last:false, em_trail:0, special:Special::None,   max:8*GB,    min:0,   desc:"MKV/WebM Video" },
    Sig { name:"mp4",     ext:"mp4",  magic:b"ftyp",                       moff:4, em:None,                     em_last:false, em_trail:0, special:Special::Mp4,    max:8*GB,    min:0,   desc:"MP4/MOV Video" },
    Sig { name:"mp3_id3", ext:"mp3",  magic:b"ID3",                        moff:0, em:None,                     em_last:false, em_trail:0, special:Special::None,   max:300*MB,  min:0,   desc:"MP3 Audio (ID3)" },
    Sig { name:"flac",    ext:"flac", magic:b"fLaC",                       moff:0, em:None,                     em_last:false, em_trail:0, special:Special::None,   max:500*MB,  min:0,   desc:"FLAC Audio" },
    Sig { name:"aac",     ext:"aac",  magic:b"\xFF\xF1",                   moff:0, em:None,                     em_last:false, em_trail:0, special:Special::Aac,    max:200*MB,  min:0,   desc:"AAC Audio (ADTS)" },
    Sig { name:"pdf",     ext:"pdf",  magic:b"%PDF-",                      moff:0, em:Some(b"%%EOF"),           em_last:true,  em_trail:2, special:Special::None,   max:500*MB,  min:0,   desc:"PDF Document" },
    Sig { name:"rtf",     ext:"rtf",  magic:b"{\\rtf",                     moff:0, em:Some(b"}"),               em_last:true,  em_trail:0, special:Special::None,   max:100*MB,  min:0,   desc:"RTF Document" },
    Sig { name:"zip",     ext:"zip",  magic:b"PK\x03\x04",                 moff:0, em:None,                     em_last:false, em_trail:0, special:Special::Zip,    max:500*MB,  min:0,   desc:"ZIP / Office Open XML" },
    Sig { name:"ole2",    ext:"doc",  magic:b"\xD0\xCF\x11\xE0\xA1\xB1\x1A\xE1", moff:0, em:None,             em_last:false, em_trail:0, special:Special::None,   max:500*MB,  min:0,   desc:"OLE2 Document" },
    Sig { name:"gz",      ext:"gz",   magic:b"\x1f\x8b",                   moff:0, em:None,                     em_last:false, em_trail:0, special:Special::Gz,     max:2*GB,    min:0,   desc:"Gzip Archive" },
    Sig { name:"7z",      ext:"7z",   magic:b"7z\xBC\xAF'\x1C",           moff:0, em:None,                     em_last:false, em_trail:0, special:Special::None,   max:4*GB,    min:0,   desc:"7-Zip Archive" },
    Sig { name:"rar",     ext:"rar",  magic:b"Rar!\x1a\x07",               moff:0, em:None,                     em_last:false, em_trail:0, special:Special::None,   max:4*GB,    min:0,   desc:"RAR Archive" },
    Sig { name:"sqlite",  ext:"db",   magic:b"SQLite format 3\x00",        moff:0, em:None,                     em_last:false, em_trail:0, special:Special::Sqlite, max:2*GB,    min:0,   desc:"SQLite Database" },
    Sig { name:"eml",     ext:"eml",  magic:b"From:",                      moff:0, em:None,                     em_last:false, em_trail:0, special:Special::None,   max:50*MB,   min:64,  desc:"Email (EML)" },
];

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
    let cdsz = u32::from_le_bytes(data[eocd+12..eocd+16].try_into().unwrap_or([0;4])) as usize;
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

// ─── Carving engine ───────────────────────────────────────────────────────────

pub struct Finding {
    pub offset:  usize,
    pub name:    &'static str,
    pub ext:     &'static str,
    pub desc:    &'static str,
    pub size:    usize,
    pub path:    PathBuf,
    pub trunc:   bool,
    pub sha256:  String,
}

pub fn carve(
    src:   &[u8],
    outdir: &Path,
    types:  Option<&[String]>,
    quiet:  bool,
) -> Result<Vec<Finding>> {
    fs::create_dir_all(outdir)?;

    let sigs: Vec<&Sig> = SIGS.iter()
        .filter(|s| types.map_or(true, |t| t.iter().any(|n| n == s.name)))
        .collect();

    let mut out:    Vec<Finding>               = Vec::new();
    let mut seen:   HashSet<usize>             = HashSet::new();
    let mut ctr:    HashMap<&'static str, usize> = HashMap::new();

    for sig in sigs {
        let finder = memmem::Finder::new(sig.magic);
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
                    if let Some(em) = sig.em {
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
                Special::Jpeg => jpeg_ok(carved),
                Special::Gz   => gz_ok(carved),
                Special::Aac  => aac_ok(carved),
                Special::Bmp  => bmp_ok(carved),
                _             => true,
            };
            if !valid { continue; }

            // ── subtype + size-field truncation ───────────────────────────────
            let (fext, fdesc, fdata): (&'static str, &'static str, &[u8]) = match sig.special {
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
                    (sig.ext, sig.desc, trim)
                }
                Special::Tiff => {
                    let trim = match tiff_size(carved) {
                        Some(n) if n <= carved.len() => { trunc = false; &carved[..n] }
                        _ => carved,
                    };
                    (sig.ext, sig.desc, trim)
                }
                Special::Sqlite => {
                    let trim = match sqlite_size(carved) {
                        Some(n) if n <= carved.len() => { trunc = false; &carved[..n] }
                        _ => carved,
                    };
                    (sig.ext, sig.desc, trim)
                }
                _ => (sig.ext, sig.desc, carved),
            };

            // ── write ─────────────────────────────────────────────────────────
            let n = { let c = ctr.entry(fext).and_modify(|x| *x += 1).or_insert(1); *c };
            let fname = format!("{fext}_{n:04}.{fext}");
            let out_path = outdir.join(&fname);
            fs::write(&out_path, fdata)
                .with_context(|| format!("writing {}", out_path.display()))?;

            let mut h = Sha256::new();
            h.update(fdata);
            let sha = format!("{:x}", h.finalize());

            if !quiet {
                eprintln!("{:>10}  {:>10}B  0x{:08x}  {}", fext, fdata.len(), start, fname);
            }

            seen.insert(start);
            out.push(Finding {
                offset: start, name: sig.name, ext: fext, desc: fdesc,
                size: fdata.len(), path: out_path, trunc, sha256: sha,
            });
        }
    }
    Ok(out)
}

// ─── CLI ──────────────────────────────────────────────────────────────────────

fn usage() {
    eprintln!("pala — filesystem-agnostic file carver\n");
    eprintln!("Usage:  pala <source> <outdir> [OPTIONS]\n");
    eprintln!("Options:");
    eprintln!("  -t, --types <csv>   File types to recover (default: all)");
    eprintln!("  -l, --list          List available types and exit");
    eprintln!("  -q, --quiet         Suppress per-file output");
    eprintln!("      --json          JSON summary to stdout");
    eprintln!("  -h, --help          Show this help\n");
    eprintln!("Examples:");
    eprintln!("  pala disk.img recovered/");
    eprintln!("  pala /dev/sdb recovered/ -t jpeg,png,pdf");
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let mut src:   Option<String>      = None;
    let mut out:   Option<String>      = None;
    let mut types: Option<Vec<String>> = None;
    let mut quiet = false;
    let mut json  = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-h"|"--help"  => { usage(); return Ok(()); }
            "-l"|"--list"  => {
                println!("{:<12}  {:<6}  {}", "NAME", "EXT", "DESCRIPTION");
                println!("{}", "-".repeat(48));
                for s in SIGS { println!("{:<12}  {:<6}  {}", s.name, s.ext, s.desc); }
                return Ok(());
            }
            "-q"|"--quiet" => quiet = true,
            "--json"       => json  = true,
            "-t"|"--types" => {
                i += 1;
                if i >= args.len() { anyhow::bail!("--types requires a value"); }
                types = Some(args[i].split(',').map(|s| s.trim().to_string()).collect());
            }
            a if a.starts_with("--types=") => {
                types = Some(a[8..].split(',').map(|s| s.trim().to_string()).collect());
            }
            a => {
                if src.is_none() { src = Some(a.to_string()); }
                else if out.is_none() { out = Some(a.to_string()); }
                else { anyhow::bail!("unexpected argument: {a}"); }
            }
        }
        i += 1;
    }

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

    if !quiet { eprintln!("pala: scanning {src_path} ({size} bytes)"); }

    let mmap = unsafe { MmapOptions::new().len(size).map(&file) }
        .with_context(|| format!("mmap {src_path}"))?;

    let outdir = PathBuf::from(&out_path);
    let findings = carve(&mmap, &outdir, types.as_deref(), quiet)?;

    if json {
        print!("[");
        for (idx, f) in findings.iter().enumerate() {
            if idx > 0 { print!(","); }
            print!(
                r#"{{"offset":{},"type":"{}","extension":"{}","description":"{}","size":{},"path":"{}","truncated":{},"sha256":"{}"}}"#,
                f.offset, f.name, f.ext, f.desc,
                f.size, f.path.display(), f.trunc, f.sha256,
            );
        }
        println!("]");
    }

    if !quiet { eprintln!("pala: recovered {} file(s) → {}", findings.len(), out_path); }
    Ok(())
}
