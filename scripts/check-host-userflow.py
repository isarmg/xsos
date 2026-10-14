#!/usr/bin/env python3
"""Real xsos + xsoc candidates, private fixture state, trusted loopback TLS.

This is an application pairing/reporting/restart test, not an installation test.
No credentials are seeded on the client and no application API is mocked.
"""
import argparse
import hashlib
import http.client
import http.cookiejar
import http.server
import json
import os
from pathlib import Path
import socket
import ssl
import subprocess
import tempfile
import threading
import time
import urllib.request


def require(value, message):
    if not value:
        raise RuntimeError(message)


def run(*command, input=None):
    environment = {key: value for key, value in os.environ.items()
                   if not key.startswith('XSOC_') and key.lower() not in ('http_proxy', 'https_proxy', 'all_proxy')}
    result = subprocess.run(command, input=input, stdout=subprocess.PIPE,
                            stderr=subprocess.PIPE, timeout=90, check=False, env=environment)
    if result.returncode:
        # The command's raw output can contain machine identity or pairing material.
        try:
            code = json.loads(result.stdout).get('error', {}).get('code', 'unknown')
        except (ValueError, AttributeError):
            code = 'non-json failure'
        raise RuntimeError(f'{Path(command[0]).name} exited {result.returncode}: {code}')
    return result.stdout


def tls_frontend(base, backend_port):
    """A loopback-only TLS terminator, matching the supported production topology."""
    receipts = []
    certificate, key = base / 'cert.pem', base / 'key.pem'
    ca, ca_key = base / 'ca.pem', base / 'ca-key.pem'
    run('openssl', 'req', '-x509', '-newkey', 'rsa:2048', '-nodes', '-days', '1',
        '-subj', '/CN=Temporary host-flow CA', '-addext', 'basicConstraints=critical,CA:TRUE',
        '-keyout', str(ca_key), '-out', str(ca))
    csr, extensions = base / 'server.csr', base / 'server.ext'
    extensions.write_text('subjectAltName=IP:127.0.0.1\nbasicConstraints=critical,CA:FALSE\nkeyUsage=critical,digitalSignature,keyEncipherment\nextendedKeyUsage=serverAuth\n')
    run('openssl', 'req', '-new', '-newkey', 'rsa:2048', '-nodes', '-subj', '/CN=127.0.0.1',
        '-keyout', str(key), '-out', str(csr))
    run('openssl', 'x509', '-req', '-in', str(csr), '-CA', str(ca), '-CAkey', str(ca_key),
        '-CAcreateserial', '-days', '1', '-extfile', str(extensions), '-out', str(certificate))
    key.chmod(0o600)
    ca_key.chmod(0o600)

    class Proxy(http.server.BaseHTTPRequestHandler):
        def log_message(self, *_args):
            pass

        def forward(self):
            length = int(self.headers.get('Content-Length', '0'))
            if not 0 <= length <= 512 * 1024:
                self.send_error(413)
                return
            connection = http.client.HTTPConnection('127.0.0.1', backend_port, timeout=5)
            try:
                connection.request(self.command, self.path, self.rfile.read(length), dict(self.headers))
                response = connection.getresponse()
                body = response.read(8 * 1024 * 1024 + 1)
                receipts.append((self.command, self.path.split('?')[0], response.status))
                require(len(body) <= 8 * 1024 * 1024, "Response exceeds smoke budget")
                self.send_response(response.status)
                for name, value in response.getheaders():
                    if name.lower() not in ('connection', 'transfer-encoding', 'content-length'):
                        self.send_header(name, value)
                self.send_header('Content-Length', str(len(body)))
                self.end_headers()
                self.wfile.write(body)
            except OSError:
                self.send_error(502)
            finally:
                connection.close()

        do_GET = do_POST = do_PUT = forward

    server = http.server.ThreadingHTTPServer(('127.0.0.1', 0), Proxy)
    context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
    context.load_cert_chain(certificate, key)
    server.socket = context.wrap_socket(server.socket, server_side=True)
    threading.Thread(target=server.serve_forever, daemon=True).start()
    server.smoke_receipts = receipts
    return server, ssl.create_default_context(cafile=str(ca))


def write_private(path, value):
    with path.open('x') as stream:
        json.dump(value, stream)
    path.chmod(0o600)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('server', 'client', 'fixture'):
        parser.add_argument('--' + name, type=Path, required=True)
    args = parser.parse_args()
    binaries = {name: getattr(args, name).resolve(strict=True) for name in ('server', 'client', 'fixture')}
    digests = {}
    for name, binary in binaries.items():
        with binary.open('rb') as stream:
            digests[name] = hashlib.file_digest(stream, 'sha256').hexdigest()
    os.umask(0o077)
    with tempfile.TemporaryDirectory(prefix='xsos-xsoc-flow-') as temporary:
        base = Path(temporary)
        root = base / 'server'
        root.mkdir(mode=0o700)
        for part in ('config', 'logs', 'run'):
            (root / part).mkdir(mode=0o700)
        with socket.socket() as listener:
            listener.bind(('127.0.0.1', 0))
            port = listener.getsockname()[1]
        write_private(root / 'config/config.json', {'bind': f'127.0.0.1:{port}', 'development': False})
        run(str(binaries['fixture']), str(root))
        server = None
        proxy = None
        log = (root / 'logs/process.log').open('wb')
        def stop():
            nonlocal server
            server.terminate()
            try:
                require(server.wait(timeout=15) == 0, 'Server did not shut down cleanly')
            except subprocess.TimeoutExpired:
                server.kill()
                server.wait(timeout=5)
                raise RuntimeError('Server shutdown timed out') from None
            server = None
        def start():
            nonlocal server
            server = subprocess.Popen([str(binaries['server']), '--root', str(root)], stdout=log, stderr=log)
            deadline = time.monotonic() + 15
            while time.monotonic() < deadline:
                require(server.poll() is None, 'Server exited before readiness')
                try:
                    with urllib.request.urlopen(f'http://127.0.0.1:{port}/readyz', timeout=1) as response:
                        if response.status == 200:
                            return
                except OSError:
                    pass
                time.sleep(.1)
            raise RuntimeError('Server readiness timed out')
        try:
            start()
            proxy, context = tls_frontend(base, port)
            origin = f'https://127.0.0.1:{proxy.server_port}'
            opener = urllib.request.build_opener(urllib.request.ProxyHandler({}),
                urllib.request.HTTPSHandler(context=context), urllib.request.HTTPCookieProcessor(http.cookiejar.CookieJar()))
            def api(path, payload=None, csrf=None):
                headers = {'Origin': origin, 'Sec-Fetch-Site': 'same-origin'}
                if csrf:
                    headers['X-CSRF-Token'] = csrf
                data = None
                if payload is not None:
                    headers['Content-Type'] = 'application/json'
                    data = json.dumps(payload).encode()
                with opener.open(urllib.request.Request(origin + path, data=data, headers=headers), timeout=10) as response:
                    return json.load(response)
            session = api('/api/v1/auth/login', {'username': 'fixture-admin', 'password': 'fixture-password-2026'})
            instance = api('/api/v1/monitoring/client-instances', {'display_name': 'Real candidate smoke'}, session['csrf_token'])
            client_dir = base / 'client'
            client_dir.mkdir(mode=0o700)
            config = client_dir / 'config.json'
            template = json.loads((Path(__file__).resolve().parents[1] / 'config/xsoc.json.example').read_text())
            template.update(endpoint=origin + '/api/v1/xsoc/report', pairing_endpoint=origin + '/api/v1/xsoc/pairing-requests',
                            state_dir=str(client_dir / 'state'), tls_ca_pem=str(base / 'ca.pem'))
            # Disable privileged SMART access; CPU/memory/network are real host samples.
            template['smart'] = {'enabled': False}
            write_private(config, template)
            common = ['--config', str(config), '--format', 'json', '--non-interactive', '--timeout', '60s']
            run(str(binaries['client']), 'config', 'validate', '--file', str(config), *common)
            require(not (client_dir / 'state/client-token').exists(), 'Client was pre-paired')
            run(str(binaries['client']), 'pair', '--input-stdin', *common,
                input=json.dumps({'server': origin, 'authorization_code': instance['authorization_code']}).encode())
            binding = json.loads((client_dir / 'state/active-binding.json').read_text())
            host_id = (client_dir / 'state/host-id').read_text().strip()
            require(binding['report_endpoint'] == origin + '/api/v1/xsoc/report', 'Wrong paired destination')
            print('PASS fresh client pairing through trusted loopback TLS', flush=True)
            run(str(binaries['client']), 'once', *common)
            detail = api('/api/v1/monitoring/hosts/' + host_id)
            require(detail.get('latest') is not None, 'Real once report was not persisted')
            require(detail['latest']['host']['id'] == host_id, 'Report belongs to a different client')
            require(detail['latest']['system']['cpu']['logical_count'] > 0, 'No real CPU sample')
            require(int(detail['latest']['system']['memory']['total_bytes']) > 0, 'No real memory sample')
            date = api('/api/v1/monitoring/logs/calendar')['today']
            reports_path = '/api/v1/monitoring/hosts/' + host_id + '/reports?date=' + date
            reports = api(reports_path)['reports']
            require(len(reports) == 1, 'Expected exactly one real collected report')
            report_id = reports[0]['report_id']
            require(report_id == detail['latest']['report_id'], 'Detail and log disagree about report identity')
            print('PASS real once collection, delivery and authenticated server readback', flush=True)
            stop()
            start()
            after_session = api('/api/v1/auth/session')
            require(after_session['authenticated'] and after_session['user_id'] == session['user_id'], 'Administrator session did not persist')
            after = api('/api/v1/monitoring/hosts/' + host_id)
            require(after['latest'] == detail['latest'], 'Collected report changed across server restart')
            persisted_reports = api(reports_path)['reports']
            require(any(item['report_id'] == report_id for item in persisted_reports), 'Report log did not persist')
            print('PASS server restart preserves the real report and administrator session', flush=True)
            stop()
            for name, binary in binaries.items():
                with binary.open('rb') as stream:
                    require(hashlib.file_digest(stream, 'sha256').hexdigest() == digests[name], 'Candidate changed during smoke')
                print(name + '_sha256=' + digests[name])
            print('BOUNDARY server provisioned by private schema fixture; no install/systemd claim')
        except Exception:
            if proxy:
                print('HTTP status receipts:', proxy.smoke_receipts, flush=True)
            raise
        finally:
            if server and server.poll() is None:
                server.terminate()
                try:
                    server.wait(timeout=15)
                except subprocess.TimeoutExpired:
                    server.kill()
                    server.wait(timeout=5)
            if proxy:
                proxy.shutdown()
                proxy.server_close()
            log.close()


if __name__ == '__main__':
    main()
