---
name: land
description: >-
  Prepare the current thread's changes for review in the PopMax repository on
  GitHub: run the project's local gates, commit on a feature branch, push, and open
  a pull request against main. Stops there — the user reviews and merges, so this
  skill never merges. Invoke this only when the user has asked to land, open a PR,
  or press "Land Changes" — not for reviewing, preparing, or just running checks.
disable-model-invocation: true
metadata:
  delta-action: land
---

# Prepare a change for review on `main`

The user has already asked to land. Carry it through to an **open pull request**
and verify it exists. Do **not** ask whether they want a PR — that is the request
that invoked this skill.

**Never merge.** The user reviews and merges their own changes, and `main` is not
the agent's to write to. Stopping at an open PR is the successful outcome, not a
half-finished one. Stop early only for a genuine blocker: a failing gate, ambiguous
scope, or denied push access.

## What "landed" means

Success is an open pull request against `main` on `origin`, with the branch pushed
and the PR confirmed to exist. A local commit or a pushed branch that never got a
PR is **not** success. If you cannot finish, say plainly that the change did not
land and why.

Merging is explicitly out of scope. Do not run `gh pr merge`, do not push to
`main`, and do not create a `v*` tag.

## Repository facts

Verify these still hold rather than assuming; the repository is the source.

- **Target branch**: `main` (confirmed via
  `gh api repos/khaledtf19/popMax --jq .default_branch`).
- **Remotes**: `origin` is GitHub (`https://github.com/khaledtf19/popMax.git`);
  `local` is the user's own checkout. **Push to `origin` only** — a push to
  `local` succeeds without publishing anything.
- **History convention**: short-lived `feat/*` and `fix/*` branches, merged with
  true merge commits (`Merge pull request #N from ...`). The user merges, so no
  merge strategy is chosen here — just target `main` and match the branch naming.
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

## 2. Run the gates before opening the PR

From `AGENTS.md`. Run all three; a failure in any one blocks the PR.

```sh
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features
```

`cargo test` must report `0 failed`, and `cargo fmt --check` must be clean. For
clippy, compare against the pre-existing baseline rather than demanding zero:
several warnings already exist in `bangs.rs`, `fav.rs`, `search.rs`, `hotkey.rs`,
`launcher.rs`, `tray.rs`, and three in `scanner.rs`. Proceed if the change adds no
*new* warnings; fix or stop if it does.

Build the release binary only if the change plausibly affects packaging or
`build.rs` — `cargo build --release` is slow because GPUI compiles from source.

## 3. Branch and commit

Branch names follow the repo's convention: `feat/<kebab-case>` for features,
`fix/<kebab-case>` for fixes.

```sh
git switch -c fix/scanner-startup-cache
GIT_EDITOR=true git add <files this change touched>
GIT_EDITOR=true git commit -m "cache and parallelize app scan"
```

Commit messages are short and imperative per `AGENTS.md`
(e.g. `add themes, update list`), scoped to one logical change. Stage only files
this change actually touched — never `git add -A`, which would sweep in unrelated
modifications that happen to be sitting in the tree. If the tree contains unrelated
work, leave it uncommitted and tell the user what you left behind.

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

## 5. Verify the PR exists

Do not trust the `gh pr create` exit status alone.

```sh
gh pr view <number> --json state,url,headRefName,baseRefName
gh pr checks <number> 2>/dev/null || true
```

Confirm the PR is `OPEN`, targets `main`, and points at the branch you pushed.
Report the PR link so the user can review it. Leave the local branch checked out or
switch back to `main`; do not delete the local branch, because the change is not
merged yet. The remote branch stays regardless — the repository has
`delete_branch_on_merge: false`.

## 6. Stop

Report the PR and hand off. Merging, tagging, and releasing are the user's calls.

## Conflicts

Merging is not this skill's job, so a merge conflict is not a failure here — it is
the user's to resolve when they merge. If the user asks you to resolve one, **use
the `resolving-merge-conflicts` skill** rather than re-implementing it: that skill
is authoritative for this repository, and it is the only correct route to a
resolved conflict.

## Report the outcome

When running in a subthread with `report_subthread_status` available, report the
result there. Otherwise report in the conversation.

| Outcome | `status` | `title` | `description` |
| --- | --- | --- | --- |
| PR opened and verified | `success` | `PR opened` | `[PR #N](<pr-url>) · [abc1234](<commit-url>). Awaiting your review.` |
| A gate failed | `failure` | `Blocked by local checks` | `cargo test failed; no branch or PR created.` |
| Push denied | `failure` | `Push blocked` | `[abc1234](<commit-url>) committed locally; push access required.` |

Use only real, verified URLs and the short SHA actually pushed — omit any link you
have not confirmed exists. Use `success` only after confirming the PR is open
against `main`. Never report success for a merge: this skill does not merge.
