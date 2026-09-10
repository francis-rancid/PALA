# PALA

File carver and data recovery tool. Recovers deleted files from raw disk images and block devices using three stacked recovery layers: signature carving, filesystem-aware inode recovery, and container unpacking.

Single static binary. ~957KB. No runtime deps. Run it from a USB stick.

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
      --triage-mode <mode>  Limit types to a preset group (media, documents, executables, archives, email, windows, databases, memory, filesystem)
      --filesystem <fs>     Filesystem-aware inode recovery (auto, ext2, ntfs, apfs, fat32, hfs)
      --skip-high-entropy   Skip candidates whose start sector has Shannon entropy > 7.5
      --container-depth     Extract member files from carved ZIP/DOCX/XLSX/PPTX containers
      --no-fat32-stage2     Disable FAT32 deleted-entry recovery stage
  -h, --help                Show this help
```

```sh
# Recover everything from a disk image
pala disk.img recovered/

# Recover only JPEG and PDF from a live device
sudo pala /dev/sdb recovered/ -t jpeg,pdf

# Filesystem-aware recovery (finds files by inode, not just magic bytes)
sudo pala /dev/sdb recovered/ --filesystem=auto

# JSON summary
pala disk.img out/ --json | jq '.findings[] | {ext, size, offset}'

# Skip encrypted/compressed sectors, extract ZIP members
pala disk.img out/ --skip-high-entropy --container-depth
```

## How it works

PALA runs three recovery stages in sequence. Each stage deduplicates against all prior stages by SHA256 — nothing is written twice.

### Stage 1 — Signature carving

Scans raw bytes for known file headers. Works on any source: intact filesystem, corrupted partition, raw block device, memory dump. No filesystem metadata required.

Extraction methods by file type:
- **End marker** — JPEG (`FF D9`), PNG (IEND chunk), GIF (`00 3B`), PDF (`%%EOF`), RTF (`}`)
- **Size field** — WAV/WEBP reads RIFF chunk size at offset 4; BMP at offset 2; SQLite from `page_size × page_count` in the 100-byte header; TIFF follows the IFD chain; Registry hives read `hive_bins_size` at offset 40; Prefetch reads `file_size` at offset 12; DEX reads `file_size` at offset 32
- **Container** — ZIP locates the EOCD record and inspects the central directory for Office filenames (DOCX/XLSX/PPTX)

Short-magic types (2-byte headers like `FF F1` for AAC, `1F 8B` for GZ, `BM` for BMP) are validated against structural fields before extraction to suppress false positives.

### Stage 2 — Filesystem-aware recovery (`--filesystem`)

Walks filesystem metadata to recover files by inode rather than magic bytes. Catches files with no recognizable header and unallocated inodes whose data clusters are still intact.

- **ext2/3/4** — full inode walk via `tsk_recover`
- **NTFS** — inode walk via TSK + MFT stage-2: parses `$DATA` attribute run lists from carved MFT entries and assembles file content from cluster offsets directly in the source image. Handles non-resident data regardless of fragmentation.
- **FAT32** — deleted-entry recovery: scans directory entries marked `0xE5` (deleted), reads `first_cluster` and `size` from the surviving entry, chains clusters via the FAT (falls back to contiguous cluster prediction when FAT entries are cleared). Recovers files deleted from consumer SD cards and USB drives without intact FAT chains.
- **APFS** — inode walk via TSK (`--filesystem=apfs`; requires TSK with APFS support)

`--filesystem=auto` probes the source and selects the appropriate driver.

### Stage 3 — Container unpacking (`--container-depth`)

Opens carved ZIP, DOCX, XLSX, PPTX, JAR, and APK files with the `zip` crate and extracts member files. Depth = 1. Catches embedded images, attachments, and sub-documents that have no independent offset in the raw byte stream.

### Entropy classification

`--skip-high-entropy` computes Shannon entropy per 512-byte sector and discards candidates whose start sector exceeds H = 7.5. Eliminates false-positive hits from encrypted volumes, compressed archives, and encrypted swap.

An entropy survey runs unconditionally after Stage 1 and is reported in `session_summary.entropy_survey` when `--json` is set:

```json
"entropy_survey": {
  "zero_sectors": 1024,
  "high_entropy_sectors": 512,
  "total_sectors": 8192,
  "high_entropy_skipped": 3
}
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

`--json` writes a structured summary to stdout. Individual findings include `source` to indicate which recovery stage produced them:

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

`source` is `null` for sig-carve results, `"mft:<hex_offset>"` for NTFS stage-2, `"fat32:<hex_offset>"` for FAT32 stage-2, `"inode:<path>"` for TSK inode recovery, and `"zip:<hex_offset>:<member_name>"` for container members.

`quality` is `Complete` (end marker found), `Partial` (truncated at max-size), or `Fragmented` (assembled from non-contiguous clusters).

## Custom signatures

```sh
pala disk.img out/ -c custom.pala
```

A corpus file contains additional signatures in PALA's binary format. The `serialize_corpus()` function in `corpus.rs` produces valid corpus bytes from a `Vec<Signature>`.

## Build

```sh
cargo build --release
# binary at target/release/pala (~957KB, statically linked)
```

Requires Rust stable. No other build dependencies. Filesystem-aware recovery (`--filesystem`) requires The Sleuth Kit (`tsk_recover`, `fls`) at runtime — if absent, PALA falls back to sig carving only.

## License

MIT OR Apache-2.0
