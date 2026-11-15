#!/usr/bin/env python3
"""Create an opt-in reading preview; keep the pixel-preserving original intact."""
import hashlib
import io
import json
from pathlib import Path
from PIL import Image

ROOT = Path(__file__).resolve().parents[2]


def main():
    manifest = ROOT / 'content/illustrations.json'
    data = json.loads(manifest.read_text())
    photo = data['photos']['laboratory-overview']
    directory = ROOT / 'web/public/photos'
    with Image.open(directory / photo['file']) as original:
        width = min(960, original.width)
        height = round(original.height * width / original.width)
        preview = original.resize((width, height), Image.Resampling.LANCZOS)
        encoded = io.BytesIO()
        preview.save(encoded, format='WEBP', quality=82, method=6)
    payload = encoded.getvalue()
    digest = hashlib.sha256(payload).hexdigest()
    filename = f'laboratory-overview-preview-{digest[:10]}.webp'
    (directory / filename).write_bytes(payload)
    photo['preview'] = {'file': filename, 'width': width, 'height': height,
                        'sha256': digest}
    manifest.write_text(json.dumps(data, ensure_ascii=False, indent=2) + '\n')
    print(f'Reading preview: {width}x{height}, {len(payload):,} bytes; original preserved')


if __name__ == '__main__':
    main()
