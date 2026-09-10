#!/usr/bin/env python3
"""
PALA Rust binary test suite.

No sudo, no loopback devices. Each test:
  1. Generate file(s) in memory
  2. Build a raw image: random_noise + file_bytes + random_noise
  3. Run ./target/release/pala on the image
  4. Verify recovered file(s) match originals by SHA256

Run from repo root:
  python3 tests/test_pala.py
"""

import hashlib
import io
import math
import os
import random
import shutil
import sqlite3
import struct
import subprocess
import sys
import tempfile
import zipfile
from pathlib import Path

# ── binary location ───────────────────────────────────────────────────────────

REPO = Path(__file__).parent.parent
PALA = REPO / "target" / "release" / "pala"

RED    = "\033[0;31m"
GREEN  = "\033[0;32m"
CYAN   = "\033[0;36m"
YELLOW = "\033[0;33m"
NC     = "\033[0m"

passed: list[str] = []
failed: list[tuple[str, str]] = []
skipped: list[str] = []

# ── helpers ───────────────────────────────────────────────────────────────────

def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()

def sha256f(path: Path) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(65536), b""):
            h.update(chunk)
    return h.hexdigest()

_rng = random.Random(0xC0FFEE)

def noise(n: int) -> bytes:
    return bytes(_rng.getrandbits(8) for _ in range(n))

def make_raw_image(*files: bytes, prefix: int = 4096, gap: int = 512) -> bytes:
    """Concatenate files into a raw image with random noise prefix/inter-gaps, zero suffix.

    The zero suffix matches real disk conditions: freed clusters after a deleted file
    are typically zeroed by the OS, not filled with random data.  Using random noise
    here would cause single-byte end-markers (e.g. RTF's '}') to be found in the
    trailing garbage instead of the file, producing a SHA256 mismatch.
    """
    out = noise(prefix)
    for i, f in enumerate(files):
        if i > 0:
            out += noise(gap)
        out += f
    out += b"\x00" * gap
    return out

def run_pala(img: bytes, outdir: Path, types: list[str] | None = None) -> dict[str, bytes]:
    """Write img to a temp file, run pala, return {filename: bytes} of carved files."""
    with tempfile.NamedTemporaryFile(suffix=".img", delete=False) as tf:
        tf.write(img)
        img_path = tf.name
    try:
        cmd = [str(PALA), img_path, str(outdir), "-q"]
        if types:
            cmd += ["-t", ",".join(types)]
        r = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
        if r.returncode not in (0,):
            raise RuntimeError(f"pala exit {r.returncode}: {r.stderr.strip()}")
        result = {}
        for p in outdir.iterdir():
            result[p.name] = p.read_bytes()
        return result
    finally:
        os.unlink(img_path)


def run_test(
    name: str,
    files: dict[str, bytes],
    skip: str | None = None,
    types: list[str] | None = None,
):
    """
    files: {label: bytes} — expected files to recover (by SHA256 match)
    types: optional list of type names to restrict the scan to
    """
    if skip:
        print(f"  {YELLOW}SKIP{NC}  {name} — {skip}")
        skipped.append(name)
        return

    print(f"  {CYAN}....{NC}  {name}", end="", flush=True)
    with tempfile.TemporaryDirectory() as td:
        outdir = Path(td)
        try:
            img = make_raw_image(*files.values())
            carved = run_pala(img, outdir, types)

            orig_hashes = {sha256(data): label for label, data in files.items()}
            carved_hashes = {sha256(data): fname for fname, data in carved.items()}

            missing = [label for h, label in orig_hashes.items() if h not in carved_hashes]
            if missing:
                print(f"\r  {RED}FAIL{NC}  {name} — not recovered: {missing}")
                failed.append((name, str(missing)))
            else:
                print(f"\r  {GREEN}PASS{NC}  {name}")
                passed.append(name)
        except Exception as e:
            print(f"\r  {RED}ERR {NC}  {name} — {e}")
            failed.append((name, str(e)))


def run_fp_test(name: str, seed: int = 0xCAFEBABE):
    """Scan random data — strong-magic types must produce 0 false positives."""
    STRONG = {"jpg", "png", "gif", "pdf", "zip", "docx", "xlsx", "pptx",
               "db", "7z", "rar", "psd", "tif", "webp", "rtf", "wav", "avi"}
    print(f"  {CYAN}....{NC}  {name}", end="", flush=True)
    with tempfile.TemporaryDirectory() as td:
        outdir = Path(td)
        try:
            rng = random.Random(seed)
            raw = bytes(rng.getrandbits(8) for _ in range(20 * 1024 * 1024))
            carved = run_pala(raw, outdir)
            strong_fps = [f for f, d in carved.items() if Path(f).suffix.lstrip(".") in STRONG]
            weak_fps   = [f for f, d in carved.items() if Path(f).suffix.lstrip(".") not in STRONG]
            if strong_fps:
                print(f"\r  {RED}FAIL{NC}  {name} — strong-magic FPs: {[Path(f).name for f in strong_fps[:5]]}")
                failed.append((name, f"{len(strong_fps)} strong-magic FPs"))
            else:
                note = f" ({len(weak_fps)} short-magic FPs)" if weak_fps else ""
                print(f"\r  {GREEN}PASS{NC}  {name}{note}")
                passed.append(name)
        except Exception as e:
            print(f"\r  {RED}ERR {NC}  {name} — {e}")
            failed.append((name, str(e)))


# ── file generators ───────────────────────────────────────────────────────────

try:
    from PIL import Image, ImageDraw
    HAS_PIL = True
except ImportError:
    HAS_PIL = False


def _pil_jpeg(w=400, h=300, quality=85, mode="RGB", progressive=False) -> bytes:
    buf = io.BytesIO()
    img = Image.new(mode, (w, h))
    draw = ImageDraw.Draw(img)
    for y in range(h):
        for x in range(w):
            draw.point((x, y), fill=(x % 256, y % 256, (x + y) % 256) if mode == "RGB" else (x + y) % 256)
    kwargs = {"quality": quality}
    if progressive:
        kwargs["progressive"] = True
    img.save(buf, "JPEG", **kwargs)
    return buf.getvalue()


def _pil_png(w=300, h=300, mode="RGB") -> bytes:
    buf = io.BytesIO()
    img = Image.new(mode, (w, h), color=(50, 100, 200) if mode == "RGB" else 128)
    ImageDraw.Draw(img).ellipse([20, 20, w-20, h-20], fill=(255, 80, 0) if mode != "L" else 200)
    img.save(buf, "PNG")
    return buf.getvalue()


def _pil_bmp(w=320, h=240) -> bytes:
    buf = io.BytesIO()
    img = Image.new("RGB", (w, h))
    draw = ImageDraw.Draw(img)
    for y in range(h):
        for x in range(w):
            draw.point((x, y), fill=(x % 256, y % 128, (x * y) % 256))
    img.save(buf, "BMP")
    return buf.getvalue()


def _pil_tiff(w=512, h=512) -> bytes:
    buf = io.BytesIO()
    img = Image.new("RGB", (w, h))
    draw = ImageDraw.Draw(img)
    for y in range(0, h, 32):
        for x in range(0, w, 32):
            draw.rectangle([x, y, x+31, y+31], fill=((x+y)%256, x%256, y%256))
    img.save(buf, "TIFF")
    return buf.getvalue()


def _pil_webp(w=400, h=300) -> bytes:
    buf = io.BytesIO()
    img = Image.new("RGB", (w, h), (40, 80, 160))
    ImageDraw.Draw(img).ellipse([50, 50, w-50, h-50], fill=(200, 60, 60))
    img.save(buf, "WEBP", quality=80)
    return buf.getvalue()


def _pil_gif89a() -> bytes:
    buf = io.BytesIO()
    frames = [Image.new("RGB", (100, 100), c).convert("P") for c in [(255,0,0),(0,255,0),(0,0,255)]]
    frames[0].save(buf, "GIF", save_all=True, append_images=frames[1:], loop=0, duration=100)
    return buf.getvalue()


def make_gif87a() -> bytes:
    header = b"GIF87a"
    lsd    = struct.pack("<HHBBB", 16, 16, 0x80, 0, 0)
    gct    = b"\xFF\x00\x00" + b"\x00\xFF\x00"
    img_d  = b"," + struct.pack("<HHHHB", 0, 0, 16, 16, 0)
    lzw    = b"\x02\x0C\x8C\x2D\x99\x87\x2A\x1C\xDC\x33\xA0\x02\x75\xEC\x00"
    return header + lsd + gct + img_d + lzw + b";"


def make_wav() -> bytes:
    sr, n = 44100, 4410
    pcm = b"".join(struct.pack("<h", int(32767 * math.sin(2*math.pi*440*i/sr))) for i in range(n))
    return (b"RIFF" + struct.pack("<I", 36 + len(pcm)) + b"WAVE"
            + b"fmt " + struct.pack("<IHHIIHH", 16, 1, 1, sr, sr*2, 2, 16)
            + b"data" + struct.pack("<I", len(pcm)) + pcm)


def make_pdf(pages: int = 1) -> bytes:
    return (
        b"%PDF-1.4\n"
        b"1 0 obj<</Type/Catalog/Pages 2 0 R>>endobj\n"
        b"2 0 obj<</Type/Pages/Kids[3 0 R]/Count 1>>endobj\n"
        b"3 0 obj<</Type/Page/Parent 2 0 R/MediaBox[0 0 612 792]>>endobj\n"
        b"xref\n0 4\n0000000000 65535 f\r\n"
        b"0000000009 00000 n\r\n0000000058 00000 n\r\n0000000115 00000 n\r\n"
        b"trailer<</Size 4/Root 1 0 R>>\nstartxref\n190\n%%EOF\n"
    )


def make_pdf_linearized() -> bytes:
    """Two %%EOF markers — tests end_magic_last=true (rfind)."""
    p1 = (b"%PDF-1.4\n1 0 obj<</Type/Catalog/Pages 2 0 R>>endobj\n"
          b"xref\n0 2\n0000000000 65535 f\r\n0000000009 00000 n\r\n"
          b"trailer<</Size 2/Root 1 0 R>>\nstartxref\n58\n%%EOF\n")
    p2 = (b"2 0 obj<</Type/Pages/Kids[3 0 R]/Count 1>>endobj\n"
          b"3 0 obj<</Type/Page/Parent 2 0 R/MediaBox[0 0 612 792]>>endobj\n"
          b"xref\n2 2\n0000000115 00000 n\r\n0000000174 00000 n\r\n"
          b"trailer<</Size 4/Root 1 0 R/Prev 58>>\nstartxref\n240\n%%EOF\n")
    return p1 + p2


def make_rtf() -> bytes:
    return (b"{\\rtf1\\ansi\\deff0\n"
            b"{\\fonttbl{\\f0 Times New Roman;}}\n"
            b"\\f0\\fs24 PALA recovery test.\\par\n"
            b"Second paragraph with \\b bold\\b0 text.\\par\n"
            b"}")


def make_docx(text: str = "PALA test") -> bytes:
    buf = io.BytesIO()
    with zipfile.ZipFile(buf, "w", zipfile.ZIP_DEFLATED) as z:
        z.writestr("[Content_Types].xml",
            '<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">'
            '<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>'
            '<Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>'
            '</Types>')
        z.writestr("_rels/.rels",
            '<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">'
            '<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>'
            '</Relationships>')
        z.writestr("word/_rels/document.xml.rels",
            '<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"/>')
        z.writestr("word/document.xml",
            f'<?xml version="1.0"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">'
            f'<w:body><w:p><w:r><w:t>{text}</w:t></w:r></w:p></w:body></w:document>')
    return buf.getvalue()


def make_xlsx() -> bytes:
    buf = io.BytesIO()
    with zipfile.ZipFile(buf, "w", zipfile.ZIP_DEFLATED) as z:
        z.writestr("[Content_Types].xml",
            '<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">'
            '<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>'
            '<Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>'
            '</Types>')
        z.writestr("_rels/.rels",
            '<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">'
            '<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>'
            '</Relationships>')
        z.writestr("xl/_rels/workbook.xml.rels",
            '<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">'
            '<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/>'
            '</Relationships>')
        z.writestr("xl/workbook.xml",
            '<?xml version="1.0"?><workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">'
            '<sheets><sheet name="Sheet1" sheetId="1" r:id="rId1" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"/></sheets>'
            '</workbook>')
        z.writestr("xl/worksheets/sheet1.xml",
            '<?xml version="1.0"?><worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">'
            '<sheetData><row r="1"><c r="A1" t="inlineStr"><is><t>Value</t></is></c></row></sheetData>'
            '</worksheet>')
    return buf.getvalue()


def make_sqlite() -> bytes:
    with tempfile.NamedTemporaryFile(suffix=".db", delete=False) as tf:
        db_path = tf.name
    try:
        conn = sqlite3.connect(db_path)
        conn.execute("CREATE TABLE files (id INTEGER PRIMARY KEY, name TEXT, size INTEGER)")
        for i in range(20):
            conn.execute("INSERT INTO files VALUES (?, ?, ?)", (i, f"file_{i}.txt", i * 1024))
        conn.commit()
        conn.close()
        return Path(db_path).read_bytes()
    finally:
        os.unlink(db_path)


# ── test cases ────────────────────────────────────────────────────────────────

def main():
    if not PALA.exists():
        print(f"{RED}pala binary not found at {PALA}{NC}")
        print("Run: cargo build --release")
        sys.exit(1)

    print(f"\npala Rust binary: {PALA}")
    print("=" * 60)

    pil_skip = None if HAS_PIL else "Pillow not installed"

    # ── JPEG ─────────────────────────────────────────────────────────────────
    print("\n  JPEG")
    run_test("JPEG standard (400×300 RGB)",
             {"std.jpg": _pil_jpeg()}, skip=pil_skip, types=["jpeg"])
    run_test("JPEG progressive",
             {"prog.jpg": _pil_jpeg(640, 480, progressive=True)}, skip=pil_skip, types=["jpeg"])
    run_test("JPEG grayscale",
             {"gray.jpg": _pil_jpeg(400, 300, mode="L")}, skip=pil_skip, types=["jpeg"])
    run_test("JPEG high quality (quality=98)",
             {"hq.jpg": _pil_jpeg(800, 600, quality=98)}, skip=pil_skip, types=["jpeg"])
    run_test("JPEG low quality (quality=10)",
             {"lq.jpg": _pil_jpeg(320, 240, quality=10)}, skip=pil_skip, types=["jpeg"])
    run_test("JPEG large (2000×1500)",
             {"large.jpg": _pil_jpeg(2000, 1500, quality=70)}, skip=pil_skip, types=["jpeg"])
    run_test("JPEG multiple adjacent",
             {"a.jpg": _pil_jpeg(200, 200), "b.jpg": _pil_jpeg(300, 200, quality=60)},
             skip=pil_skip, types=["jpeg"])

    # ── PNG ──────────────────────────────────────────────────────────────────
    print("\n  PNG")
    run_test("PNG RGB",
             {"rgb.png": _pil_png(300, 300)}, skip=pil_skip, types=["png"])
    run_test("PNG RGBA",
             {"rgba.png": _pil_png(300, 300, "RGBA")}, skip=pil_skip, types=["png"])
    run_test("PNG grayscale",
             {"gray.png": _pil_png(400, 300, "L")}, skip=pil_skip, types=["png"])
    run_test("PNG large (1920×1080)",
             {"large.png": _pil_png(1920, 1080)}, skip=pil_skip, types=["png"])
    run_test("PNG multiple adjacent",
             {"a.png": _pil_png(200, 200), "b.png": _pil_png(150, 150)},
             skip=pil_skip, types=["png"])

    # ── GIF ──────────────────────────────────────────────────────────────────
    print("\n  GIF")
    run_test("GIF87a (hand-crafted)",
             {"anim.gif": make_gif87a()}, types=["gif87a"])
    run_test("GIF89a animated (3 frames)",
             {"anim.gif": _pil_gif89a()}, skip=pil_skip, types=["gif89a"])

    # ── BMP ──────────────────────────────────────────────────────────────────
    print("\n  BMP")
    run_test("BMP 24-bit (320×240)",
             {"img.bmp": _pil_bmp()}, skip=pil_skip, types=["bmp"])

    # ── TIFF ─────────────────────────────────────────────────────────────────
    print("\n  TIFF")
    run_test("TIFF little-endian (512×512 RGB)",
             {"img.tif": _pil_tiff()}, skip=pil_skip, types=["tiff_le"])

    # ── RIFF containers ───────────────────────────────────────────────────────
    print("\n  RIFF (WAV / WEBP)")
    run_test("WAV PCM mono 44100Hz",
             {"audio.wav": make_wav()}, types=["riff"])
    run_test("WEBP lossy (400×300)",
             {"img.webp": _pil_webp()}, skip=pil_skip, types=["riff"])
    run_test("WAV + WEBP adjacent (RIFF subtype discrimination)",
             {"audio.wav": make_wav(), "img.webp": _pil_webp()},
             skip=pil_skip, types=["riff"])

    # ── PDF ───────────────────────────────────────────────────────────────────
    print("\n  PDF")
    run_test("PDF single page",
             {"doc.pdf": make_pdf()}, types=["pdf"])
    run_test("PDF linearized (two %%EOF — rfind test)",
             {"doc.pdf": make_pdf_linearized()}, types=["pdf"])

    # ── RTF ───────────────────────────────────────────────────────────────────
    print("\n  RTF")
    run_test("RTF document",
             {"doc.rtf": make_rtf()}, types=["rtf"])

    # ── Office Open XML (ZIP-based) ───────────────────────────────────────────
    print("\n  Office / ZIP")
    run_test("DOCX (ZIP with word/ prefix)",
             {"doc.docx": make_docx()}, types=["zip"])
    run_test("XLSX (ZIP with xl/ prefix)",
             {"sheet.xlsx": make_xlsx()}, types=["zip"])
    run_test("DOCX + XLSX adjacent (adjacent ZIP separation)",
             {"doc.docx": make_docx("first"), "sheet.xlsx": make_xlsx()},
             types=["zip"])

    # ── SQLite ────────────────────────────────────────────────────────────────
    print("\n  SQLite")
    run_test("SQLite database (20 rows)",
             {"data.db": make_sqlite()}, types=["sqlite"])

    # ── Mixed types ───────────────────────────────────────────────────────────
    print("\n  Mixed")
    run_test("JPEG + PNG + PDF together",
             {"img.jpg": _pil_jpeg(), "img.png": _pil_png(), "doc.pdf": make_pdf()},
             skip=pil_skip)
    run_test("JPEG + WAV + DOCX + SQLite together",
             {"img.jpg": _pil_jpeg(), "audio.wav": make_wav(),
              "doc.docx": make_docx(), "data.db": make_sqlite()},
             skip=pil_skip)

    # ── Extension spoofing ────────────────────────────────────────────────────
    print("\n  Extension spoofing")

    def ext_spoof_test():
        """Files with wrong extensions — PALA must carve by magic, not filename."""
        files = {
            "document.mp3": make_docx("spoofed as mp3"),
            "photo.csv":    _pil_jpeg() if HAS_PIL else None,
            "data.txt":     make_pdf(),
        }
        files = {k: v for k, v in files.items() if v is not None}
        run_test("Extension-spoofed files (DOCX→mp3, JPEG→csv, PDF→txt)",
                 files, skip=None if files else "Pillow missing, partial test only")

    ext_spoof_test()

    # ── False positives ───────────────────────────────────────────────────────
    print("\n  False positives")
    run_fp_test("20MB random data — 0 strong-magic FPs")

    # ── Summary ───────────────────────────────────────────────────────────────
    total = len(passed) + len(failed) + len(skipped)
    print()
    print("=" * 60)
    if failed:
        print(f"  {RED}FAILED{NC}:  {len(failed)}")
        for name, reason in failed:
            print(f"    - {name}: {reason}")
    print(f"  {GREEN}PASSED{NC}:  {len(passed)}")
    if skipped:
        print(f"  {YELLOW}SKIPPED{NC}: {len(skipped)}")
    print(f"  Total:   {total}")
    print()
    sys.exit(0 if not failed else 1)


if __name__ == "__main__":
    main()
