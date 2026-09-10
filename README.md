# PALA

Filesystem-agnostic file carver. Recovers deleted files from raw disk images and block devices by scanning for known file signatures (magic bytes) — no filesystem metadata required.

Single static binary. ~507KB. No runtime deps. Run it from a USB stick without installing anything on the target system.

## Usage

```
pala <source> <outdir> [OPTIONS]

Options:
  -t, --types <csv>    File types to recover (default: all)
  -c, --corpus <path>  Load extra signatures from a PALA corpus file
  -l, --list           List available types and exit
  -q, --quiet          Suppress all output (use with --json)
      --json           Write structured JSON summary to stdout
  -h, --help           Show this help
```

```sh
# Recover everything from a disk image
pala disk.img recovered/

# Recover only JPEG and PDF from a live device
pala /dev/sdb recovered/ -t jpeg,pdf

# JSON summary piped to jq
pala disk.img out/ --json | jq '.findings[] | {ext, size, offset}'

# Load custom signatures alongside built-in ones
pala disk.img out/ -c my_sigs.pala
```

## Supported types

| Type | Extension | Description |
|------|-----------|-------------|
| jpeg | jpg | JPEG Image |
| png | png | PNG Image |
| gif87a / gif89a | gif | GIF Image |
| bmp | bmp | BMP Image |
| tiff_le / tiff_be | tif | TIFF Image |
| psd | psd | Photoshop Document |
| riff | wav / avi / webp | RIFF Container (subtype auto-detected) |
| mkv | mkv | MKV/WebM Video |
| mp4 | mp4 | MP4/MOV Video |
| mp3_id3 | mp3 | MP3 Audio (ID3) |
| flac | flac | FLAC Audio |
| aac | aac | AAC Audio (ADTS) |
| pdf | pdf | PDF Document |
| rtf | rtf | RTF Document |
| zip | zip / docx / xlsx / pptx | ZIP and Office Open XML (subtype auto-detected) |
| ole2 | doc | Legacy Office (DOC / XLS / PPT) |
| gz | gz | Gzip Archive |
| 7z | 7z | 7-Zip Archive |
| rar | rar | RAR Archive |
| sqlite | db | SQLite Database |
| sqlite_wal | db-wal | SQLite Write-Ahead Log |
| eml | eml | Email (EML) |
| evtx | evtx | Windows Event Log |
| regf | dat | Windows Registry Hive |
| lnk | lnk | Windows Shell Link |
| pf | pf | Windows Prefetch |
| thumbcache | db | Windows Thumbcache |
| hibr | bin | Windows Hibernate File |
| pagedump | dmp | Windows Memory Dump (BSOD) |
| bplist | plist | Apple Binary Property List |
| dex | dex | Android Dalvik Executable |

## How it works

PALA scans raw bytes for file header patterns (magic bytes). For each match it extracts the file using the appropriate algorithm:

- **End marker**: JPEG (FF D9), PNG (IEND chunk), GIF (00 3B), PDF (%%EOF), RTF (})
- **Size field**: WAV/WEBP reads RIFF chunk size at offset 4; BMP reads size at offset 2; SQLite computes `page_size × page_count` from the 100-byte header; TIFF follows the IFD chain; Registry hives read `hive_bins_size` at offset 40; Prefetch reads file_size at offset 12; DEX reads file_size at offset 32
- **Container**: ZIP finds the EOCD record then inspects the central directory for Office filenames

Short-magic types (2-byte headers like `FF F1` for AAC, `1F 8B` for GZ, `BM` for BMP) are validated against structural fields before extraction to suppress false positives from random data.

JPEG files where the marker walk hits the max-size window without finding an EOI marker are written with a `_partial` suffix so you know the recovered file is incomplete.

## Custom signatures

PALA supports loadable signature corpus files (`-c custom.pala`). A corpus file contains one or more additional file signatures in PALA's binary format. The `corpus.rs` module documents the format; `serialize_corpus()` in that module produces valid corpus bytes from a `Vec<Signature>`.

## Why keep it small

The act of downloading a recovery tool can overwrite the deleted data you are trying to recover. A 507KB binary fits on any USB stick and never touches the target drive during download.

## Build

```sh
cargo build --release
# binary at target/release/pala
```

Requires Rust stable. No other build dependencies.

## License

MIT OR Apache-2.0
