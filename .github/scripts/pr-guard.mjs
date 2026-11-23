#!/usr/bin/env node
// Guards for a pull request, used by the deciding workflow before anything is built.
// Plain node, no dependencies. Every value this script prints has passed a strict
// pattern, so the shell never sees free text from a pull request.
//
//   node pr-guard.mjs pr <pr.json> <files.json> <tree.json> <commits.json> <repo> <default branch>
//   node pr-guard.mjs step <steps.json> <step id> <target>
//
// Exit 0: prints one JSON object. Exit 1: prints {"refused": "<reason>"}. Exit 3: bad usage.
import { readFileSync } from 'node:fs';
import { pathToFileURL } from 'node:url';

export const FILE_RE = /^targets\/([a-z0-9_]{1,40})\/holder\.rs$/;
export const BRANCH_RE = /^step-([1-9][0-9]{0,8})$/;
export const MAX_BYTES = 256 * 1024;
const SHA_RE = /^[0-9a-f]{40}$/;
const LOGIN_RE = /^[A-Za-z0-9](?:[A-Za-z0-9-]{0,38})$/;

class Refused extends Error {}
const refuse = (why) => {
  throw new Refused(why);
};

// The pull request must replace exactly one record holder file, as a plain file, and
// change nothing else. Read from the API's own answers, never from a checkout.
export function checkPr(pr, files, tree, commits, repo, defaultBranch) {
  if (pr?.state !== 'open') refuse('the pull request is not open');
  if (pr.base?.repo?.full_name !== repo) refuse('the pull request is not against this repository');
  if (pr.base?.ref !== defaultBranch) refuse(`the pull request is not against ${defaultBranch}`);
  const head = pr.head?.sha;
  if (!SHA_RE.test(head ?? '')) refuse('the pull request has no head commit');
  const author = pr.user?.login;
  if (!LOGIN_RE.test(author ?? '')) refuse('the author is not a plain account name');
  const branch = BRANCH_RE.exec(pr.head?.ref ?? '');
  if (!branch) refuse('the branch must be named step-<number>, as the desk gave it');
  // One commit, named after the branch. The merge keeps that commit in history, so it must
  // hold the one change and nothing to read: no earlier commits, no message of its own.
  if (pr.commits !== 1 || !Array.isArray(commits) || commits.length !== 1) {
    refuse('the pull request must have exactly one commit');
  }
  if (commits[0].sha !== head) refuse('the commit list and the head commit disagree');
  if (commits[0].commit?.message !== pr.head.ref) refuse('the commit message must be the branch name');
  // changed_files is the whole count, so a long file list cannot hide a second file.
  if (pr.changed_files !== 1 || !Array.isArray(files) || files.length !== 1) {
    refuse('the pull request must change exactly one file');
  }
  const file = files[0];
  const path = FILE_RE.exec(typeof file.filename === 'string' ? file.filename : '');
  if (!path) refuse('the one file must be targets/<target>/holder.rs');
  if (file.status !== 'modified') refuse(`the file must be modified, not ${String(file.status).slice(0, 20)}`);
  if (!SHA_RE.test(file.sha ?? '')) refuse('the file list names no content hash');
  // 100644 is a plain file. A symbolic link is 120000, an executable 100755.
  if (tree?.truncated) refuse('the tree of the head commit is too large to read');
  const entries = (tree?.tree ?? []).filter((e) => e.path === file.filename);
  if (entries.length !== 1) refuse('the file is not in the tree of the head commit');
  const entry = entries[0];
  if (entry.type !== 'blob' || entry.mode !== '100644') refuse('the file must be a plain file');
  if (entry.sha !== file.sha) refuse('the file list and the tree disagree about the content');
  if (!Number.isInteger(entry.size) || entry.size < 1 || entry.size > MAX_BYTES) {
    refuse(`the file must be between 1 and ${MAX_BYTES} bytes`);
  }
  return { target: path[1], step: Number(branch[1]), head, blob: file.sha, author };
}

// The step must be open at the desk, and it must be a step for this target.
export function checkStep(list, step, target) {
  const steps = Array.isArray(list?.steps) ? list.steps : refuse('the desk sent no step list');
  const found = steps.find((s) => s.id === step);
  if (!found) refuse(`step-${step} is not open at the desk`);
  if (found.target !== target) refuse(`step-${step} is not a step for this target`);
  return { step, target };
}

const json = (path) => JSON.parse(readFileSync(path, 'utf8'));

function main(argv) {
  const [mode, ...rest] = argv;
  if (mode === 'pr' && rest.length === 6) {
    return checkPr(json(rest[0]), json(rest[1]), json(rest[2]), json(rest[3]), rest[4], rest[5]);
  }
  if (mode === 'step' && rest.length === 3) {
    return checkStep(json(rest[0]), Number(rest[1]), rest[2]);
  }
  console.error('usage: pr-guard.mjs pr <pr> <files> <tree> <commits> <repo> <branch> | step <steps> <id> <target>');
  process.exit(3);
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? '').href) {
  try {
    console.log(JSON.stringify(main(process.argv.slice(2))));
  } catch (error) {
    const reason = error instanceof Refused ? error.message : 'the pull request could not be read';
    console.log(JSON.stringify({ refused: reason }));
    process.exit(1);
  }
}
