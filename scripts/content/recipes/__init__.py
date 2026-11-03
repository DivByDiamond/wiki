"""Generate recipe reference pages from the mod's recipe JSON files."""
import hashlib
import json
from .render import render

ORIGIN = 'src/main/resources/data/oc2r/recipe'


def pages(source, revision):
    files = sorted((source / ORIGIN).glob('*.json'))
    if not files:
        raise ValueError('No OC2R recipes found')
    digest = hashlib.sha256()
    recipes = []
    for path in files:
        digest.update(path.name.encode() + b'\0' + path.read_bytes() + b'\0')
        recipes.append((path.stem, json.loads(path.read_text())))
    for lang, locale in [('ru', 'ru_ru'), ('en', 'en_us')]:
        labels = json.loads((source / f'src/main/resources/assets/oc2r/lang/{locale}.json').read_text())
        yield lang, render(recipes, lang, labels, revision), ORIGIN, digest.hexdigest()
