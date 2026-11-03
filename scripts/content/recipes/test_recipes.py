import json
import tempfile
import unittest
from pathlib import Path
from .render import render, visual_item
from .links import attach
from . import ORIGIN


class RecipeReferenceTest(unittest.TestCase):
    def test_tag_names_are_localized_with_safe_unknown_fallback(self):
        manifest = {'tags': {'c:ingots/iron': 'minecraft:iron_ingot'},
                    'tag_names': {'c:ingots/iron': {'ru': 'Железные слитки', 'en': 'Iron ingots'}}}
        item = visual_item({'tag': 'c:ingots/iron'}, {}, 'ru', manifest)
        self.assertEqual(item['name'], 'Железные слитки')
        self.assertEqual(item['tag'], 'c:ingots/iron')
        self.assertEqual(item['icon'], 'minecraft:iron_ingot')
        self.assertEqual(visual_item({'tag': 'mod:unknown/group'}, {}, 'ru', manifest)['name'], 'unknown · group')

    def test_shaped_grid_preserves_empty_cells_tags_and_output_count(self):
        recipe = {'type': 'minecraft:crafting_shaped', 'pattern': ['A A', ' B '],
                  'key': {'A': {'tag': 'c:ingots/iron'}, 'B': {'item': 'minecraft:redstone'}},
                  'result': {'id': 'oc2r:example', 'count': 4}}
        page = render([('example', recipe)], 'ru', {'item.oc2r.example': 'Пример'}, 'abc')
        data = json.loads(page.split(':::recipe\n')[1].split('\n:::')[0])
        self.assertEqual(data['kind'], 'shaped')
        self.assertEqual(len(data['slots']), 9)
        self.assertEqual([i for i, slot in enumerate(data['slots']) if slot], [0, 2, 4])
        self.assertEqual(data['slots'][0]['tag'], 'c:ingots/iron')
        self.assertEqual(data['output']['count'], 4)
        self.assertNotIn('```text', page)
        self.assertIn('<details>', page)
        self.assertIn('<summary>Список рецептов', page)
        self.assertIn('## Пример {#recipe-example}', page)
        self.assertIn('/blob/abc/src/main/resources/data/oc2r/recipe/example.json', page)

    def test_furnace_and_wrench_semantics_are_kept(self):
        recipes = [('silicon', {'type': 'minecraft:smelting', 'ingredient': {'item': 'oc2r:silicon_blend'},
                               'result': {'id': 'oc2r:silicon'}, 'cookingtime': 200, 'experience': 0.1}),
                   ('flash', {'type': 'oc2r:wrench', 'ingredients': [{'tag': 'oc2r:wrenches'}],
                              'result': {'id': 'oc2r:flash_memory'}})]
        page = render(recipes, 'ru', {}, 'abc')
        furnace = json.loads(page.split(':::recipe\n')[1].split('\n:::')[0])
        self.assertEqual(furnace['seconds'], 10)
        self.assertEqual(furnace['experience'], 0.1)
        self.assertEqual(len(furnace['slots']), 1)
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
            self.assertNotIn('[cpu_tier_1]', once)
            self.assertTrue(once.endswith('Source.\n'))
