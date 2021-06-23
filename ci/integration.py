#!/usr/bin/env python3
import subprocess
from subprocess import DEVNULL, PIPE
import shutil
from pathlib import Path
import tempfile
import json
import sys


def _sn0int(tempdir, binary, args, piped_stdout=False):
    return subprocess.Popen(
        ['/usr/bin/env', 'HOME='+tempdir, binary] + args,
        stdin=PIPE,
        stdout=PIPE if piped_stdout else None,
    )


def sn0int(tempdir, binary, cmds):
    p = _sn0int(tempdir, binary, [])
    for cmd in cmds:
        p.stdin.write((cmd + '\n').encode('utf-8'))
    p.communicate()
    if p.returncode != 0:
        raise Exception('process failed')


def sn0int_select(tempdir, binary, query):
    p = _sn0int(tempdir, binary, ['select', '--json'] + query, piped_stdout=True)
    stdout, _ = p.communicate()
    lines = filter(None, stdout.decode('utf-8').split('\n'))
    return [json.loads(x) for x in lines]


def main(tempdir, binary):
    print('[*] setting up workspace')
    sn0int(tempdir, binary, [])

    print('[*] adding domain')
    sn0int(tempdir, binary, [
        'add domain',
        'example.com',
        'select domains',
    ])

    print('[*] testing db for domain')
    domains = sn0int_select(tempdir, binary, ['domains'])
    assert domains == [{'id': 1, 'value': 'example.com', 'unscoped': False}]

    print('[*] installing modules')
    sn0int(tempdir, binary, [
        'pkg install kpcyrd/ctlogs',
        'pkg install kpcyrd/dns-resolve',
        'pkg install kpcyrd/url-scan',
        'pkg install kpcyrd/geoip',
    ])

    print('[*] running ctlogs')
    sn0int(tempdir, binary, [
        'use ctlogs',
        'run',
        'select subdomains',
    ])

    print('[*] testing db for subdomains')
    subdomains = sn0int_select(tempdir, binary, ['subdomains'])
    assert {x['value'] for x in subdomains} == {
        'example.com',
        'www.example.com',
        'm.example.com',
        'dev.example.com',
        'products.example.com',
        'support.example.com',
    }

    print('[*] running dns-resolve')
    sn0int(tempdir, binary, [
        'use dns-resolve',
        'run',
        'select ipaddrs',
    ])

    print('[*] testing db for ipaddrs')
    ipaddrs = sn0int_select(tempdir, binary, ['ipaddrs'])
    assert len(ipaddrs) >= 1

    print('[*] running url-scan')
    sn0int(tempdir, binary, [
        'use url-scan',
        'run',
        'select urls',
    ])

    print('[*] testing db for urls')
    urls = sn0int_select(tempdir, binary, ['urls'])
    assert {(x['value'], x['status']) for x in urls} == {
        ('http://example.com/', 200),
        ('https://example.com/', 200),
        ('http://www.example.com/', 200),
        ('https://www.example.com/', 200),
    }

    cache = Path.home() / '.cache' / 'sn0int'
    if cache.exists():
        print('[*] copying geoip files')
        shutil.copytree(cache, tempdir + '/.cache/sn0int', dirs_exist_ok=True)

    print('[*] running geoip')
    sn0int(tempdir, binary, [
        'use geoip',
        'run',
        'select ipaddrs',
    ])

    print('[*] testing db for ipaddrs again')
    ipaddrs2 = sn0int_select(tempdir, binary, ['ipaddrs'])
    assert ipaddrs != ipaddrs2

    print('')
    print('\t###########')
    print('\t# SUCCESS #')
    print('\t###########')
    print('')


if __name__ == '__main__':
    try:
        binary = sys.argv[1]
    except IndexError:
        print('Usage: %s target/release/sn0int' % sys.argv[0])
    else:
        with tempfile.TemporaryDirectory(prefix='sn0int-') as tempdir:
            main(tempdir, binary)
