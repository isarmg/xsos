#!/usr/bin/env python3
"""Adapter contract regressions, not substitutes for the real installed CI gate."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('adapter', Path(__file__).with_name('installed-lifecycle-adapter.py'))
adapter = importlib.util.module_from_spec(spec)
spec.loader.exec_module(adapter)


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


class AdapterContract(unittest.TestCase):
    def test_production_configuration(self):
        config = {}
        adapter.configure(config, Path('/root'), Path('/shared'), 12345)
        self.assertEqual(config, {'development': False, 'bind': '127.0.0.1:12345'})

    def test_empty_204_is_not_decoded(self):
        with patch.object(adapter.json, 'loads', side_effect=AssertionError('must not decode')):
            self.assertIsNone(adapter.api(lambda *args: (204, b''), '/instance', expected=204))
        for status, body in ((200, b'{}'), (204, b'private-response')):
            with self.subTest(status=status, body=bool(body)):
                with self.assertRaises(RuntimeError) as error:
                    adapter.api(lambda *args: (status, body), '/instance', expected=204)
                self.assertNotIn('private-response', str(error.exception))

    def test_invalid_json_does_not_disclose_body(self):
        with self.assertRaisesRegex(RuntimeError, '^Invalid business API JSON$'):
            adapter.api(lambda *args: (200, b'private-response'), '/instance')

    def test_business_and_persisted_session_contract(self):
        # Shapes match CreatedClientInstance, ClientInstanceSummary and
        # ClientInstanceListResponse in model.rs; PATCH in http.rs has no JSON.
        instance = {'request_id': '11111111-1111-4111-8111-111111111111',
                    'instance_id': '22222222-2222-4222-8222-222222222222',
                    'display_name': 'Original lifecycle instance', 'status': 'pending',
                    'created_at': '2026-10-14T00:00:00Z', 'authorization_code': 'a' * 36}
        session = {'csrf_token': 'csrf', 'authenticated': True, 'user_id': 'admin-id'}
        calls = []

        def request(path, method, body, headers):
            calls.append((path, method))
            data = None if body is None else json.loads(body)
            if method != 'GET':
                self.assertEqual(headers['X-CSRF-Token'], session['csrf_token'])
            if path == '/api/v1/auth/session':
                result = session
            elif method == 'POST':
                self.assertEqual(path, adapter.INSTANCES)
                self.assertEqual(data, {'display_name': 'Original lifecycle instance'})
                result = {**instance, 'activation_code': instance['authorization_code']}
            elif method == 'PATCH':
                self.assertEqual(path, adapter.INSTANCES + '/' + instance['request_id'])
                self.assertEqual(data, {'display_name': 'Renamed lifecycle instance'})
                instance.update(data)
                return 204, b''
            elif method == 'PUT':
                self.assertEqual(path, adapter.INSTANCES + '/' + instance['request_id'] + '/authorization')
                self.assertEqual(set(data), {'authorization_code'})
                instance.update(data)
                result = instance
            else:
                self.assertEqual(path, adapter.INSTANCES)
                result = {'instances': [instance], 'hosts': [], 'next_cursor': None, 'previous_cursor': None}
            return (201 if method == 'POST' else 200), json.dumps(result).encode()

        sensitive = []
        state = adapter.exercise(request, session, sensitive, require)
        adapter.persisted(request, session, state, require)
        self.assertEqual(sensitive[:2], ['a' * 36, 'a' * 36])
        self.assertEqual(sensitive[2], instance['authorization_code'])
        self.assertRegex(sensitive[2], r'^[0-9a-f]{36}$')
        self.assertEqual(len(calls), 6)
        instance['authorization_code'] = 'b' * 36
        with self.assertRaisesRegex(RuntimeError, 'authorization did not survive'):
            adapter.persisted(request, session, state, require)
        session['authenticated'] = False
        with self.assertRaisesRegex(RuntimeError, 'session did not survive'):
            adapter.persisted(request, session, state, require)

    def test_retains_database_and_secrets(self):
        self.assertEqual(adapter.RETAINED_FILES, ('state/xsos.sqlite3', 'state/secrets.json'))
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / 'state').mkdir()
            (root / 'state/xsos.sqlite3').touch()
            with self.assertRaisesRegex(RuntimeError, 'retain private data and secrets'):
                adapter.retained(root, None, None, require)
            (root / 'state/secrets.json').touch()
            adapter.retained(root, None, None, require)


if __name__ == '__main__':
    unittest.main()
