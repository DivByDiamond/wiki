import unittest
from adapt import convert


class ConversionTest(unittest.TestCase):
    def test_nested_links_and_english_fallback(self):
        body = '[Computer](../block/computer.md)\n![Screenshot](../../img/a.png)'
        result = convert(body, 'item/cpu.md', 'ru', {('en', 'block/computer')})
        self.assertIn('(/ru/block/computer)', result)
        self.assertIn('(/static/manual/a.png)', result)

    def test_game_ids_are_text_and_code_is_preserved(self):
        code = '```md\n[x](missing.md)\n```'
        result = convert('![GPU](item:oc2r:gpu_tier_1)\n' + code, 'item/gpu.md', 'en', set())
        self.assertIn('`oc2r:gpu_tier_1`', result)
        self.assertIn(code, result)

    def test_broken_link_fails_generation(self):
        with self.assertRaises(ValueError):
            convert('[x](missing.md)', 'index.md', 'en', set())


if __name__ == '__main__':
    unittest.main()
