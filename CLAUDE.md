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

The binary lands at `target/release/pala`. It is ~423KB. You can copy it to a USB stick.

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

Claude Code can help at each stage. Paste errors or questions directly into the conversation.

### Identify your drive:

> "I ran `lsblk` and got this output — which device is my main drive?"

### Figure out what to recover:

> "I deleted a folder of photos and a PDF report. What `-t` flags should I use?"

### Read the JSON findings:

> "Here is `findings.json` from pala — which files are most likely to be the ones I deleted?"

```sh
cat findings.json | head -40
```

### If you are unsure what file type you need:

```sh
./target/release/pala --list
```

Paste the list into Claude Code and describe what you deleted.

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

**PALA crashes or exits non-zero:**

- Run with `-q` removed to see per-file output.
- Open an issue at https://github.com/sshpie/PALA with the error output and the pala version.
