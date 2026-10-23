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


class PhotoAttachmentTest(unittest.TestCase):
    def plan(self):
        return {'capture': {'minecraft': '26.1.2'},
                'photos': {'screen': {'file': 'screen-abcd.webp', 'width': 800, 'height': 450,
                                      'caption': {'en': 'A real screen', 'ru': 'Настоящий экран'}}},
                'pages': {'page': {'lead': 'screen', 'gallery': ['screen']}}}

    def test_photos_are_idempotent_and_source_stays_last(self):
        from photos import attach
        source = '# Page\n\nIntro.\n\n## Detail\n\nText.\n\n---\n\n[OC2R source](https://github.com/DivByDiamond/OC2R/blob/abc/file.md) · `abc`\n'
        once = attach(source, 'ru', 'page', self.plan())
        self.assertEqual(attach(once, 'ru', 'page', self.plan()), once)
        self.assertIn('Настоящий экран', once)
        self.assertIn('width="800" height="450" loading="lazy"', once)
        self.assertTrue(once.endswith(source[source.rfind('\n---'):]))
        self.assertIn('## Detail\n\nText.', once)

    def test_unillustrated_pages_are_untouched(self):
        from photos import attach
        source = '# Other\n\nEditor text.\n'
        self.assertEqual(attach(source, 'en', 'other', self.plan()), source)


if __name__ == '__main__':
    unittest.main()
