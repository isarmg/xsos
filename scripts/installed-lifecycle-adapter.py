"""XSOS assertions for XSSI's real installed lifecycle gate (no database fixtures)."""
import json
import secrets

PRODUCT = 'xsos'
SERVICES = ('xsos.service',)
ACCOUNT = 'isarmg-xsos'
RETAINED_FILES = ('state/xsos.sqlite3', 'state/secrets.json')
INSTANCES = '/api/v1/monitoring/client-instances'


def configure(config, root, shared, port):
    config.update(bind=f'127.0.0.1:{port}', development=False)


def api(request, path, method='GET', body=None, csrf=None, expected=200):
    headers = {'Content-Type': 'application/json'}
    if csrf:
        headers['X-CSRF-Token'] = csrf
    status, response = request(path, method, None if body is None else json.dumps(body).encode(), headers)
    if status != expected:
        raise RuntimeError('Unexpected business API status')
    if expected == 204:
        if response:
            raise RuntimeError('Expected empty business API response')
        return None
    try:
        return json.loads(response)
    except (ValueError, UnicodeError):
        raise RuntimeError('Invalid business API JSON') from None


def exercise(request, session, sensitive, require):
    csrf = session['csrf_token']
    created = api(request, INSTANCES, 'POST',
                  {'display_name': 'Original lifecycle instance'}, csrf, expected=201)
    # Register both secret fields before checking anything; never log API bodies.
    sensitive.extend((created['activation_code'], created['authorization_code']))
    require(created['display_name'] == 'Original lifecycle instance'
            and created['status'] == 'pending'
            and created['activation_code'] == created['authorization_code'],
            'Instance creation differs')
    path = INSTANCES + '/' + created['request_id']
    api(request, path, 'PATCH', {'display_name': 'Renamed lifecycle instance'}, csrf, expected=204)
    code = secrets.token_hex(18)
    sensitive.append(code)
    authorization = api(request, path + '/authorization', 'PUT', {'authorization_code': code}, csrf)
    expected = {key: created[key] for key in ('request_id', 'instance_id', 'created_at')}
    expected.update(display_name='Renamed lifecycle instance', status='pending', authorization_code=code)
    require(authorization == expected, 'Instance rename or authorization differs')
    instances = api(request, INSTANCES)
    require(instances == {'instances': [expected], 'hosts': [],
                          'next_cursor': None, 'previous_cursor': None}, 'Instance list differs')
    return {'instances': instances}


def persisted(request, session, state, require):
    current = api(request, '/api/v1/auth/session')
    require(current['authenticated'] is True and current['user_id'] == session['user_id'],
            'Administrator session did not survive restart')
    require(api(request, INSTANCES) == state['instances'],
            'Instances and authorization did not survive restart')


def retained(root, shared, state, require):
    require(all((root / relative).is_file() for relative in RETAINED_FILES),
            'Operator uninstall did not retain private data and secrets')
