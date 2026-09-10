# PALA — Claude Code Recovery Guide

You accidentally deleted a file. This guide walks you through recovering it with PALA and Claude Code.

---

## Step 1 — Stop writing to the drive immediately

Every byte written to the drive can overwrite the deleted file. Do not:

- Save files to the drive where the deletion happened
- Install software on that drive
- Run the browser or any app that logs to that drive

If you deleted from your main system drive (`/`), work fast.

---

## Step 2 — Find your source drive

Run this to list drives and their mount points:

```sh
lsblk -o NAME,SIZE,TYPE,MOUNTPOINT
```

Example output:

```
NAME   SIZE TYPE MOUNTPOINT
sda    500G disk
├─sda1   1G part /boot
└─sda2 499G part /
sdb    128G disk
└─sdb1 128G part /media/usb
```

Find the drive or partition where the deletion happened. You will use it as the PALA source (e.g. `/dev/sda2` or a disk image file).

Also check where you want to save recovered files:

```sh
df -h
```

---

## Step 3 — Choose a safe output location

**Critical:** The output directory must be on a **different drive** than the source. Writing recovered files to the same drive risks overwriting the data you are trying to recover.

Good: source = `/dev/sda`, output = `/media/usb/recovered/`
Bad: source = `/dev/sda`, output = `/home/user/recovered/`

PALA warns you if source and output are on the same device:

```
WARNING: output directory is on the SAME device as the source.
         Writing recovered files here risks overwriting the data you are
         trying to recover.  Use a different drive for --output.
```

If you see this warning, stop and use a USB stick or external drive.

---

## Step 4 — Build PALA (first time only)

```sh
cargo build --release
```

The binary lands at `target/release/pala`. It is ~501KB. You can copy it to a USB stick.

No internet connection required after the initial build. No installer. No runtime dependencies.

---

## Step 5 — Run recovery

### Recover everything:

```sh
sudo ./target/release/pala /dev/sda2 /media/usb/recovered/
```

`sudo` is needed to read a raw block device. Not needed for disk image files.

### Recover only specific file types:

```sh
sudo ./target/release/pala /dev/sda2 /media/usb/recovered/ -t jpeg,png,pdf
```

### List all supported types:

```sh
./target/release/pala --list
```

### Get structured output (pipe to jq or save for Claude):

```sh
sudo ./target/release/pala /dev/sda2 /media/usb/recovered/ --json > findings.json
```

---

## Step 6 — Check results

Recovered files are named `{type}_{N:04}.{ext}` — for example `jpg_0001.jpg`, `pdf_0003.pdf`.

A summary prints at the end:

```
pala: recovered 14 file(s) → /media/usb/recovered/
pala: 7 jpg, 4 png, 3 pdf
```

---

## Working with Claude Code

Claude Code can guide the full recovery session interactively. Here is what a typical session looks like.

---

**You:** I accidentally deleted a folder of photos from my laptop. I have a USB drive plugged in. Where do I start?

**Claude Code:** Run this to see your drives:

```sh
lsblk -o NAME,SIZE,TYPE,MOUNTPOINT
```

Paste the output here and I'll tell you which device to scan and how to safely route the output to your USB stick.

---

**You:** [pastes lsblk output]

**Claude Code:** Your main drive is `/dev/sda`. Your USB stick is at `/dev/sdb`, mounted at `/media/usb`. Run:

```sh
sudo ./target/release/pala /dev/sda /media/usb/recovered/ -t jpeg,png --json > /media/usb/findings.json
```

The `--json` flag writes a structured report alongside the recovered files. I can read that report and tell you which files are most likely to be the ones you deleted.

---

**You:** [runs pala, then:] Done. Here's the findings file.

**Claude Code:** [reads findings.json] I can see 47 JPEGs and 12 PNGs. The 6 largest JPEGs are between 2MB and 8MB — those are likely your camera photos. The smaller ones are probably thumbnails or embedded images from documents. Here are the ones to check first: `jpg_0003.jpg`, `jpg_0007.jpg`, `jpg_0012.jpg`.

---

The JSON output from `--json` is structured for easy reading:

```json
{
  "source": "/dev/sda",
  "source_bytes": 500107862016,
  "elapsed_ms": 41200,
  "found": 59,
  "findings": [
    {
      "offset": 1073741824,
      "type": "jpeg",
      "extension": "jpg",
      "description": "JPEG Image",
      "size": 3145728,
      "path": "/media/usb/recovered/jpg_0001.jpg",
      "truncated": false,
      "sha256": "a3f2..."
    }
  ]
}
```

### Other things to ask Claude Code

- "I ran `pala --list` and see 49 types — which ones are most likely to contain my deleted presentation?"
- "What does `truncated: true` mean in the JSON?"
- "I found 0 files — what should I try next?"
- "I have a proprietary file format I need to recover. How do I write a custom signature?"

---

## Custom signatures

If you need to recover a file type not built into PALA, you can write a custom signature corpus and load it with `-c`:

```sh
pala disk.img out/ -c my_game_saves.pala
```

A corpus file is a small binary file containing one or more signature records. The format is documented in `src/corpus.rs`. To create one:

```python
# example: add a signature for a proprietary save file
import struct

PALA_MAGIC = b"PALA"
VERSION = 0x01

def make_corpus(sigs):
    buf = PALA_MAGIC + bytes([VERSION]) + struct.pack("<I", len(sigs))
    for s in sigs:
        name  = s["name"].encode()
        ext   = s["ext"].encode()
        magic = s["magic"]
        desc  = s["desc"].encode()
        flags = 0  # no end_magic, no special handling
        buf += bytes([len(name)]) + name
        buf += bytes([len(ext)])  + ext
        buf += bytes([len(magic)]) + magic
        buf += bytes([flags])
        buf += bytes([0])              # magic_offset
        buf += struct.pack("<Q", s["max_size"])
        buf += struct.pack("<I", s["min_size"])
        buf += bytes([len(desc)]) + desc
    return buf

corpus = make_corpus([{
    "name": "mysave",
    "ext":  "sav",
    "magic": b"MYSAVE\x01",
    "max_size": 10 * 1024 * 1024,
    "min_size": 64,
    "desc": "My Game Save File",
}])

open("my_game_saves.pala", "wb").write(corpus)
```

Ask Claude Code: "I need to recover `.sav` files from a game. The file starts with `MYSAVE` followed by a version byte. How do I write a PALA corpus for this?"

---

## Supported file types

| Type | Extension | Description |
|------|-----------|-------------|
| jpeg | jpg | JPEG Image |
| png | png | PNG Image |
| gif87a / gif89a | gif | GIF Image |
| bmp | bmp | BMP Image |
| tiff_le / tiff_be | tif | TIFF Image |
| psd | psd | Photoshop Document |
| riff | wav / avi / webp | WAV Audio, AVI Video, WebP Image |
| mkv | mkv | MKV / WebM Video |
| mp4 | mp4 | MP4 / MOV Video |
| mp3_id3 | mp3 | MP3 Audio |
| flac | flac | FLAC Audio |
| aac | aac | AAC Audio |
| pdf | pdf | PDF Document |
| rtf | rtf | RTF Document |
| zip | zip / docx / xlsx / pptx | ZIP and Office Open XML |
| ole2 | doc | Legacy Office (DOC / XLS / PPT) |
| gz | gz | Gzip Archive |
| 7z | 7z | 7-Zip Archive |
| rar | rar | RAR Archive |
| sqlite | db | SQLite Database |
| eml | eml | Email (EML) |
| evtx | evtx | Windows Event Log |
| regf | dat | Windows Registry Hive |
| lnk | lnk | Windows Shell Link |
| pf | pf | Windows Prefetch |
| thumbcache | db | Windows Thumbcache |
| hibr / HIBR / wake / WAKE | bin | Windows Hibernate File |
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

## Notes on what PALA can and cannot recover

PALA uses **file carving** — it scans raw bytes for known file signatures (magic bytes). It does not use filesystem metadata.

**PALA can recover:**
- Files deleted with `rm`, moved to Trash then emptied, or lost after a format
- Files from damaged or corrupted filesystems
- Files from drives with missing partition tables

**PALA cannot recover:**
- Files whose disk sectors have been overwritten by new data
- Files from drives with full-disk encryption (unless you have the key)
- Files that were never written to disk (e.g. in-memory only)

**Recovery likelihood is highest when:**
- The deletion happened recently
- Little or no new data has been written to the drive since

---

## Troubleshooting

**`Permission denied` reading a block device:**

```sh
sudo ./target/release/pala /dev/sda2 /media/usb/recovered/
```

**No files recovered:**

- The sectors may have been overwritten. Run without `-t` to scan all types.
- Try the parent device instead of a partition: `/dev/sda` instead of `/dev/sda2`.

**Recovered files are corrupt:**

- The file may have been partially overwritten. The recovered portion is still present.
- JPEG and PNG use end markers so they are usually complete. TIFF and ZIP use internal size fields. Formats without size fields (MKV, MP3) may be truncated at the max size window.

**Too many recovered files:**

- Use `-t` to restrict to the type you care about.
- Sort by size: the file you want likely has a specific size range.

**PALA finds 0 hibr/hibernate files:**

The drive may have resumed from hibernation, which zeros the `hiberfil.sys` header. No magic bytes → PALA cannot find it by signature. Manual recovery: extract `hiberfil.sys` by inode using the filesystem's undelete tools, then use Volatility `imagecopy` to reconstruct the image.

**PALA crashes or exits non-zero:**

- Run with `-q` removed to see per-file output.
- Open an issue at https://github.com/sshpie/PALA with the error output and the pala version.
