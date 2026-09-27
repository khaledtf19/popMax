---
name: land
description: >-
  Land the current thread's changes into the PopMax repository on GitHub: run the
  project's local gates, commit on a feature branch, push, open a pull request,
  merge it into main with a merge commit, and verify the result. Invoke this only
  when the user has asked to land, merge, ship, or press "Land Changes" — not for
  reviewing, preparing, or just running checks.
disable-model-invocation: true
metadata:
  delta-action: land
---

# Land a change into `main`

The user has already asked to land. Carry it through to a merged state on `main`
and verify it arrived. Do **not** ask whether they want to merge — that is the
request that invoked this skill. Stop only for a genuine blocker: a failing gate,
ambiguous scope, denied push access, or a conflict that cannot be resolved
safely.

## What "landed" means

Success is the change present on `main` at `origin` via a merged pull request.
Preparing a commit, pushing a branch, or opening a PR is **not** success. If you
cannot finish, say plainly that the change did not land and why.

## Repository facts

Verify these still hold rather than assuming; the repository is the source.

- **Target branch**: `main` (confirmed via
  `gh api repos/khaledtf19/popMax --jq .default_branch`).
- **Remotes**: `origin` is GitHub (`https://github.com/khaledtf19/popMax.git`);
  `local` is the user's own checkout. **Push to `origin` only** — a push to
  `local` succeeds without publishing anything.
- **History convention**: short-lived `feat/*` and `fix/*` branches, merged with
  true merge commits (`Merge pull request #N from ...`). Use
  `gh pr merge --merge`, not squash or rebase.
- **No pull-request CI.** `.github/workflows/release.yml` triggers only on `v*`
  tags, so no check will run on a PR. Local gates below are the real gate; do not
  claim to have awaited CI, and do not treat a missing check as passing.
- **Do not tag.** Pushing a `v*` tag publishes a release. Landing does not
  release; leave tagging to the user.

## 1. See the current state

```sh
git branch --show-current
git --no-optional-locks status --short
git diff --stat
```

Landing usually starts from uncommitted work on `main` (this thread's own edits).
That is expected — carry it onto a new branch rather than committing to `main`.
If the tree is clean, stop and say there is nothing to land.

## 2. Run the gates before landing

From `AGENTS.md`. Run all three; a failure in any one blocks landing.

```sh
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features
```

`cargo test` must report `0 failed`, and `cargo fmt --check` must be clean. For
clippy, compare against the pre-existing baseline rather than demanding zero:
several warnings already exist in `bangs.rs`, `fav.rs`, `search.rs`, `hotkey.rs`,
`launcher.rs`, `tray.rs`, and three in `scanner.rs`. Land if the change adds no
*new* warnings; fix or stop if it does.

Build the release binary only if the change plausibly affects packaging or
`build.rs` — `cargo build --release` is slow because GPUI compiles from source.

## 3. Branch and commit

Branch names follow the repo's convention: `feat/<kebab-case>` for features,
`fix/<kebab-case>` for fixes.

```sh
git switch -c fix/scanner-startup-cache
GIT_EDITOR=true git add -A
GIT_EDITOR=true git commit -m "cache and parallelize app scan"
```

Commit messages are short and imperative per `AGENTS.md`
(e.g. `add themes, update list`), scoped to one logical change. Stage only files
this change actually touched — do not sweep in unrelated modifications.

## 4. Push and open the PR

```sh
git push -u origin <branch>
gh pr create --base main --head <branch> \
  --title "<imperative summary>" \
  --body "<what changed and why>"
```

There is no PR template and no required body in this repo, but still write a real
body: what changed, why, and how it was verified. State the measured result where
there is one. If the push is denied, stop and report that the change did not land.

## 5. Merge

```sh
gh pr merge <number> --merge
```

## 6. Verify it landed

Do not trust the merge command's exit status alone.

```sh
git fetch origin
git log --oneline -1 origin/main
gh pr view <number> --json state,mergeCommit
```

Confirm the PR state is `MERGED` and that `origin/main` contains the change.
Then delete the local branch (`git branch -d <branch>`); the remote branch is
left in place because the repository has `delete_branch_on_merge: false`.

## Conflicts

If `gh pr merge` conflicts, **use the `resolving-merge-conflicts` skill** — do not
re-implement it here. That skill is authoritative for this repository: find the
primary source of each side, preserve both intents, prefer the change matching the
merge's goal, never `--abort`, and re-run the gates afterwards.

Resolve automatically when the intended result is clear. Stop and ask only when
the conflict is genuinely ambiguous or resolving it would mean inventing
behaviour.

## Report the outcome

When running in a subthread with `report_subthread_status` available, report the
result there. Otherwise report in the conversation.

| Outcome | `status` | `title` | `description` |
| --- | --- | --- | --- |
| Merged and verified | `success` | `Landed on main` | `[abc1234](<commit-url>) · [PR #N](<pr-url>).` |
| A gate failed | `failure` | `Blocked by local checks` | `cargo test failed on [branch](<pr-url>). Not landed.` |
| Push denied | `failure` | `Push blocked` | `[abc1234](<commit-url>) ready; push access required.` |
| Unresolved conflict | `failure` | `Merge conflicts` | `[branch](<pr-url>) conflicts with main.` |

Use only real, verified URLs and the short SHA actually merged — omit any link
you have not confirmed exists. Use `success` only after confirming the change
reached `origin/main`.
