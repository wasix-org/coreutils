#!/usr/bin/env python3
"""Check every exported command and exercise the WASIX-specific behavior."""
import os
from pathlib import Path
import subprocess
import tempfile
import tomllib

ROOT = Path(__file__).resolve().parents[1]
PACKAGE = ROOT / '.wasix/coreutils-1.0.27.webc'
WASMER = os.environ.get('WASMER_BIN', 'wasmer')

def run(command, *args, stdin=None, directory=None, packages=()):
    invocation = [WASMER, 'run', str(PACKAGE), '--entrypoint', command]
    if directory:
        invocation += ['--volume', str(directory) + ':/work', '--cwd', '/work']
    for package in packages:
        invocation += ['--use', package]
    return subprocess.run([*invocation, '--', *args], input=stdin,
                          text=True, capture_output=True, timeout=30)

def check(command, *args, stdout=None, code=0, **kwargs):
    result = run(command, *args, **kwargs)
    assert result.returncode == code, (command, args, result.returncode, result.stdout, result.stderr)
    if stdout is not None:
        assert result.stdout == stdout, (command, result.stdout, stdout)
    return result

manifest = tomllib.loads((ROOT / 'wasmer.toml').read_text())
commands = [c['name'] for c in manifest['command']]
compiled = check('coreutils', '--list').stdout.splitlines()
assert set(commands) == {'coreutils', *compiled}
assert {'tail', 'nohup', 'env'} <= set(compiled)
for command in commands:
    if command == '[':
        result = check(command, 'x', ']')
    elif command == 'test':
        result = check(command, 'x')
    else:
        result = check(command, '--help', code=1 if command == 'false' else 0)
    assert 'function/utility not found' not in result.stdout + result.stderr
    assert 'panicked at' not in result.stderr
print(f'PASS all {len(commands)} exported commands dispatch correctly', flush=True)

# License mounts must not prevent command installation under /usr/bin.
assert '0.13.0' in check('env', '/usr/bin/coreutils', '--version').stdout
assert 'uutils' in check('cat', '/opt/coreutils/licenses/coreutils-0.13.0/LICENSE').stdout
print('PASS package license mount and /usr/bin command dispatch', flush=True)

with tempfile.TemporaryDirectory() as tmp:
    directory = Path(tmp)
    (directory / 'lines').write_text('one\ntwo\nthree\n')
    check('tail', '-n', '2', '/work/lines', directory=directory, stdout='two\nthree\n')
    check('tail', '-c', '6', '/work/lines', directory=directory, stdout='three\n')
    check('tail', '-n', '+2', stdin='one\ntwo\nthree\n', stdout='two\nthree\n')
    check('tail', '-z', '-n', '1', stdin='one\0two\0', stdout='two\0')
    # The native CLI exposes its standard descriptors as a guest terminal,
    # even when the host harness captures them, so nohup writes nohup.out.
    check('nohup', 'cat', '/work/lines', directory=directory, stdout='')
    assert (directory / 'nohup.out').read_text() == 'one\ntwo\nthree\n'
    (directory / 'nohup.out').unlink()
    check('nohup', 'false', directory=directory, code=1)
    check('nohup', 'coreutils-command-does-not-exist', directory=directory, code=127)
    check('env', 'WASIX_TEST=value', 'printenv', 'WASIX_TEST', stdout='value\n')
    (directory / 'nohup.out').unlink()
    check('nohup', 'bash', '-c', 'kill -HUP $$; echo survived; exit 7',
          directory=directory, packages=('wasmer/bash@1.0.25',), stdout='', code=7)
    assert (directory / 'nohup.out').read_text() == 'survived\n'
    check('cp', '/work/lines', '/work/copied', directory=directory)
    check('cat', '/work/copied', directory=directory, stdout='one\ntwo\nthree\n')
    check('sort', stdin='z\na\nz\n', stdout='a\nz\nz\n')
    print('PASS tail, nohup execution/SIGHUP/exit status, env, copying and sorting', flush=True)

    # Rotate within WASIX so the guest inode table observes the rename.
    subprocess.run(['cargo', 'wasix', 'build', '--release', '--locked',
                    '--manifest-path', 'wasix/tests/Cargo.toml'], cwd=ROOT, check=True)
    fixture = ROOT / 'wasix/tests/target/wasm32-wasmer-wasi/release/test-tail-wasix.wasm'
    # Map the freshly built atom directly: --use can cache local WebCs by
    # package name/version when iterating on an unpublished release.
    result = subprocess.run([WASMER, 'run', str(fixture),
        '--map-command', 'tail=' + str(ROOT / '.wasix/coreutils.wasm'),
        '--volume', str(directory) + ':/work', '--cwd', '/work'],
        text=True, capture_output=True, timeout=30)
    assert result.returncode == 0, (result.stdout, result.stderr)
    assert 'PASS tail -F' in result.stdout
    print(result.stdout, end='', flush=True)
