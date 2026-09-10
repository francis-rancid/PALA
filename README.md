# PALA - File carver and data recovery tool

PALA recovers deleted files from any disk in a 957KB binary, because the 200MB recovery tool you just downloaded probably overwrote them.

Most recovery tools are 200MB or more. That is 200MB of crucial disk space - the same space your deleted files still occupy. PALA is a single static binary with no runtime deps and no installer. Drop it on a USB stick, plug it in, and run it without writing a single byte to the target drive.

Claude Code accidentally deleted something important? Run PALA against the disk, pipe the JSON output back to your AI, and let it tell you exactly what it found and which file is the one you lost. The whole recovery session stays inside your terminal, inside your conversation, without switching tools or losing context. Most recovery tools give you a GUI and a progress bar. PALA gives you structured JSON and gets out of the way.

Works on Linux and Windows. Steal it. Make it better. Use it for reverse engineering.

## Use Case

PALA runs three recovery stages in sequence. Each stage deduplicates against all prior stages by SHA256 - nothing is written twice.

**Signature carving** - scans raw bytes for known file headers across 60 file types. Works on any source: intact filesystem, corrupted partition, formatted drive, raw block device, or disk image. No filesystem metadata required.

**Filesystem-aware inode recovery** (`--filesystem`) - walks live or partially-intact filesystem metadata to recover files by inode rather than magic bytes. Catches files with no recognizable header. Supports ext2/3/4, NTFS, FAT32, and APFS. NTFS stage-2 parses `$DATA` run lists from carved MFT entries and assembles file content directly from cluster offsets. FAT32 stage-2 recovers deleted entries whose FAT chain has been cleared using contiguous cluster prediction.

**Container unpacking** (`--container-depth`) - extracts member files from carved ZIP, DOCX, XLSX, PPTX, JAR, and APK archives. Catches embedded images and attachments that have no independent offset in the raw byte stream.

**Entropy classification** - classifies every 512-byte sector by Shannon entropy and reports zero sectors (unwritten or wiped), high-entropy sectors (encrypted volumes, compressed regions), and normal sectors. `--skip-high-entropy` drops false-positive hits from encrypted regions automatically.

**Pipeline integration** - `--json` produces machine-readable output with per-finding offset, size, SHA256, quality flag, and source stage. Pipe it directly into Claude Code, Codex, or any other AI to triage findings, prioritize results, and reconstruct what happened.

## Installation

Requires Rust stable. No other build dependencies.

```bash
git clone https://github.com/sshpie/PALA.git
cd PALA
cargo build --release
```

Binary lands at `target/release/pala`. Copy it to a USB stick or anywhere on your PATH.

For filesystem-aware recovery (`--filesystem`), The Sleuth Kit (`tsk_recover`, `fls`) must be available at runtime. If absent, PALA falls back to signature carving only.

```bash
# Debian/Ubuntu
sudo apt install sleuthkit

# macOS
brew install sleuthkit
```

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
      --triage-mode <mode>  Limit types to a preset group (media, documents, executables,
                            archives, email, windows, databases, memory, filesystem)
      --filesystem <fs>     Filesystem-aware inode recovery (auto, ext2, ntfs, apfs, fat32)
      --skip-high-entropy   Skip candidates whose start sector has Shannon entropy > 7.5
      --container-depth     Extract member files from carved ZIP/DOCX/XLSX/PPTX containers
      --no-fat32-stage2     Disable FAT32 deleted-entry recovery stage
  -h, --help                Show this help
```

```bash
# Recover everything from a disk image
pala disk.img recovered/

# Recover only JPEG and PDF from a live device
sudo pala /dev/sdb recovered/ -t jpeg,pdf

# Filesystem-aware recovery (finds files by inode, not just magic bytes)
sudo pala /dev/sdb recovered/ --filesystem=auto

# JSON output piped to an AI or jq
pala disk.img out/ --json | jq '.findings[] | {ext, size, offset}'

# Skip encrypted sectors, extract ZIP members
pala disk.img out/ --skip-high-entropy --container-depth

# Forensic triage - pull only Windows artifacts
pala disk.img out/ --triage-mode=windows
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

## JSON output

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

`source` is `null` for sig-carve results, `"mft:<hex>"` for NTFS stage-2, `"fat32:<hex>"` for FAT32 stage-2, `"inode:<path>"` for TSK inode recovery, and `"zip:<hex>:<member>"` for container members.

`quality` is `Complete` (end marker found), `Partial` (truncated at max-size), or `Fragmented` (assembled from non-contiguous clusters).

## Custom signatures

PALA supports loadable signature corpus files for proprietary or niche file types.

```bash
pala disk.img out/ -c custom.pala
```

A corpus file contains additional signatures in PALA's binary format. The `serialize_corpus()` function in `corpus.rs` produces valid corpus bytes from a `Vec<Signature>`.

## Known issues

- Filesystem-aware recovery requires The Sleuth Kit at runtime. If TSK is absent, PALA silently falls back to signature carving only.
- FAT32 stage-2 cluster prediction assumes unfragmented files. Heavily fragmented volumes will produce incomplete recoveries.
- Files whose sectors have been overwritten by new data cannot be recovered regardless of method.
- Full-disk encryption: PALA cannot recover from an encrypted volume without the key. The entropy survey will report a high percentage of high-entropy sectors, which is a reliable indicator.

## Getting help

Open an issue at [github.com/sshpie/PALA](https://github.com/sshpie/PALA/issues). Include the error output and the PALA version (`pala --version`).

## Getting involved

Contributions welcome. The most useful additions are new file signatures - if you have a proprietary format or an obscure type that PALA misses, open a PR adding it to the `SIGS` array in `src/main.rs`. Each signature needs a magic byte sequence, an optional end marker or size field strategy, a minimum and maximum size, and a short description.

## Credits and references

- [Claude Code](https://claude.ai/code) - contributed signature parsing, filesystem recovery stages, entropy classification, ZIP container depth, and test suite

## License

MIT OR Apache-2.0
