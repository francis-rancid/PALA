<div align="center">

<h1>PALA</h1>

<p><strong>Taking up less space, to recover more.</strong></p>

<p>Download it when you least expect to need it.<br>
Keep it on hand for when Claude Code accidentally deletes that directory.</p>

<p>
<a href="#features--benefits">Features & Benefits</a> &nbsp;|&nbsp;
<a href="#the-pala-difference">What Makes It Different</a> &nbsp;|&nbsp;
<a href="#use-cases">Use Cases</a> &nbsp;|&nbsp;
<a href="#claude-code-integration">Claude Code</a> &nbsp;|&nbsp;
<a href="#installation">Installation</a>
</p>

</div>

---

## Features & Benefits

<img src="assets/accent.svg">

**Smallest full-featured file carver available.**

The 200MB recovery tool you just downloaded probably overwrote the files you are trying to recover. PALA is 988KB. Drop it on a USB stick. Run it without touching a single byte on the target drive.

Three recovery stages run in sequence. Each deduplicates against all prior stages by SHA256 - nothing is written twice.

<table>
<tr>
<td width="33%" valign="top">
<strong>Stage 1 - Signature Carving</strong><br><br>
Scans raw bytes for known file headers across 70+ types. No filesystem metadata required. Works on formatted, corrupted, or wiped drives.
</td>
<td width="33%" valign="top">
<strong>Stage 2 - Inode Recovery</strong><br><br>
Walks live filesystem metadata for ext2/3/4, NTFS, FAT32, and APFS. Recovers files whose directory entries still exist even after deletion.
</td>
<td width="33%" valign="top">
<strong>Stage 3 - Cluster Analysis</strong><br><br>
Parses NTFS MFT run lists and FAT32 deleted entries from raw cluster offsets. Recovers fragmented files that carving alone misses.
</td>
</tr>
</table>

- **70+ file types** across media, documents, archives, forensic artifacts, firmware images, and memory captures
- **Precision size parsers** for FLAC, LiME, SquashFS, U-Boot, FIT, and cramfs - header-derived exact boundaries, no static caps
- **Entropy classification** identifies and optionally skips encrypted sectors to eliminate false-positive hits
- **Container unpacking** (`--container-depth`) extracts member files from carved ZIP, DOCX, XLSX, JAR, and APK archives
- **Sector-aligned scan** (`--align=N`) restricts matches to block-aligned offsets for raw block device forensics
- **Structured JSON output** (`--json`) for automation, pipelines, and AI analysis

---

## The PALA Difference

<img src="assets/accent.svg">

<table>
<tr>
<td width="25%" align="center" valign="top">
<br>
<strong>Carving Without Compromise &raquo;</strong>
<br><br>
988KB static binary. No installer. No runtime.
<br><br>

- Fits on any USB stick
- Zero writes to the source drive
- Standard recovery tools are 200MB or more - the same space your deleted files occupy

</td>
<td width="25%" align="center" valign="top">
<br>
<strong>Precision Size Parsers &raquo;</strong>
<br><br>
Static caps produce garbage tails and truncated recoveries.
<br><br>

- FLAC: walks METADATA_BLOCK chain, reads STREAMINFO total_samples
- LiME: reads segment headers to compute exact dump extent
- SquashFS, U-Boot, FIT, cramfs: all header-derived

</td>
<td width="25%" align="center" valign="top">
<br>
<strong>Three-Stage Recovery &raquo;</strong>
<br><br>
Carving alone misses fragmented files.
<br><br>

- NTFS MFT $DATA run list parsing assembles files from non-contiguous clusters
- FAT32 deleted-entry recovery uses contiguous cluster prediction
- All three stages deduplicate against each other

</td>
<td width="25%" align="center" valign="top">
<br>
<strong>Pipeline Ready &raquo;</strong>
<br><br>
<code>--json</code> writes structured findings to stdout.
<br><br>

- Per-finding: offset, type, size, path, quality, sha256
- Pipe directly into Claude Code, jq, or any downstream processor
- Entire recovery session stays in one terminal

</td>
</tr>
</table>

---

## Use Cases

<img src="assets/accent.svg">

Runs on Linux and Windows. Works on disk images, raw block devices, and firmware flash dumps.

<table>
<tr>
<td width="33%" align="center" valign="top">
<br>
<strong>INCIDENT RESPONSE</strong>
<br><br>
Recover deleted evidence from seized drives without installing software on the target system. Run from USB with no footprint on the source.
<br><br>
</td>
<td width="33%" align="center" valign="top">
<br>
<strong>FORENSIC ANALYSIS</strong>
<br><br>
Reconstruct files from damaged or corrupted filesystems. No partition table or directory structure required. Raw bytes are enough.
<br><br>
</td>
<td width="33%" align="center" valign="top">
<br>
<strong>FIRMWARE EXTRACTION</strong>
<br><br>
Carve SquashFS, JFFS2, UBIFS, U-Boot, FIT, and cramfs images from raw flash dumps with header-derived exact boundaries.
<br><br>
</td>
</tr>
<tr>
<td width="33%" align="center" valign="top">
<br>
<strong>MEMORY FORENSICS</strong>
<br><br>
Recover LiME and HPAK acquisition images. The LiME size parser reads the segment chain to compute the exact dump extent.
<br><br>
</td>
<td width="33%" align="center" valign="top">
<br>
<strong>MALWARE INVESTIGATION</strong>
<br><br>
Extract PE, ELF, DEX, and APK files from disk images where artifacts were cleared from the filesystem. No metadata needed.
<br><br>
</td>
<td width="33%" align="center" valign="top">
<br>
<strong>CTF & RESEARCH</strong>
<br><br>
Custom signature corpus support (<code>.pala</code> format). Write a new file fingerprint in 10 lines of Python. Load it at runtime with <code>-c</code>.
<br><br>
</td>
</tr>
</table>

---

## Claude Code Integration

<img src="assets/accent.svg">

PALA was built with Claude Code. It is designed to run inside a Claude Code session.

The binary is 988KB. It runs from a USB stick. The full recovery workflow - disk enumeration, raw carving, result analysis, file identification - stays in a single terminal conversation without switching tools or losing context.

**`--json` output is designed for AI consumption.** Every finding includes offset, type, extension, size, path, quality, and SHA256. Pipe it directly into a Claude Code session and ask it to identify which files match what you are looking for.

```bash
# Run recovery and write findings
sudo pala /dev/sda /media/usb/recovered/ --json > findings.json

# Ask Claude Code: "Read findings.json. Which files are most likely my deleted presentation?"
```

Claude reads the structured output, ranks candidates by type and size, explains what each recovered file probably contains, and tells you exactly which files to open first - all inside the same session.

```json
{
  "source": "/dev/sda",
  "found": 47,
  "findings": [
    {
      "offset": 1073741824,
      "type": "jpeg",
      "extension": "jpg",
      "size": 3145728,
      "path": "/media/usb/recovered/jpg_0001.jpg",
      "quality": "Complete",
      "sha256": "a3f2..."
    }
  ],
  "session_summary": {
    "bad_sectors": 0,
    "merge_count": 3,
    "partial_count": 1,
    "files_per_gb": 2.74
  }
}
```

---

## Installation

Requires Rust stable. No other build dependencies.

```bash
git clone https://github.com/francis-rancid/PALA.git
cd PALA
cargo build --release
```

Binary lands at `target/release/pala`. Copy it to a USB stick or your PATH.

For filesystem-aware recovery (`--filesystem`), The Sleuth Kit must be available at runtime. If absent, PALA warns and falls back to signature carving only.

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

# Recover only specific types from a live device
sudo pala /dev/sdb recovered/ -t jpeg,pdf

# Filesystem-aware recovery
sudo pala /dev/sdb recovered/ --filesystem=auto

# Firmware/embedded recovery
pala firmware.bin recovered/ --triage-mode=firmware

# Sector-aligned scan for block devices
sudo pala /dev/sdb recovered/ --align=512

# Structured output for downstream tooling
pala disk.img out/ --json | jq '.findings[] | {ext, size, offset}'

# Skip encrypted sectors, unpack ZIP members
pala disk.img out/ --skip-high-entropy --container-depth

# Windows forensic artifacts only
pala disk.img out/ --triage-mode=windows
```

**Critical:** the output directory must be on a different drive than the source. Writing recovered files to the same drive risks overwriting data you are trying to recover. PALA warns if source and output share a device.

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
      --max-size <bytes>    Maximum size per recovered file
      --min-size <bytes>    Minimum size per recovered file
  -n, --count <n>           Stop after recovering N files
      --align <bytes>       Only match signatures at offsets that are multiples of N
      --triage-mode <mode>  Limit types to a preset group:
                            media | documents | executables | archives |
                            email | windows | databases | memory | filesystem | firmware
      --filesystem <fs>     Filesystem-aware inode recovery:
                            auto | ext2 | ntfs | apfs | fat32
      --skip-high-entropy   Skip sectors with Shannon entropy > 7.5
      --container-depth     Extract member files from carved ZIP/DOCX/XLSX/PPTX containers
      --no-fat32-stage2     Disable FAT32 deleted-entry recovery stage
  -h, --help                Show this help
```

---

## Supported Types

<details>
<summary>70+ supported file types across 8 categories</summary>

| Category | Types |
|----------|-------|
| **Images** | jpeg, png, gif87a, gif89a, bmp, tiff (LE/BE), psd, mng, jng |
| **Video** | mp4/mov, mkv/webm, riff (avi/wav/webp) |
| **Audio** | mp3 (ID3), flac, aac (ADTS) |
| **Documents** | pdf, rtf, ole2 (doc/xls/ppt), zip (docx/xlsx/pptx/jar/apk), eml |
| **Archives** | gz, 7z, rar |
| **Databases** | sqlite, sqlite-wal |
| **Executables** | elf, pe/mz, dex, art |
| **Forensic - Windows** | evtx, regf, lnk, prefetch, thumbcache, hibernate, bsod dumps |
| **Forensic - Filesystem** | ntfs-mft, fat32-fsinfo, ext2/3/4 superblock, ufs1/ufs2 superblock, apfs |
| **Forensic - Memory** | lime, hpak |
| **Forensic - Crypto** | luks, bitlocker |
| **Forensic - Network** | pcap (LE/BE), pcapng |
| **Forensic - Virtual** | vmdk, vhdx |
| **Firmware** | squashfs (LE/BE/v3), jffs2 (LE/BE), ubifs, u-boot, fit/dtb, cramfs (LE/BE) |
| **Certificates** | openssh private key, pem certificate chain, der certificate |
| **Other** | iso9660, bplist, systemd journal |

</details>

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
    "bad_sectors": 0,
    "merge_count": 3,
    "partial_count": 1,
    "files_per_gb": 2.74
  }
}
```

| Field | Values |
|-------|--------|
| `source` | `null` (sig carve), `"mft:<hex>"` (NTFS stage-2), `"fat32:<hex>"` (FAT32 stage-2), `"inode:<path>"` (TSK), `"zip:<hex>:<member>"` (container) |
| `quality` | `Complete` (end marker found), `Partial` (truncated at max-size), `Fragmented` (non-contiguous clusters) |

---

## Custom Signatures

Load additional signatures at runtime with `-c`:

```bash
pala disk.img out/ -c custom.pala
```

Write a corpus file in Python:

```python
import struct

MAGIC = b"PALA"

def make_corpus(sigs):
    buf = MAGIC + b"\x01" + struct.pack("<I", len(sigs))
    for s in sigs:
        name  = s["name"].encode()
        ext   = s["ext"].encode()
        magic = s["magic"]
        desc  = s["desc"].encode()
        buf += bytes([len(name)]) + name
        buf += bytes([len(ext)])  + ext
        buf += bytes([len(magic)]) + magic
        buf += b"\x00"  # flags
        buf += b"\x00"  # magic_offset
        buf += struct.pack("<Q", s["max_size"])
        buf += struct.pack("<I", s["min_size"])
        buf += bytes([len(desc)]) + desc
    return buf

open("custom.pala", "wb").write(make_corpus([{
    "name": "mysave", "ext": "sav", "magic": b"MYSAVE\x01",
    "max_size": 10 * 1024 * 1024, "min_size": 64, "desc": "My Game Save"
}]))
```

The full corpus format is documented in `src/corpus.rs`.

---

## Known Issues

- FAT32 stage-2 cluster prediction assumes unfragmented files; heavily fragmented volumes produce incomplete recoveries
- JFFS2 and UBIFS do not carry a total-size field; PALA uses a conservative static cap for these formats
- Files whose sectors have been overwritten by new data cannot be recovered
- Encrypted volumes: PALA detects LUKS and BitLocker headers but cannot decrypt content
- Hibernate recovery: if the drive resumed from hibernation, the `hiberfil.sys` header is zeroed; no magic bytes means no carve

---

## Getting Help

Open an issue at [github.com/francis-rancid/PALA/issues](https://github.com/francis-rancid/PALA/issues). Include the error output and `pala --version`.

---

## Contributing

Contributions welcome. The most useful additions are new file signatures. Extend the `SIGS` array in `src/main.rs` with a magic byte sequence, an optional end marker or size field strategy, size bounds, and a short description.

---

## Credits

See [CONTRIBUTORS.md](CONTRIBUTORS.md).

- **Nicholas Kloster** ([@francis-rancid](https://github.com/francis-rancid)) - author
- **Claude Code** ([claude.ai/code](https://claude.ai/code)) - filesystem recovery stages, MFT run list parsing, FAT32 deleted-entry recovery, entropy classification, ZIP container depth, firmware signatures, FLAC/LiME/SquashFS/U-Boot/FIT/cramfs size parsers, sector-aligned scan mode, and test suite

---

MIT OR Apache-2.0
