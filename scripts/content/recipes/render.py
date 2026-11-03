"""Render visual recipes with explicit ingredient groups and source provenance."""
import json
from pathlib import Path


def visual_item(value, labels, lang, manifest):
    if isinstance(value, list):
        items = [visual_item(v, labels, lang, manifest) for v in value]
        first = dict(items[0])
        first['name'] = ' / '.join(v['name'] for v in items)
        first['tag'] = ' / '.join(v.get('tag', v.get('id', '')) for v in items)
        return first
    if isinstance(value, str):
        value = {'tag': value[1:]} if value.startswith('#') else {'item': value}
    if 'tag' in value:
        tag = value['tag']
        representative = manifest.get('tags', {}).get(tag)
        fallback = tag.split(':', 1)[-1].replace('_', ' ').replace('/', ' · ')
        group = manifest.get('tag_names', {}).get(tag, {}).get(lang) or fallback
        return {'name': group, 'tag': tag, 'icon': representative or ''}
    item = value.get('item', value.get('id'))
    entry = manifest.get('items', {}).get(item, {})
    name = entry.get('name', {}).get(lang) or item_name(item, labels)
    return {'id': item, 'name': name, 'count': value.get('count', 1)}


def visual_recipe(recipe, labels, lang, manifest):
    kind = recipe['type']
    if kind == 'minecraft:crafting_shaped':
        rows = [row.ljust(3) for row in recipe['pattern']]
        rows += ['   '] * (3 - len(rows))
        slots = [None if c == ' ' else visual_item(recipe['key'][c], labels, lang, manifest)
                 for row in rows for c in row]
        kind = 'shaped'
    elif kind in ('minecraft:crafting_shapeless', 'oc2r:wrench'):
        slots = [visual_item(v, labels, lang, manifest) for v in recipe['ingredients']]
        slots += [None] * (9 - len(slots))
        kind = 'shapeless'
    elif kind == 'minecraft:smelting':
        slots = [visual_item(recipe['ingredient'], labels, lang, manifest)]
        kind = 'furnace'
    else:
        raise ValueError(f'Unsupported recipe type: {kind}')
    result = recipe['result']
    if isinstance(result, str):
        result = {'id': result}
    data = {'kind': kind, 'lang': lang, 'slots': slots,
            'output': visual_item(result, labels, lang, manifest)}
    if kind == 'furnace':
        data.update(seconds=recipe.get('cookingtime', 200) / 20,
                    experience=recipe.get('experience', 0))
    return ':::recipe\n' + json.dumps(data, ensure_ascii=False) + '\n:::'



def item_name(item, labels):
    namespace, name = item.split(':', 1)
    for kind in ('item', 'block'):
        if value := labels.get(f'{kind}.{namespace}.{name}'):
            return value
    return item


def ingredient(value, labels):
    if isinstance(value, list):
        return ' / '.join(ingredient(v, labels) for v in value)
    if isinstance(value, str):
        return f'`{value}`' if value.startswith('#') else item_text(value, labels)
    if 'tag' in value:
        return f'`#{value["tag"]}`'
    return item_text(value['item'], labels)


def item_text(item, labels):
    name = item_name(item, labels)
    return f'{name} (`{item}`)' if name != item else f'`{item}`'


def render(recipes, lang, labels, revision):
    ru = lang == 'ru'
    path = Path(__file__).resolve().parents[3] / 'web/public/items/manifest.json'
    manifest = json.loads(path.read_text()) if path.exists() else {}
    updates_path = Path(__file__).resolve().parents[3] / 'content/recipe-updates.json'
    updates = json.loads(updates_path.read_text()) if updates_path.exists() else {}
    title = 'Крафты и рецепты' if ru else 'Crafting recipes'
    intro = (
        f'Здесь собраны все {len(recipes)} рецептов из файлов OC2R указанной внизу ревизии. '
        'Сетка показывает расположение ингредиентов и результат. '
        'Запись с `#` означает тег: подойдёт любой предмет из этой группы. '
        'Например, `#c:ingots/iron` означает железные слитки, а `#c:chests/wooden` означает деревянные сундуки. '
        'Бесформенные рецепты не требуют определённого расположения. '
        'Рецепты сборки сервера могут отличаться, если их меняет датапак.'
        if ru else
        f'All {len(recipes)} recipes from the OC2R source revision linked below are collected here. '
        'The grid shows ingredients and output. A value beginning with `#` is an item tag: any item in that group works. '
        'For example, `#c:ingots/iron` accepts iron ingots and `#c:chests/wooden` accepts wooden chests. '
        'Shapeless ingredients can be placed in any order. Server datapacks can change these recipes.'
    )
    out = [f'# {title}', intro,
           ('Предметы без рецепта, в том числе оборудование только для творческого режима, сюда не включены.'
            if ru else 'Items without a recipe, including creative-only equipment, are not listed.'),
           '<details>', '<summary>' + ('Список рецептов' if ru else 'Recipe index') + f' · {len(recipes)}</summary>']
    for name, recipe in recipes:
        result = recipe['result']
        item = result if isinstance(result, str) else result['id']
        label = visual_item({'id': item}, labels, lang, manifest)['name']
        if name == 'unflash':
            label = 'Очистка прошивки' if ru else 'Clear firmware'
        out.append(f'- [{label}](#recipe-{name})')
    out.append('</details>')
    for name, recipe in recipes:
        result = recipe['result']
        if isinstance(result, str):
            result = {'id': result}
        label = visual_item(result, labels, lang, manifest)['name']
        if name == 'unflash':
            label = 'Очистка прошивки' if ru else 'Clear firmware'
        out.append(f'## {label} {{#recipe-{name}}}')
        kind = recipe['type']
        if update := updates.get(name):
            replacement = update['recipe']
            if replacement['result'] != recipe['result']:
                raise ValueError(f'Development recipe changes its output: {name}')
            notice = 'В разработке' if ru else 'In development'
            out.append(f":::version {notice}\n{update['note'][lang]}\n:::")
            recipe = replacement
        out.append(visual_recipe(recipe, labels, lang, manifest))
        if kind == 'oc2r:wrench':
            out.append('Ключ, если он указан среди ингредиентов, остаётся после крафта.' if ru else
                       'A wrench listed as an ingredient is returned after crafting.')
        components = result.get('components', {})
        if components:
            if result['id'] not in ('oc2r:computer', 'oc2r:robot'):
                raise ValueError(f'Undocumented output components: {name}')
            out.append('В комплекте уже установлена прошивка Linux. Процессор и ОЗУ добавьте отдельно.' if ru else
                       'Linux firmware is preinstalled. Install a CPU and RAM separately.')
        url = f'https://github.com/DivByDiamond/OC2R/blob/{revision}/src/main/resources/data/oc2r/recipe/{name}.json'
        out.append(f'[{"Файл рецепта" if ru else "Recipe source"}]({url})')
    return '\n\n'.join(out) + '\n'
