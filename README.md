<h1 align="center">PALA</h1>

<div align="center">
<img src="https://img.shields.io/badge/Rust-stable-orange?style=flat-square&logo=rust&logoColor=white" alt="Rust stable">
<img src="https://img.shields.io/badge/binary-957KB-brightgreen?style=flat-square" alt="957KB">
<img src="https://img.shields.io/badge/dependencies-none-brightgreen?style=flat-square" alt="No runtime deps">
<a href="https://github.com/sshpie/PALA/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/sshpie/PALA/ci.yml?label=tests&style=flat-square" alt="tests"></a>
<a href="https://github.com/sshpie/PALA/blob/main/LICENSE-MIT"><img src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue?style=flat-square" alt="license"></a>
</div>

<br />

<div align="center">
File carver and data recovery tool.<br>
Recovers deleted files from any disk in a 957KB binary, because the 200MB recovery tool you just downloaded probably overwrote them.
</div>

---

## Table of Contents

1. [Overview](#overview)
2. [Features](#features)
3. [Installation](#installation)
4. [Quick Start](#quick-start)
5. [Usage](#usage)
6. [Supported Types](#supported-types)
7. [JSON Output](#json-output)
8. [Custom Signatures](#custom-signatures)
9. [Known Issues](#known-issues)
10. [Getting Help](#getting-help)
11. [Contributing](#contributing)
12. [Credits](#credits)

---

## Overview

Most recovery tools are 200MB or more. That is 200MB of crucial disk space - the same space your deleted files still occupy. PALA is a single static binary with no runtime deps and no installer. Drop it on a USB stick, plug it in, and run it without writing a single byte to the target drive.

Claude Code accidentally deleted something important? Run PALA against the disk, pipe the JSON output back to your AI, and let it tell you exactly what it found and which file is the one you lost. The whole recovery session stays inside your terminal, inside your conversation, without switching tools or losing context.

PALA runs three recovery stages in sequence. Each stage deduplicates against all prior stages by SHA256 - nothing is written twice.

Works on Linux and Windows. Steal it. Make it better. Use it for reverse engineering.

---

## Features

- **Signature carving** - scans raw bytes for known file headers across 60 file types; works on any source with no filesystem metadata required
- **Filesystem-aware inode recovery** (`--filesystem`) - walks live or partially-intact filesystem metadata for ext2/3/4, NTFS, FAT32, and APFS
- **NTFS MFT stage-2** - parses `$DATA` run lists from carved MFT entries and assembles file content directly from cluster offsets; handles fragmented files
- **FAT32 deleted-entry recovery** - recovers deleted files whose FAT chain has been cleared using contiguous cluster prediction
- **Container unpacking** (`--container-depth`) - extracts member files from carved ZIP, DOCX, XLSX, PPTX, JAR, and APK archives
- **Entropy classification** - classifies every 512-byte sector by Shannon entropy; `--skip-high-entropy` drops false-positive hits from encrypted volumes automatically
- **Triage modes** - built-in presets for media, documents, executables, archives, email, windows, databases, memory, and filesystem types
- **SHA256 deduplication** - no file is written twice regardless of which stage finds it
- **Metadata extraction** (`--meta`) - JPEG EXIF, PNG headers, ELF/PE fields, MFT cluster runs, SQLite schema
- **Pipeline integration** - `--json` produces machine-readable output per finding; pipe directly into Claude Code, Codex, or any downstream tool
- **957KB binary** - single statically linked executable; no installer, no runtime, no dependencies

---

## Installation

Requires Rust stable. No other build dependencies.

```bash
git clone https://github.com/sshpie/PALA.git
cd PALA
cargo build --release
```

Binary lands at `target/release/pala`. Copy it to a USB stick or anywhere on your PATH.

For filesystem-aware recovery (`--filesystem`), The Sleuth Kit must be available at runtime. If absent, PALA falls back to signature carving only.

```bash
# Debian/Ubuntu
sudo apt install sleuthkit

# macOS
brew install sleuthkit
```

---

## Quick Start

```bash
# Recover everything from a disk image
pala disk.img recovered/

# Recover only JPEG and PDF from a live device
sudo pala /dev/sdb recovered/ -t jpeg,pdf

# Filesystem-aware recovery - finds files by inode, not just magic bytes
sudo pala /dev/sdb recovered/ --filesystem=auto

# AI-assisted recovery - pipe findings to Claude Code or jq
pala disk.img out/ --json | jq '.findings[] | {ext, size, offset}'

# Skip encrypted sectors, unpack ZIP members
pala disk.img out/ --skip-high-entropy --container-depth

# Forensic triage - pull only Windows artifacts
pala disk.img out/ --triage-mode=windows
```

---

## Usage

```
pala <source> <outdir> [OPTIONS]

Options:
  -t, --types <csv>         File types to recover (default: all)
  -c, --corpus <path>       Load extra signatures from a PALA corpus file
  -l, --list                List available types and exit
  -q, --quiet               Suppress progress output (use with --json)
      --json                Write structured JSON summary to stdout
      --meta                Extract file metadata into JSON output
      --max-size <bytes>    Maximum size per recovered file (default: type-specific)
      --min-size <bytes>    Minimum size per recovered file
  -n, --count <n>           Stop after recovering N files
      --triage-mode <mode>  Limit types to a preset group:
                            media, documents, executables, archives,
                            email, windows, databases, memory, filesystem
      --filesystem <fs>     Filesystem-aware inode recovery:
                            auto, ext2, ntfs, apfs, fat32
      --skip-high-entropy   Skip candidates whose start sector has Shannon entropy > 7.5
      --container-depth     Extract member files from carved ZIP/DOCX/XLSX/PPTX containers
      --no-fat32-stage2     Disable FAT32 deleted-entry recovery stage
  -h, --help                Show this help
```

---

## Supported Types

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
| hibr / hibr_upper | bin | Windows Hibernate File |
| wake_lower / wake_upper | bin | Windows Hibernate Resume |
| pagedump / pagedu64 | dmp | Windows Memory Dump (BSOD) |
| bplist | plist | Apple Binary Property List |
| dex | dex | Android Dalvik Executable |
| ntfs_mft | mft | NTFS MFT Entry |
| fat32_fsinfo | fsinfo | FAT32 FSINFO Sector |
| ext2_sb | sb | Ext2/3/4 Superblock |
| ufs1_sb / ufs2_sb | ufs | UFS1/UFS2 Superblock |
| lime | lime | Linux Memory Acquisition (LiME) |
| hpak | hpak | HBGary Memory Acquisition (HPAK) |
| elf | elf | ELF Binary |
| pe | exe | PE/MZ Executable |
| mng | mng | MNG Animation |
| jng | jng | JNG Image |

---

## JSON Output

`--json` writes a structured summary to stdout. The `source` field on each finding identifies which recovery stage produced it.

```json
{
  "source": "/dev/sdb",
  "source_bytes": 17179869184,
  "elapsed_ms": 8200,
  "found": 47,
  "findings": [
    {
      "offset": 4096,
      "type": "jpeg",
      "extension": "jpg",
      "size": 544,
      "path": "recovered/jpg_0001.jpg",
      "sha256": "a3f2...",
      "quality": "Complete",
      "source": null
    },
    {
      "offset": 0,
      "type": "jpeg",
      "extension": "jpg",
      "size": 131072,
      "path": "recovered/jpg_0002.jpg",
      "sha256": "b7c4...",
      "quality": "Complete",
      "source": "mft:00004000"
    },
    {
      "offset": 0,
      "type": "jpeg",
      "extension": "jpg",
      "size": 65536,
      "path": "recovered/jpg_0003.jpg",
      "sha256": "d1e9...",
      "quality": "Fragmented",
      "source": "fat32:00000200"
    }
  ],
  "session_summary": {
    "entropy_survey": {
      "zero_sectors": 1024,
      "high_entropy_sectors": 0,
      "total_sectors": 32768,
      "high_entropy_skipped": 0
    }
  }
}
```

| Field | Values |
|-------|--------|
| `source` | `null` (sig carve), `"mft:<hex>"` (NTFS stage-2), `"fat32:<hex>"` (FAT32 stage-2), `"inode:<path>"` (TSK), `"zip:<hex>:<member>"` (container) |
| `quality` | `Complete` (end marker found), `Partial` (truncated at max-size), `Fragmented` (non-contiguous clusters) |

---

## Custom Signatures

PALA supports loadable signature corpus files for proprietary or niche file types.

```bash
pala disk.img out/ -c custom.pala
```

A corpus file contains additional signatures in PALA's binary format. The `serialize_corpus()` function in `src/corpus.rs` produces valid corpus bytes from a `Vec<Signature>`.

---

## Known Issues

- Filesystem-aware recovery requires The Sleuth Kit at runtime; if absent, PALA silently falls back to signature carving only
- FAT32 stage-2 cluster prediction assumes unfragmented files; heavily fragmented volumes will produce incomplete recoveries
- Files whose sectors have been overwritten by new data cannot be recovered regardless of method
- Full-disk encryption: PALA cannot recover from an encrypted volume without the key; the entropy survey will report a high percentage of high-entropy sectors as an indicator

---

## Getting Help

Open an issue at [github.com/sshpie/PALA/issues](https://github.com/sshpie/PALA/issues). Include the error output and PALA version (`pala --version`).

---

## Contributing

Contributions welcome. The most useful additions are new file signatures. To add one, extend the `SIGS` array in `src/main.rs` with a magic byte sequence, an optional end marker or size field strategy, a minimum and maximum size, and a short description.

---

## Credits

- [Claude Code](https://claude.ai/code) - filesystem recovery stages, entropy classification, ZIP container depth, MFT run list parsing, FAT32 deleted-entry recovery, and test suite

---

MIT OR Apache-2.0
