"""Render shaped, shapeless, wrench and furnace recipes without guessing ingredients."""


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
    title = 'Крафты и рецепты' if ru else 'Crafting recipes'
    intro = (
        f'Здесь собраны все {len(recipes)} рецептов из файлов OC2R указанной внизу ревизии. '
        'Сетка показывает расположение ингредиентов. Точка обозначает пустую ячейку. '
        'Запись с `#` означает тег: подойдёт любой предмет из этой группы. '
        'Например, `#c:ingots/iron` означает железные слитки, а `#c:chests/wooden` означает деревянные сундуки. '
        'Бесформенные рецепты не требуют определённого расположения. '
        'Рецепты сборки сервера могут отличаться, если их меняет датапак.'
        if ru else
        f'All {len(recipes)} recipes from the OC2R source revision linked below are collected here. '
        'Dots mark empty crafting slots. A value beginning with `#` is an item tag: any item in that group works. '
        'For example, `#c:ingots/iron` accepts iron ingots and `#c:chests/wooden` accepts wooden chests. '
        'Shapeless ingredients can be placed in any order. Server datapacks can change these recipes.'
    )
    out = [f'# {title}', intro,
           ('Предметы без рецепта, в том числе оборудование только для творческого режима, сюда не включены.'
            if ru else 'Items without a recipe, including creative-only equipment, are not listed.'),
           '## ' + ('Список рецептов' if ru else 'Recipe index')]
    for name, recipe in recipes:
        result = recipe['result']
        item = result if isinstance(result, str) else result['id']
        label = item_name(item, labels)
        if name == 'unflash':
            label = 'Очистка прошивки' if ru else 'Clear firmware'
        out.append(f'- [{label}](#recipe-{name})')
    for name, recipe in recipes:
        result = recipe['result']
        if isinstance(result, str):
            result = {'id': result}
        label = item_name(result['id'], labels)
        if name == 'unflash':
            label = 'Очистка прошивки' if ru else 'Clear firmware'
        out.extend([f'## {label} {{#recipe-{name}}}',
                    ('Результат: ' if ru else 'Output: ') +
                    f'{item_text(result["id"], labels)} × {result.get("count", 1)}.'])
        kind = recipe['type']
        if kind == 'minecraft:crafting_shaped':
            pattern = recipe['pattern']
            rows = [row.ljust(3) for row in pattern] + ['   '] * (3 - len(pattern))
            out.append(('Верстак, сетка 3×3.' if ru else 'Crafting table, 3×3 grid.'))
            out.append('```text\n' + '\n'.join(' '.join(c if c != ' ' else '.' for c in row) for row in rows) + '\n```')
            out.append('\n'.join(f'- `{key}`: {ingredient(value, labels)}' for key, value in sorted(recipe['key'].items())))
        elif kind in ('minecraft:crafting_shapeless', 'oc2r:wrench'):
            out.append('Бесформенный рецепт.' if ru else 'Shapeless recipe.')
            out.append('\n'.join('- ' + ingredient(v, labels) for v in recipe['ingredients']))
            if kind == 'oc2r:wrench':
                out.append('Ключ, если он указан среди ингредиентов, остаётся после крафта.' if ru else
                           'A wrench listed as an ingredient is returned after crafting.')
        elif kind == 'minecraft:smelting':
            out.append(('Печь. Вход: ' if ru else 'Furnace. Input: ') + ingredient(recipe['ingredient'], labels) + '.')
            seconds = recipe.get('cookingtime', 200) / 20
            out.append((f'Время: {seconds:g} с при 20 TPS. Опыт: {recipe.get("experience", 0):g}.' if ru else
                        f'Time: {seconds:g} s at 20 TPS. Experience: {recipe.get("experience", 0):g}.'))
        else:
            raise ValueError(f'Unsupported recipe type: {kind}')
        components = result.get('components', {})
        if components:
            if result['id'] not in ('oc2r:computer', 'oc2r:robot'):
                raise ValueError(f'Undocumented output components: {name}')
            out.append('В комплекте уже установлена прошивка Linux. Процессор и ОЗУ добавьте отдельно.' if ru else
                       'Linux firmware is preinstalled. Install a CPU and RAM separately.')
        url = f'https://github.com/DivByDiamond/OC2R/blob/{revision}/src/main/resources/data/oc2r/recipe/{name}.json'
        out.append(f'[{"Файл рецепта" if ru else "Recipe source"}]({url})')
    return '\n\n'.join(out) + '\n'
