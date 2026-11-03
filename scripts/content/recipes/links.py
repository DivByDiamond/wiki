"""Point equipment articles at the central recipe reference."""
import json
import re
from pathlib import Path
from . import ORIGIN


def attach(pages, source):
    manifest = json.loads((Path(__file__).resolve().parents[3] / 'web/public/items/manifest.json').read_text())
    recipes = {}
    results = {}
    for path in sorted((source / ORIGIN).glob('*.json')):
        recipe = json.loads(path.read_text())
        result = recipe['result']
        item = (result if isinstance(result, str) else result['id']).split(':', 1)[1]
        base = re.sub(r'_tier_\d+$', '', item)
        if base.startswith(('memory_', 'hard_drive_', 'flash_memory_')):
            base = next(prefix for prefix in ('memory', 'hard_drive', 'flash_memory') if base.startswith(prefix))
        if base == 'floppy_modern':
            base = 'floppy'
        recipes.setdefault(base, []).append(path.stem)
        results[path.stem] = result if isinstance(result, str) else result['id']
    for (lang, slug), page in pages.items():
        body = re.sub(r'\n*<!-- wiki-recipes -->.*?<!-- /wiki-recipes -->\n*', '\n\n', page['body'], flags=re.S)
        names = recipes.get(slug.split('/', 1)[-1], []) if slug.startswith(('item/', 'block/')) else []
        if slug in ('item/index', 'block/index'):
            names = ['']
        if not names:
            page['body'] = body
            continue
        title = {'ru': 'Крафты и рецепты', 'en': 'Crafting recipes', 'zh': '合成配方'}[lang]
        def label(name):
            if not name:
                return title
            if name == 'unflash':
                return {'ru': 'Очистка прошивки', 'en': 'Clear firmware', 'zh': '清除固件'}[lang]
            return manifest['items'].get(results[name], {}).get('name', {}).get(lang) or name
        links = '\n'.join(f'- [{label(name)}](/{lang}/recipes' + (f'#recipe-{name}' if name else '') + ')' for name in names)
        block = f'<!-- wiki-recipes -->\n## {title}\n\n{links}\n<!-- /wiki-recipes -->'
        content, separator, footer = body.rpartition('\n---\n')
        page['body'] = (content.rstrip() + '\n\n' + block + '\n\n---\n' + footer) if separator else body.rstrip() + '\n\n' + block + '\n'
