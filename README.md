# PALA

Filesystem-agnostic file carver. Recovers deleted files from raw disk images and block devices by scanning for known file signatures (magic bytes) — no filesystem metadata required.

Single static binary. ~423KB. No runtime deps. Run it from a USB stick without installing anything on the target system.

## Usage

```
pala <source> <outdir> [OPTIONS]

Options:
  -t, --types <csv>   File types to recover (default: all)
  -l, --list          List available types and exit
  -q, --quiet         Suppress per-file output
      --json          Write JSON summary to stdout
  -h, --help          Show this help
```

```sh
# Recover everything from a disk image
pala disk.img recovered/

# Recover only JPEG and PDF from a live device
pala /dev/sdb recovered/ -t jpeg,pdf

# JSON summary piped to jq
pala disk.img out/ --json | jq '.[] | {ext, size, offset}'
```

## Supported types

| Type | Extension | Description |
|------|-----------|-------------|
| jpeg | jpg | JPEG Image |
| png | png | PNG Image |
| gif87a | gif | GIF Image (87a) |
| gif89a | gif | GIF Image (89a) |
| bmp | bmp | BMP Image |
| tiff_le | tif | TIFF Image (little-endian) |
| tiff_be | tif | TIFF Image (big-endian) |
| psd | psd | Photoshop Document |
| riff | wav/avi/webp | RIFF Container (subtype auto-detected) |
| mkv | mkv | MKV/WebM Video |
| mp4 | mp4 | MP4/MOV Video |
| mp3_id3 | mp3 | MP3 Audio (ID3) |
| flac | flac | FLAC Audio |
| aac | aac | AAC Audio (ADTS) |
| pdf | pdf | PDF Document |
| rtf | rtf | RTF Document |
| zip | zip/docx/xlsx/pptx | ZIP / Office Open XML (subtype auto-detected) |
| ole2 | doc | OLE2 Document (DOC/XLS/PPT) |
| gz | gz | Gzip Archive |
| 7z | 7z | 7-Zip Archive |
| rar | rar | RAR Archive |
| sqlite | db | SQLite Database |
| eml | eml | Email (EML) |

## How it works

PALA scans raw bytes for file header patterns (magic bytes). For each match it extracts the file using the appropriate algorithm:

- **End marker**: JPEG (FF D9), PNG (IEND chunk), GIF (00 3B), PDF (%%EOF), RTF (})
- **Size field**: WAV/WEBP reads RIFF chunk size at offset 4; BMP reads size at offset 2; SQLite computes `page_size × page_count` from the 100-byte header; TIFF follows the IFD chain
- **Container**: ZIP finds the EOCD record then inspects the central directory for Office filenames

Short-magic types (2-byte headers like `FF F1` for AAC, `1F 8B` for GZ, `BM` for BMP) are validated against structural fields before extraction to suppress false positives from random data.

## Why keep it small

The act of downloading a recovery tool can overwrite the deleted data you are trying to recover. A 423KB binary fits on any USB stick and never touches the target drive during download.

## Build

```sh
cargo build --release
# binary at target/release/pala
```

Requires Rust stable. No other build dependencies.

## License

MIT OR Apache-2.0
