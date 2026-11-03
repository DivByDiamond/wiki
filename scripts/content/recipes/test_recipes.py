import tempfile
import unittest
from pathlib import Path
from .render import render
from .links import attach
from . import ORIGIN


class RecipeReferenceTest(unittest.TestCase):
    def test_shaped_grid_preserves_empty_cells_tags_and_output_count(self):
        recipe = {'type': 'minecraft:crafting_shaped', 'pattern': ['A A', ' B '],
                  'key': {'A': {'tag': 'c:ingots/iron'}, 'B': {'item': 'minecraft:redstone'}},
                  'result': {'id': 'oc2r:example', 'count': 4}}
        page = render([('example', recipe)], 'ru', {'item.oc2r.example': 'Пример'}, 'abc')
        self.assertIn('A . A\n. B .\n. . .', page)
        self.assertIn('`#c:ingots/iron`', page)
        self.assertIn('× 4', page)
        self.assertIn('## Пример {#recipe-example}', page)
        self.assertIn('/blob/abc/src/main/resources/data/oc2r/recipe/example.json', page)

    def test_furnace_and_wrench_semantics_are_kept(self):
        recipes = [('silicon', {'type': 'minecraft:smelting', 'ingredient': {'item': 'oc2r:silicon_blend'},
                               'result': {'id': 'oc2r:silicon'}, 'cookingtime': 200, 'experience': 0.1}),
                   ('flash', {'type': 'oc2r:wrench', 'ingredients': [{'tag': 'oc2r:wrenches'}],
                              'result': {'id': 'oc2r:flash_memory'}})]
        page = render(recipes, 'ru', {}, 'abc')
        self.assertIn('10 с при 20 TPS', page)
        self.assertIn('Опыт: 0.1', page)
        self.assertIn('остаётся после крафта', page)
        with self.assertRaises(ValueError):
            render([('bad', {'type': 'oc2r:unknown', 'result': {'id': 'oc2r:bad'}})], 'en', {}, 'abc')

    def test_recipe_links_are_idempotent_and_keep_source_footer(self):
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory)
            folder = source / ORIGIN
            folder.mkdir(parents=True)
            (folder / 'cpu_tier_1.json').write_text('{"result":{"id":"oc2r:cpu_tier_1"}}')
            pages = {('ru', 'item/cpu'): {'body': '# CPU\n\nText.\n\n---\n\nSource.\n'}}
            attach(pages, source)
            once = pages[('ru', 'item/cpu')]['body']
            attach(pages, source)
            self.assertEqual(once, pages[('ru', 'item/cpu')]['body'])
            self.assertIn('/ru/recipes#recipe-cpu_tier_1', once)
            self.assertTrue(once.endswith('Source.\n'))
