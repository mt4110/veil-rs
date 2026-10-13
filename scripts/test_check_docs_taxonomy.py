from pathlib import Path
import tempfile
import unittest
from check_docs_taxonomy import iter_docs, classify, RULES


class PublicDocsTaxonomyTests(unittest.TestCase):
    def test_private_and_sales_documents_never_enter_the_oss_inventory(self):
        with tempfile.TemporaryDirectory() as scratch:
            root = Path(scratch)
            for name in ['docs/cli/scan.md', '.private/sales/internal.md', '.design/internal.md', 'docs/sales/internal.md']:
                p = root / name
                p.parent.mkdir(parents=True, exist_ok=True)
                p.write_text('synthetic document')
            self.assertEqual([p.relative_to(root).as_posix() for p in iter_docs(root)], ['docs/cli/scan.md'])

    def test_unknown_public_document_is_still_unclassified(self):
        root = Path('/synthetic-root')
        self.assertIsNone(classify(root / 'docs/new-unreviewed-guide.md', root))
        self.assertEqual(classify(root / 'docs/cli/scan.md', root), 'reference')

    def test_public_rules_do_not_encode_private_file_identities(self):
        self.assertTrue(all(pattern.startswith('docs/') for pattern, status in RULES))
        self.assertFalse(any(pattern.startswith('docs/sales/') for pattern, status in RULES))
