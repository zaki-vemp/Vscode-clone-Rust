import unittest
from datetime import date
from scripts.dependency_audit import violations


class AdvisoryGateTests(unittest.TestCase):
    def test_review_is_specific_to_version_and_expires(self):
        result = {'findings': [{'name': 'example', 'version': '1.2.3', 'advisories': ['EXAMPLE-1']}]}
        policy = {'reviews': [{'name': 'example', 'version': '1.2.3', 'advisory': 'EXAMPLE-1',
                              'reason': 'Reachability reviewed', 'review_by': '2026-12-02'}]}
        self.assertEqual(violations(result, policy, date(2026, 11, 2)), [])
        self.assertTrue(violations(result, policy, date(2026, 12, 3)))
        result['findings'][0]['version'] = '1.2.4'
        self.assertTrue(violations(result, policy, date(2026, 11, 2)))
        result['findings'][0]['version'] = '1.2.3'
        result['findings'][0]['advisories'].append('EXAMPLE-2')
        self.assertTrue(violations(result, policy, date(2026, 11, 2)))


if __name__ == '__main__':
    unittest.main()
