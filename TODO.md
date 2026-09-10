# PALA — Roadmap

Items in order of priority. One at a time, quality first.

---

## #1 — Streaming architecture

**Problem:** The block device path (`read_resilient`) allocates `vec![0u8; size]` — a 4TB drive
requires 4TB of RAM. The mmap path for regular files already pages on demand (OS handles this),
so files are fine. Block devices are the gap.

**Fix:** Process block devices in overlapping 256MB chunks using `pread()` with bad-sector
zero-fill. Keep `carve()` on `&[u8]` (unchanged). Add `CarveState` to carry the seen-offset
set and file counter across chunks so findings are globally deduplicated and sequentially
numbered. Files (mmap path) unchanged.

**Overlap:** 64MB. Files starting within 64MB of a chunk boundary are captured in the next
chunk's overlap region. Files larger than 256MB (video, memory dumps) will be truncated at the
chunk window — acceptable; those go through ddrescue first anyway.

**Status:** [x] done — `carve_device()` + `CarveState`; 44/44 tests pass; loop device verified

---

## #2 — Parallel scanning with rayon

**Problem:** Signatures run serially. Each `memmem` search is independent.

**Fix:** Replace the `for (si, sig) in sigs.iter().enumerate()` loop in `carve()` with
`sigs.par_iter()` (rayon). Findings merged and sorted by offset after the parallel phase.
The `seen` set becomes a `DashSet` or we dedup post-merge.

**Status:** [x] done — `scan_sig()` + `par_iter()` + serial write phase; user > real confirmed; 44/44 tests pass

---

## #3 — More size algorithms

Types that currently fall back to max size (and get marked truncated or over-carve):

| Type | Algorithm |
|------|-----------|
| MP4/MOV | Walk ISOBMFF boxes: `mdat` size at offset 0 (u32 BE) or 8 (u64 BE for extended) |
| MKV/WebM | EBML variable-length integer at header gives element size |
| RAR | Scan for end-of-archive marker `\xC4\x3D\x7B\x00\x40\x07\x00` |
| 7z | `StartHeader.NextHeaderOffset + NextHeaderSize + 32` from bytes 12-27 |
| OLE2 | `(num_sectors + 1) × sector_size` from 512-byte header |
| MP3 | Walk frame headers: `144 × bitrate / sample_rate + padding_bit` per frame |

**Status:** [x] done — mp4_size (ISOBMFF box walk) + mkv_size (EBML vint) + rar_size (4.x block chain) + sevenz_size (StartHeader) + ole2_size (FAT sector estimate) + mp3_id3_size (Xing/Info VBR header); 6 new Special variants; 50/50 tests pass

---

## #4 — Metadata extraction to JSON

**Problem:** `--json` output has offset, size, SHA256, truncated. Triage needs more.

**Fix:** Add optional `"meta"` object per finding. Parse on the carved bytes, emit only what
validates. Per type:

| Type | Fields |
|------|--------|
| JPEG | EXIF: GPS lat/lon, timestamp, Make, Model, Orientation |
| PNG | tEXt/iTXt: Creation Time, Software, Author |
| ELF | e_machine, e_type, entry point, interpreter path, stripped flag |
| PE | TimeDateStamp (compilation), OriginalFilename, PDB path, imports count |
| NTFS MFT | $FILE_NAME attribute: filename, FILETIME created/modified/accessed |
| SQLite | application_id (u32 at offset 60), user_version (u32 at 68), page count |

New flag: `--meta` to opt in (parsing is extra work per file).

**Status:** [x] done — JPEG/EXIF, PNG tEXt, ELF, PE, NTFS MFT, SQLite; `--meta` flag; JSON output extended; 53/53 tests pass

---

## #5 — More file types

High forensic value types not yet covered:

| Type | Magic | Notes |
|------|-------|-------|
| PCAP | `\xd4\xc3\xb2\xa1` (LE) / `\xa1\xb2\xc3\xd4` (BE) | Network captures |
| PCAPng | `\x0a\x0d\x0d\x0a` | Modern capture format, section block |
| VMDK | `KDMV` (sparse) / `COWD` (COW) | VMware disk images |
| VHDX | `vhdxfile` (8 bytes) | Hyper-V disk images |
| APFS | `\x4e\x58\x53\x42` ("NXSB") at offset 32 of container superblock | macOS 2017+ |
| OpenSSH private key | `-----BEGIN OPENSSH PRIVATE KEY-----` | High-value artifact |
| PEM cert | `-----BEGIN CERTIFICATE-----` | X.509 in PEM encoding |
| X.509 DER | `\x30\x82` (SEQUENCE, length > 127) | DER-encoded cert |
| Android ART | `art\n` | Android 5+ compiled app image |

**Status:** [x] done — PCAP LE/BE (pcap_size block-walk), PCAPng (pcapng_size SHB+block-walk), X.509 DER (der_size SEQUENCE header), APFS (apfs_size block_size×block_count), OpenSSH key, PEM cert, Android ART, VMDK, VHDX; ISO 9660 (iso9660_size via PVD VolumeSpaceSize×LogicalBlockSize, moff=32768); systemd journal (lpkshhrh magic); bad_sectors in JSON; 88/88 tests pass

---

## #6 — Fragment reassembly

**Problem:** PALA assumes files are contiguous. Fragmented files are silently missed.

**Three-stage approach (least to most complex):**

1. **Sequential pair scoring** — when two adjacent carved blocks of the same type both pass
   their validator, and the second block's content is plausible as a continuation of the first
   (e.g. JPEG entropy continuity, ZIP central directory follows local file headers), merge them.
   Catches the most common case: 2-fragment files.

2. **Block hash database** — hash every 512-byte block against a reference corpus (NSRL-style).
   Skip known-OS blocks. Chain remaining blocks greedily. Reduces the search space dramatically
   for drives that ran common OS installs.

3. **Statistical block classifier** — train a per-type classifier on 512-byte block content
   distributions. Chain blocks by classifier confidence score. This is what `pala-rank` could
   eventually do at the block level rather than the file level.

Start with (1). (2) and (3) are research items.

**Status:** [x] done — stage 1: frag_merge() post-dedup pass; only fires on trunc=true + same-type + exact-contiguous adjacency; frag_valid() per Special; outer fixed-point loop for 3+ fragment runs; --max-size flag for window capping (enables test + memory-constrained recovery); 63/63 tests pass

---

## #7 — Resume / checkpoint

**Problem:** Multi-hour scans on large drives get interrupted. No way to resume.

**Fix:** After each signature pass completes, write `.pala-state` (JSON) next to the output
directory:

```json
{
  "source": "/dev/sda",
  "source_size": 4000000000000,
  "completed_sigs": ["jpeg", "png", "pdf"],
  "findings_so_far": 142
}
```

On re-run with the same source + outdir, detect the state file, skip completed signatures,
resume from the next one. Finding counter restores from the state file.

**Status:** [x] done — .pala-state.json in outdir; per-chunk checkpoint on block device path; source+size validation on --resume; ctr and completed_chunks restore; file-mode writes state at end; 73/73 tests pass

---

## #8 — Rate limiting for degraded drives

**Problem:** When `read_resilient` encounters bad sectors (EIO), hammering a mechanically
failing drive at full speed accelerates head degradation and risks total failure.

**Fix:** When the bad-sector counter exceeds a threshold (e.g. 3 sectors), insert a configurable
inter-read delay (`--degraded [ms]`, default 1ms). Log a warning at first bad sector. The delay
is applied between `read_at()` calls in `read_resilient()`, not between chunks, so it only
activates on the block device path.

**Status:** [x] done — `--degraded [ms]` (optional value, default 1ms); delay activates after 3 bad sectors; zero-fill on EIO; first-bad-sector warning with --degraded hint; applied per-sector in read_chunk_resilient(); 73/73 tests pass

---

## #9 — Recovery quality classification

**Problem:** `truncated: bool` is coarse. Triage needs to know whether a carved file is
complete, truncated at the max-size window, or was assembled from fragments.

**Fix:** Add a `quality` field to JSON output and to the verbose completion line.
Three values: `complete` (size algo found natural end), `partial` (hit max-size window without
natural end), `fragmented` (frag_merge assembled two or more pieces). Track merge count in
`CarveState`. No new scanner logic — these states are already known at write time.

**Status:** [x] done — Quality enum (complete/partial/fragmented); quality field in JSON + verbose line; merge_count in CarveState; _partial suffix on filename; session_summary in JSON; 76/76 tests pass

---

## #10 — Session summary line

**Problem:** After a long scan the user has no quick way to assess drive health beyond
the final file count. Actionable triage needs more.

**Fix:** At completion, always print (to stderr) a one-line session summary:
`pala: {N} file(s) | {rate:.1} files/GB | {bad} bad sector(s) | {merges} fragment merge(s)`
Add `session_summary` object to `--json` output with the same fields.
All counters are already tracked (`total_bad`, `state.findings_so_far`,
`state.merge_count` once #9 adds it).

**Status:** [x] done — session_summary object in JSON (bad_sectors, merge_count, files_per_gb); human-readable summary line to stderr when on block device or merges > 0; 76/76 tests pass

---

## #11 — Triage presets (`--triage-mode`)

**Problem:** Users frequently want to recover one category (documents after ransomware,
media from a phone wipe, forensic artifacts from a compromised host) but must manually
supply `-t jpeg,png,pdf,...`.

**Fix:** Add `--triage-mode=documents|databases|media|forensic` that expands to predefined
sig groups:
- `documents` → pdf, rtf, docx, xlsx, pptx, odt, ole2
- `databases` → sqlite, sqlite_wal
- `media`     → jpeg, png, gif87a, gif89a, bmp, tiff_le, tiff_be, psd, riff, mkv, mp4, mp3_id3, flac, aac
- `forensic`  → evtx, regf, lnk, pf, thumbcache, hibr, pagedump, ntfs_mft, fat32_fsinfo, lime

No new scanner logic — pure alias expansion before the sigs filter.

**Status:** [x] done — `--triage-mode=documents|databases|media|forensic` alias expansion before sigs filter; no new scanner logic; 88/88 tests pass

---

## #12 — Gap-tolerant fragment reassembly

**Problem:** `frag_merge()` requires `b_start == a_end` — exact byte contiguity. Real fragmentation
rarely works this way: a freed cluster, a FAT directory entry, or a single bad sector may sit
between two fragments of the same file. Current code silently drops fragment pairs with any gap.

**Fix:** Add a `--frag-gap=N` flag (default 0 = current behavior). When N > 0, `frag_merge()` also
tries merges where `b_start` is in `(a_end, a_end + N]`. The gap bytes are zero-filled in the
merged output (already zeroed by bad-sector handling on block devices). The merged candidate is
written with the gap bytes as part of the file — the file may still be partially corrupt in the gap
region, but the reconstructed envelope is correct and parseable.

Trigger condition remains: `A.trunc == true && same name && b_start <= a_end + gap_limit`. The
`frag_valid()` pass still runs on the merged result, so invalid joins are rejected.

**Implementation:**
- `frag_merge(candidates, gap_limit: usize)` — new signature
- Gap fill: `vec![0u8; b_start - a_end]` inserted between A.data and B.data
- `--frag-gap=N` parsed in main(), passed through `carve()` -> `frag_merge()`
- New tests: ZIP/JPEG with a 512-byte gap between fragments

**Status:** [x] done — `frag_merge(gap_limit)` with zero-fill gap insertion; `--frag-gap=N` CLI flag; passed through `carve()` and `carve_device()`; test asserts count==1 + size==original+gap (not SHA256 — merged output includes zero-filled gap bytes); 91/91 tests pass

---

## #13 — Bad sector map in JSON output

**Problem:** The `--json` report includes `bad_sectors_zeroed: N` (a count) but no location
information. Forensic workflows need to know *where* the gaps are to cross-reference with ddrescue
maps, identify which recovered files have zero-filled regions, and decide which files are worth
further recovery attempts.

**Fix:** Track the byte offset of every bad sector during `read_chunk_resilient()`. Add to JSON
output:
```json
{
  "bad_sector_offsets": [4096, 8704, 1048576],
  "bad_sectors_zeroed": 3
}
```

Per-finding, add a `"has_bad_sectors": true` flag when any bad sector offset falls within the
carved byte range `[finding.offset, finding.offset + finding.size)`.

Cap the reported offset list at 1,000 entries (drives with mass failure emit millions); log a
truncation note. The `bad_sector_offsets` field is omitted entirely when count is 0.

**Status:** [x] done — `bad_sector_offsets` Vec in CarveState (capped at 1000); `has_bad_sectors: bool` on Finding; `bad_sector_offsets` in JSON (omitted when empty, truncation flag when ≥1000); read_chunk_resilient() appends offsets via &mut Vec<u64> param; 90/90 tests pass

---

## #14 — Entropy-based block skip for fragment reassembly

**Problem:** On drives with large free-space clusters, `frag_merge()` is called against many
candidate pairs that will never merge — free-space blocks are carved as truncated candidates and
sit adjacent to real fragments. This is a performance issue for high-candidate-count scans.

**Fix:** Before calling `frag_merge()`, compute a lightweight entropy estimate for each candidate's
first 512 bytes. Candidates with near-zero entropy (< 0.1 bits/byte — all-zeros, slack space,
repeating FAT patterns) are tagged `low_entropy = true`. `frag_merge()` skips the join attempt
when A or B is low-entropy.

Entropy formula: Shannon entropy on byte frequency histogram of the first 512 bytes, capped at
8 bits/byte.

Note: low-entropy candidates are still written to disk. The skip applies only to the merge
attempt, not to writing. A legitimately-carved all-zeros block (e.g., a sparse ELF section) is
unaffected because its validator would have already accepted it as non-truncated.

**Status:** [x] done — `low_entropy: bool` field on Candidate; `entropy_estimate()` helper (Shannon entropy, first 512 bytes, threshold 0.1 bits/byte); `frag_merge()` skips the join attempt when A or B is low_entropy; 91/91 tests pass

---

## #15 — Filesystem-assisted recovery (`--filesystem`)

**Problem:** PALA is entirely signature-driven — it knows nothing about the target filesystem's
own bookkeeping. For the common case (ext4/NTFS with recently deleted files), the filesystem
still holds the *exact* block map of each deleted inode in its allocation tables. `frag_merge()`
must guess at non-contiguous layouts by adjacency; the filesystem *already knows* the correct
block sequence. Files with no recognizable magic bytes (plaintext, sparse databases, proprietary
formats) are invisible to the signature scanner but fully described by their inode.

**Fix:** Add `--filesystem=<type>` (supported: `ext2`, `ext3`, `ext4`, `fat`, `ntfs`) that runs a
two-phase recovery:

**Phase A — inode map extraction (requires The Sleuth Kit):**
- Call `fls -rd <device>` to enumerate deleted inodes
- For each deleted inode, call `istat <device> <inum>` to get the block address list
- Build a `Vec<InodeManifest>` of `{inum, name_hint, blocks: Vec<(start_block, block_count)>}`

**Phase B — block-addressed extraction:**
- For each manifest entry, read blocks in declared order (via `pread()` at `block * block_size`)
- Run the same sig-validator on the assembled bytes (confirm file type, apply quality label)
- Write to outdir as `{type}_{N:04}.{ext}` with `"source": "inode:{inum}"` in JSON
- Mark `quality: Fragmented` when blocks were non-contiguous (inode-mapped fragmentation)

**Phase A+B is additive:** the existing linear sig-scan still runs afterwards as a fallback,
deduplicating by SHA256 so inode-recovered files are not written twice.

**TSK dependency:** `fls` and `istat` must be on PATH. If absent, emit a warning and fall
through to signature-only mode. No static linking — TSK is invoked via `Command::new("fls")`.

**New JSON field per finding:**
```json
{ "source": "inode:12345", "blocks": [[4096, 8], [32768, 3]] }
```

**Implementation sketch:**
```rust
fn extract_inode_manifests(dev: &str, fs_type: &str) -> Result<Vec<InodeManifest>> {
    // fls -rd <dev> → parse deleted entries → istat per inum → block list
}
fn carve_inode(dev: &File, m: &InodeManifest, block_size: u64) -> Result<Vec<u8>> {
    // pread each block range in order, concat, return assembled bytes
}
```

**Status:** [x] done — `fls_deleted()` + `icat_read()` + `detect_sig()` + `carve_inode_phase()`; `--filesystem=<type>` CLI flag; inode phase runs before sig scan and populates `state.seen_sha256`; SHA256 dedup check runs BEFORE counter increment and `fs::write` so no file is written twice and no sequence number is consumed; graceful fallback when TSK absent; `source: Option<String>` on Finding + `"source":"inode:N"` in JSON; 3 Rust unit tests (detect_sig ×3 + sha256_dedup); 3 Python tests (CLI flag + dedup + real ext2 image via mkfs.ext2+debugfs+fls+icat — no root, no loop device); 94/94 tests pass

---

## #16 — NTFS cluster-run stage-2

**Problem:** The `ntfs_mft` sig already carves MFT entries. The non-resident `$DATA` attribute
inside each entry holds a run list — `(start_lcn, length_clusters)` pairs — that pinpoints
exactly where the file's data blocks are on disk. Previously those run lists were ignored;
only the MFT entry metadata was extracted.

**Fix:** After the sig scan, iterate over all carved MFT entries. For each one, walk its
1024-byte records (the ntfs_mft sig carves contiguous blocks of records from the MFT stream).
Parse the non-resident `$DATA` attribute, decode the variable-length run list, and assemble the
file content by reading those clusters directly from the source image (mmap path only; block
device path is chunked and can't address arbitrary clusters). Deduplicates against
`state.seen_sha256` so files already found by the sig scan or inode phase are not written twice.
Type detection uses magic-byte matching first, then filename extension from `$FILE_NAME` as
fallback.

**NTFS BPB detection:** `ntfs_cluster_size()` reads `NTFS    ` OEM ID + `bytes_per_sector ×
sectors_per_cluster` from the volume's boot sector. Negative `sectors_per_cluster` encodes large
clusters as `2^(-spc)` bytes (Windows 8+ style). Defaults to 4096 if BPB absent or corrupt.

**Run list encoding:** Each run = 1-byte header (low nibble = length-field bytes, high nibble =
offset-field bytes) + unsigned length + signed delta-LCN. Delta is cumulative; each run's LCN
= sum of all previous deltas. Sparse runs (high nibble = 0) have no LCN and are skipped.

**`--meta` enrichment:** `mft_meta()` now calls `mft_run_info()` to add `cluster_runs`,
`data_size_bytes`, and `ntfs_filename` to the JSON metadata output when `--meta` is set.

**New JSON fields per stage-2 finding:**
```json
{ "source": "mft:00014000", "quality": "Fragmented", "extension": "dat" }
```

**Status:** [x] done — `ntfs_cluster_size()`, `parse_data_runs()`, `MftRunInfo`,
`mft_run_info()`, `carve_mft_stage2()`; runs as Phase C (after sig scan, file/mmap path only);
iterates all 1024-byte records within each carved MFT chunk; `mft_meta()` extended for
cluster_runs in `--meta` output; 7 new Rust unit tests (ntfs_cluster_size, parse_data_runs ×4,
mft_run_info ×2); 1 Python integration test (real 4MB NTFS image via mkntfs+ntfscp, blob SHA256
verified in stage-2 findings, no SHA256 duplicates); 95/95 tests pass; 14/14 Rust unit tests pass

---

## #17 — Entropy sector classification

**Problem:** High-entropy sectors (encrypted volumes, compressed archives, encrypted swap) produce
false-positive sig hits. The `\xFF\xD8` SOI of a JPEG carved from inside an AES-CBC block looks
real but the file is garbage. Scanning these sectors wastes I/O and pollutes output.

**Fix:** Shannon entropy per 512-byte sector: H = -Σ p_i × log2(p_i). Two gates:
- **Skip gate:** `--skip-high-entropy` — if the sector containing a candidate's start offset has
  H > 7.5, discard without writing. Applied after SHA256 dedup (entropy check runs per-candidate,
  SHA256 is still inserted to prevent re-processing from other phases).
- **Survey:** `entropy_survey()` scans the entire source once after Phase B and reports
  `zero_sectors` (H < 0.1), `high_entropy_sectors` (H > 7.5), and `total_sectors`. Emitted in
  `session_summary.entropy_survey` JSON. Runs unconditionally; skip gate is opt-in.

**Implementation:** `sector_entropy(data: &[u8]) -> f32` (single 512-byte slice → H);
`entropy_survey(src: &[u8]) -> (u64, u64, u64)` (full-pass counters). `CarveState` extended with
`skip_high_entropy: bool`, `zero_sectors: u64`, `high_entropy_sectors_skipped: u64`.
`--skip-high-entropy` CLI flag sets `state.skip_high_entropy = true`.

**Status:** [x] done — `sector_entropy()`, `entropy_survey()`; `--skip-high-entropy` flag; Phase C
entropy survey runs unconditionally after sig scan; skip gate in `carve()` write phase;
`session_summary.entropy_survey` JSON; 2 Python integration tests (CLI no-crash + JSON field);
101/101 tests pass

---

## #18 — ZIP container depth (DOCX/XLSX/PPTX member extraction)

**Problem:** ZIP-based office formats (DOCX, XLSX, PPTX) and plain ZIP archives often contain
recoverable files (embedded images, attachments, sub-documents) that sig scan misses because the
member content is stored compressed or at cluster-aligned offsets inside the container.

**Fix:** Phase F post-sig-scan: for every ZIP/DOCX/XLSX/PPTX/JAR/APK finding from all prior
phases (inode, MFT stage-2, FAT32 stage-2, sig scan), open with `zip::ZipArchive`, iterate
members, SHA256-dedup against `state.seen_sha256`, detect type by magic-first then filename
extension fallback, write to output. Depth = 1 (members only; no recursive extraction of
ZIPs-within-ZIPs to avoid combinatorial explosion). Controlled by `--container-depth` flag
(default off). 500MB per-member cap.

**Source field:** `"zip:{hex_offset}:{member_name}"` e.g. `"zip:00000400:photo.jpg"`.

**Crate:** `zip = { version = "2", default-features = false, features = ["deflate"] }`.

**Status:** [x] done — `unpack_zip_finding()`, `carve_zip_members()`; `--container-depth` flag;
Phase F runs after FAT32 stage-2 on combined findings from all prior phases; SHA256 dedup; type
detection; 1 Python integration test (outer image → carved ZIP → embedded JPEG found by
--container-depth, SHA256 verified); 101/101 tests pass

---

## #19 — FAT32 cluster-chain stage-2

**Problem:** FAT32 deleted-file recovery: directory entries survive deletion (`first_cluster` and
`size` retained) but FAT chain entries are cleared to 0. Sig scan cannot find files whose content
doesn't start with a recognized magic byte.

**Fix:** Phase E post-sig-scan: scan FAT32 FSINFO findings, parse BPB, walk deleted directory
entries (first byte 0xE5), assemble file content via cluster chain. When a FAT entry is cleared
(deleted state), fall back to contiguous cluster prediction (`cur + 1`) — valid for
unfragmented volumes which represent the majority of consumer SD cards and USB drives.
500MB per-file cap.

**`fat32_params()`:** validates `"FAT32   "` at BPB offset 0x52, `root_entry_count == 0`,
`fat_size_16 == 0`. `partition_start = FSINFO_offset - 512`. Returns `Fat32Params` struct:
`{partition_start, fat_offset, data_start, cluster_size, root_cluster, bps}`.

**`fat32_deleted_entries()`:** scans all directory clusters from `root_cluster`; skips 0x0F (LFN),
0x08 (volume label), 0x10 (directory); extracts `{name, first_cluster, size}` for 0xE5 entries.

**`fat32_read_file()`:** chains via `fat32_next_cluster()` with `unwrap_or(cur + 1)` fallback;
stops at `size` bytes; returns `(data, is_complete)`.

**Source field:** `"fat32:{hex_partition_start}"`.

**Status:** [x] done — `Fat32Params`, `fat32_params()`, `fat32_cluster_offset()`,
`fat32_next_cluster()`, `fat32_read_file()`, `fat32_deleted_entries()`, `carve_fat32_stage2()`;
`--no-fat32-stage2` flag to disable (on by default); Phase E in main carve flow; 2 Python
integration tests (no-op on non-FAT32 image + real deleted file recovery via mkdosfs+mcopy+mdel,
blob SHA256 verified); 101/101 tests pass

---

## #20 — APFS support via TSK

**Problem:** APFS volumes (macOS 10.13+, iOS 10.3+) use a completely different on-disk layout
from HFS+. Without filesystem-aware enumeration, only sig carving applies, missing intact inodes.

**Fix:** APFS is already supported by the existing `--filesystem` mechanism via `tsk_recover`
and `fls -f apfs`. Wire through `--filesystem=apfs` in CLI parsing. TSK on this system supports
APFS (confirmed via `fls -f list`). No new code required — the inode phase runs `fls -f apfs`
and hands results to the same `tsk_recover` pipeline.

**Graceful fallback:** when TSK finds no inodes on an APFS image (e.g. wrong partition offset),
the inode phase returns an empty list and sig scan proceeds normally. No crash.

**Status:** [x] done — no new code; `--filesystem=apfs` parsed by existing `-f` handler;
1 Python integration test (flag accepted, graceful empty-inode fallback); 101/101 tests pass
