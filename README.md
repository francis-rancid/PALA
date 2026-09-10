# PALA

PALA recovers deleted files from any disk in a 957KB binary, because the 200MB recovery tool you just downloaded probably overwrote them.

Single static binary. No runtime deps. No installer. Works on Linux and Windows. Drop it on a USB stick, plug it in, and run it without touching the target drive. Steal it. Make it better. Use it for reverse engineering.

Claude Code accidentally deleted something important? Run PALA against the disk, pipe the JSON output back to your AI, and let it tell you exactly what it found and which file is the one you lost. The whole recovery session stays inside your terminal, inside your conversation, without switching tools or losing context.

Most recovery tools give you a GUI and a progress bar. PALA gives you structured JSON output and gets out of the way. That means you can pipe it directly into Claude Code, Codex, or any other AI and let the model triage what was found, prioritize the files worth looking at, and tell you what happened. Run it in a script. Automate it. Chain it with anything. The `--json` flag makes PALA a first-class citizen in any pipeline, not a dead end you have to click through.

Under the hood PALA runs three recovery stages in sequence - signature carving across 60 file types, filesystem-aware inode recovery for ext2/3/4, NTFS, FAT32, and APFS, and container unpacking for ZIP-based formats. Every stage deduplicates against the others by SHA256 so nothing is written twice. It also classifies every 512-byte sector by Shannon entropy, which tells you whether you are looking at normal data, wiped space, or an encrypted volume before you commit to a full scan. Custom signature corpora let you add proprietary file types without touching the source.

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

Requires Rust stable. No other build dependencies. Filesystem-aware recovery (`--filesystem`) requires The Sleuth Kit (`tsk_recover`, `fls`) at runtime - if absent, PALA falls back to sig carving only.

## License

MIT OR Apache-2.0
