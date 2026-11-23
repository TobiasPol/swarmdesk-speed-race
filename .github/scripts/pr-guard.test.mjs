import assert from 'node:assert/strict';
import { test } from 'node:test';
import { checkPr, checkStep } from './pr-guard.mjs';

const SHA = 'a'.repeat(40);
const BLOB = 'b'.repeat(40);
const REPO = 'owner/race';

const good = () => ({
  pr: {
    state: 'open',
    base: { repo: { full_name: REPO }, ref: 'main' },
    head: { sha: SHA, ref: 'step-12' },
    user: { login: 'ada' },
    changed_files: 1,
  },
  files: [{ filename: 'targets/crc32/holder.rs', status: 'modified', sha: BLOB }],
  tree: { tree: [{ path: 'targets/crc32/holder.rs', type: 'blob', mode: '100644', sha: BLOB, size: 900 }] },
});

const run = (change) => {
  const input = good();
  change?.(input);
  return checkPr(input.pr, input.files, input.tree, REPO, 'main');
};

test('a pull request that replaces one record holder passes', () => {
  assert.deepEqual(run(), { target: 'crc32', step: 12, head: SHA, blob: BLOB, author: 'ada' });
});

test('refusals', async (t) => {
  const cases = [
    ['closed', (i) => (i.pr.state = 'closed'), /not open/],
    ['other repository', (i) => (i.pr.base.repo.full_name = 'evil/race'), /not against this repository/],
    ['other base branch', (i) => (i.pr.base.ref = 'dev'), /not against main/],
    ['branch name', (i) => (i.pr.head.ref = 'my-branch'), /step-<number>/],
    ['branch name with text', (i) => (i.pr.head.ref = 'step-12; rm -rf /'), /step-<number>/],
    ['two files', (i) => (i.pr.changed_files = 2), /exactly one file/],
    ['hidden second file', (i) => i.files.push({ ...i.files[0] }), /exactly one file/],
    ['workflow file', (i) => (i.files[0].filename = '.github/workflows/deciding.yml'), /holder\.rs/],
    ['harness file', (i) => (i.files[0].filename = 'harness/src/judge.rs'), /holder\.rs/],
    ['path climbing', (i) => (i.files[0].filename = 'targets/../x/holder.rs'), /holder\.rs/],
    ['added file', (i) => (i.files[0].status = 'added'), /must be modified/],
    ['symbolic link', (i) => (i.tree.tree[0].mode = '120000'), /plain file/],
    ['executable', (i) => (i.tree.tree[0].mode = '100755'), /plain file/],
    ['too large', (i) => (i.tree.tree[0].size = 300 * 1024), /between 1 and/],
    ['content mismatch', (i) => (i.tree.tree[0].sha = 'c'.repeat(40)), /disagree/],
    ['odd author', (i) => (i.pr.user.login = 'ada[bot]'), /plain account name/],
    ['truncated tree', (i) => (i.tree.truncated = true), /too large to read/],
  ];
  for (const [name, change, pattern] of cases) {
    await t.test(name, () => assert.throws(() => run(change), pattern));
  }
});

test('the step must be open and belong to the target', () => {
  const list = { steps: [{ id: 12, target: 'crc32' }, { id: 13, target: 'sha256' }] };
  assert.deepEqual(checkStep(list, 12, 'crc32'), { step: 12, target: 'crc32' });
  assert.throws(() => checkStep(list, 99, 'crc32'), /not open at the desk/);
  assert.throws(() => checkStep(list, 13, 'crc32'), /not a step for this target/);
  assert.throws(() => checkStep({}, 12, 'crc32'), /no step list/);
});
