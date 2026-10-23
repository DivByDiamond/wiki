#!/usr/bin/env python3
"""Export reviewed, pixel-preserving crops from the user's Minecraft screenshots."""
import argparse
import hashlib
import io
import json
from pathlib import Path
from PIL import Image

ROOT = Path(__file__).resolve().parents[2]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('session', type=Path, help='Photo-session directory containing originals/')
    args = parser.parse_args()
    manifest = ROOT / 'content/illustrations.json'
    data = json.loads(manifest.read_text())
    out = ROOT / 'web/public/photos'
    out.mkdir(parents=True, exist_ok=True)
    total = 0
    for name, entry in data['photos'].items():
        source = args.session / 'originals' / entry['source']
        with Image.open(source) as original:
            left, top, right, bottom = entry['crop']
            assert 0 <= left < right <= original.width and 0 <= top < bottom <= original.height, name
            crop = original.convert('RGB').crop((left, top, right, bottom))
            encoded = io.BytesIO()
            crop.save(encoded, format='WEBP', lossless=True, method=6)
            payload = encoded.getvalue()
            with Image.open(io.BytesIO(payload)) as decoded:
                assert decoded.convert('RGB').tobytes() == crop.tobytes(), name
            digest = hashlib.sha256(payload).hexdigest()
            file = f'{name}-{digest[:10]}.webp'
            (out / file).write_bytes(payload)
            entry.update(file=file, width=crop.width, height=crop.height,
                         source_sha256=hashlib.sha256(source.read_bytes()).hexdigest(),
                         sha256=digest)
            total += len(payload)
    manifest.write_text(json.dumps(data, ensure_ascii=False, indent=2) + '\n')
    print(f'Exported {len(data["photos"])} verified lossless crops, {total:,} bytes')


if __name__ == '__main__':
    main()
