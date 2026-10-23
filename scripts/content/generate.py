#!/usr/bin/env python3
"""Rebuild the vendored, reviewable content bundle from an OC2R checkout."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
from adapt import LOCALES, convert, section_for, slug_for
from photos import attach

ROOT = Path(__file__).resolve().parents[2]
TECHNICAL = {
    "docs/ONYXOS.md": "systems/onyxos-reference",
    "docs/BUILDROOT.md": "systems/buildroot",
    "docs/ONYX_RPC.md": "reference/onyx-rpc",
    "docs/DEVICES.md": "reference/devices",
    "docs/NETWORKING.md": "reference/networking",
    "docs/CABLE-SYSTEM.md": "reference/cables",
}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path)
    args = parser.parse_args()
    source = args.source.resolve()
    manual = source / "src/main/resources/assets/oc2r/doc"
    revision = subprocess.check_output(["git", "-C", str(source), "rev-parse", "HEAD"], text=True).strip()
    dirty = bool(subprocess.check_output(["git", "-C", str(source), "status", "--porcelain"], text=True).strip())
    if dirty:
        parser.error("Source checkout has uncommitted changes; use a clean revision for reproducible source links.")
    url = f"https://github.com/DivByDiamond/OC2R/blob/{revision}/"
    inputs = [(lang, slug_for(p.relative_to(manual / locale).as_posix()), p)
              for locale, lang in LOCALES.items() for p in sorted((manual / locale).rglob("*.md"))]
    known = {(lang, slug) for lang, slug, _ in inputs}
    pages = {}

    def add(lang, slug, body, origin, digest):
        labels = {"ru": "Источник OC2R", "en": "OC2R source", "zh": "OC2R 来源"}
        body = body.rstrip() + f"\n\n---\n\n[{labels[lang]}]({url}{origin}) · `{revision[:12]}`\n"
        first = next((line[2:].strip() for line in body.splitlines() if line.startswith("# ")), slug)
        pages[(lang, slug)] = dict(lang=lang, slug=slug, title=first, section=section_for(slug),
                                  position=0, body=body, source=origin, source_sha256=digest)

    for lang, slug, path in inputs:
        locale = next(k for k, v in LOCALES.items() if v == lang)
        body = convert(path.read_text(), path.relative_to(manual / locale).as_posix(), lang, known)
        add(lang, slug, body, path.relative_to(source).as_posix(), hashlib.sha256(path.read_bytes()).hexdigest())
    # Technical documents retain links to files in the exact source revision.
    import re
    for origin, slug in TECHNICAL.items():
        path = source / origin
        body = path.read_text()
        def technical_link(m):
            dest = m.group(2)
            if re.match(r"[a-z]+:", dest) or dest.startswith("#"):
                return m.group(0)
            import posixpath
            target = posixpath.normpath(posixpath.join(posixpath.dirname(origin), dest))
            return f"[{m.group(1)}]({url}{target})"
        body = re.sub(r"\[([^\]]*)\]\(([^)]+)\)", technical_link, body)
        add("en", slug, body, origin, hashlib.sha256(path.read_bytes()).hexdigest())
    for path in sorted((ROOT / "content/overrides").rglob("*.md")):
        lang, *parts = path.relative_to(ROOT / "content/overrides").parts
        slug = "/".join(parts).removesuffix(".md")
        raw = path.read_text()
        origin, expected, body = raw.split("\n", 2)
        origin = origin.removeprefix("source: ")
        digest = hashlib.sha256((source / origin).read_bytes()).hexdigest()
        if expected != f"source_sha256: {digest}":
            raise ValueError(f"Review curated article {path}: its OC2R source changed. Update source_sha256 after reviewing the text.")
        add(lang, slug, body.strip(), origin, digest)
    preferred = ["start", "getting_started", "basics", "block/index", "item/index", "networking", "robotics", "systems/overview", "systems/onyxos", "scripting"]
    for page in pages.values():
        page["position"] = preferred.index(page["slug"]) if page["slug"] in preferred else 100
    illustrations = json.loads((ROOT / "content/illustrations.json").read_text())
    for page in pages.values():
        page["body"] = attach(page["body"], page["lang"], page["slug"], illustrations)
    output = dict(source_revision=revision, source_dirty=dirty, pages=sorted(pages.values(), key=lambda p: (p["lang"], p["slug"])))
    (ROOT / "content/catalog.json").write_text(json.dumps(output, ensure_ascii=False, indent=2) + "\n")
    for path in (manual / "img").glob("*.png"):
        shutil.copy2(path, ROOT / "web/public/manual" / path.name)
    counts = {lang: sum(p["lang"] == lang for p in pages.values()) for lang in LOCALES.values()}
    print(json.dumps(dict(pages=counts, source_revision=revision, source_dirty=dirty), ensure_ascii=False))


if __name__ == "__main__":
    main()
