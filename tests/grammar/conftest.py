"""Pytest hooks for the KCL grammar test suite.

Some grammar fixtures (the ones carrying a ``git_setup.yaml`` file)
need a git repository around them: the ``file.read`` /
``file.readbase64`` ``path:ref`` syntax resolves revisions through
the enclosing git repository.  The grammar runner itself only
executes ``libkcl run main.k`` inside the fixture directory, so this
conftest prepares the fixture-side git repositories before the
session and removes them afterwards.

The suite runs in parallel (``pytest -n 5``), where every worker is a
separate process, so setup is guarded by per-fixture lock files and a
cross-process worker counter: the repositories are torn down only
after the *last* worker has finished, and a crashed session is
detected through stale counter/setup files.

A ``git_setup.yaml`` fixture spec looks like::

    file: asset.txt            # single relative file name, committed in the fixture repo
    revisions:
      - content: |             # one commit per entry, in order
          version one
        tag: v1.0              # optional tag on that commit
      - content: |
          version two
    worktree: |                # optional, left on disk uncommitted after the last commit
      version three (uncommitted)

The committed asset file is generated (and git-ignored) so that a
plain read (from disk), a ``:HEAD`` read and a tag-pinned read can
all yield different contents within one fixture.
"""

import hashlib
import os
import pathlib
import shutil
import subprocess
import tempfile
import time

import pytest
from ruamel.yaml import YAML

GRAMMAR_ROOT = pathlib.Path(__file__).parent
SETUP_SPEC = "git_setup.yaml"
LOCK_FILE = ".git_setup.lock"
DONE_FILE = ".git_setup.done"

GIT_IDENTITY = {
    "GIT_AUTHOR_NAME": "kcl-grammar-test",
    "GIT_AUTHOR_EMAIL": "kcl-grammar-test@example.com",
    "GIT_COMMITTER_NAME": "kcl-grammar-test",
    "GIT_COMMITTER_EMAIL": "kcl-grammar-test@example.com",
}

# Fixed commit timestamps keep the fixture repositories reproducible.
_COMMIT_DATE = "2024-01-01T00:00:{:02d} +0000"
_SETUP_TIMEOUT = 120.0
# A worker-counter file older than this is assumed to be left over
# from a crashed session and is reset.
_COUNTER_STALE_AFTER = 3600.0


class _FileLock:
    """Exclusive lock via ``O_CREAT | O_EXCL`` with a timeout."""

    def __init__(self, path, timeout=_SETUP_TIMEOUT):
        self._path = path
        self._timeout = timeout

    def __enter__(self):
        deadline = time.monotonic() + self._timeout
        while True:
            try:
                fd = os.open(self._path, os.O_CREAT | os.O_EXCL | os.O_WRONLY)
                os.close(fd)
                return self
            except FileExistsError:
                if time.monotonic() > deadline:
                    raise RuntimeError(f"timed out waiting for {self._path}")
                time.sleep(0.05)

    def __exit__(self, *exc):
        try:
            os.remove(self._path)
        except OSError:
            pass


def _counter_dir():
    key = hashlib.sha256(str(GRAMMAR_ROOT.resolve()).encode()).hexdigest()[:16]
    path = pathlib.Path(tempfile.gettempdir()) / f"kcl-grammar-git-fixtures-{key}"
    path.mkdir(parents=True, exist_ok=True)
    return path


def _worker_enter(counter_dir):
    counter = counter_dir / "workers"
    with _FileLock(counter_dir / "workers.lock", timeout=30.0):
        count = 0
        if counter.exists():
            if time.time() - counter.stat().st_mtime > _COUNTER_STALE_AFTER:
                counter.unlink()
            else:
                count = int(counter.read_text().strip() or 0)
        counter.write_text(str(count + 1))


def _worker_exit(counter_dir):
    counter = counter_dir / "workers"
    with _FileLock(counter_dir / "workers.lock", timeout=30.0):
        count = int(counter.read_text().strip()) if counter.exists() else 1
        count -= 1
        if count <= 0:
            counter.unlink(missing_ok=True)
            return True
        counter.write_text(str(count))
        return False


def _git(fixture, args, env=None):
    environ = dict(os.environ)
    environ.update(GIT_IDENTITY)
    if env:
        environ.update(env)
    process = subprocess.run(
        ["git", *args],
        cwd=fixture,
        env=environ,
        capture_output=True,
        text=True,
    )
    if process.returncode != 0:
        raise RuntimeError(
            f"git {' '.join(args)} failed in {fixture}:\n{process.stderr}"
        )


def _load_spec(fixture):
    return YAML(typ="safe").load((fixture / SETUP_SPEC).read_text()) or {}


def _setup_fixture(fixture):
    spec = _load_spec(fixture)
    rel = spec.get("file")
    revisions = spec.get("revisions")
    if (
        not rel
        or pathlib.PurePosixPath(rel).is_absolute()
        or len(pathlib.PurePosixPath(rel).parts) != 1
    ):
        raise ValueError(f"{fixture}/{SETUP_SPEC}: 'file' must be a single relative name")
    if not revisions:
        raise ValueError(f"{fixture}/{SETUP_SPEC}: 'revisions' must be a non-empty list")

    # Remove leftovers from a previously crashed session.
    shutil.rmtree(fixture / ".git", ignore_errors=True)
    (fixture / rel).unlink(missing_ok=True)

    _git(fixture, ["init", "-q", "-b", "main"])
    for index, revision in enumerate(revisions):
        (fixture / rel).write_text(revision.get("content", ""))
        _git(fixture, ["add", "-f", rel])
        _git(
            fixture,
            ["commit", "-q", "-m", f"revision {index + 1}"],
            env={
                "GIT_AUTHOR_DATE": _COMMIT_DATE.format(index),
                "GIT_COMMITTER_DATE": _COMMIT_DATE.format(index),
            },
        )
        if revision.get("tag"):
            _git(fixture, ["tag", str(revision["tag"])])
    if spec.get("worktree") is not None:
        (fixture / rel).write_text(spec["worktree"])


def _setup_done(fixture):
    return (fixture / DONE_FILE).exists() and (fixture / ".git").exists()


def _ensure_setup(fixture):
    if _setup_done(fixture):
        return
    lock = fixture / LOCK_FILE
    try:
        fd = os.open(lock, os.O_CREAT | os.O_EXCL | os.O_WRONLY)
        os.close(fd)
    except FileExistsError:
        # Another worker is setting this fixture up; wait for its sentinel.
        deadline = time.monotonic() + _SETUP_TIMEOUT
        while not _setup_done(fixture):
            if time.monotonic() > deadline:
                raise RuntimeError(f"timed out setting up git fixture {fixture}")
            time.sleep(0.05)
        return
    try:
        # Re-check under the lock in case the holder just finished.
        if not _setup_done(fixture):
            _setup_fixture(fixture)
            (fixture / DONE_FILE).write_text("ok\n")
    finally:
        lock.unlink(missing_ok=True)


def _teardown_fixture(fixture):
    try:
        with _FileLock(fixture / LOCK_FILE, timeout=30.0):
            shutil.rmtree(fixture / ".git", ignore_errors=True)
            for name in (LOCK_FILE, DONE_FILE):
                (fixture / name).unlink(missing_ok=True)
    except OSError:
        pass


@pytest.fixture(scope="session", autouse=True)
def grammar_git_fixtures():
    specs = sorted(GRAMMAR_ROOT.rglob(SETUP_SPEC))
    if not specs:
        yield
        return
    counter_dir = _counter_dir()
    _worker_enter(counter_dir)
    try:
        for spec in specs:
            _ensure_setup(spec.parent)
        yield
    finally:
        if _worker_exit(counter_dir):
            # Last worker out removes every fixture repository so the
            # enclosing checkout is left untouched.
            for spec in specs:
                _teardown_fixture(spec.parent)
            shutil.rmtree(counter_dir, ignore_errors=True)
