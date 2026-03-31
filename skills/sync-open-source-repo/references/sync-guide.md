# Sync Guide

## When This Skill Applies

Use this workflow when:

- an internal repo and a public repo both continue to evolve
- the public repo rejected a push because upstream changed
- the two repos have different docs, assets, or repo policies
- only part of an internal change should become public
- a user explicitly asks to “sync to the open-source repo” and you need to inspect diff first

## Safe Default Flow

1. Inspect current state in the internal repo.
   - `git status --short`
   - `git branch --show-current`
   - `git remote -v`
2. Inspect the public target without mutating the current branch.
   - `git ls-remote <public-url> refs/heads/<branch>`
   - compare public head with local `HEAD`
   - check `git rev-list --left-right --count A...B`
   - try `git merge-base A B`
3. If the public branch is not already available locally, fetch it to a dedicated remote ref.
4. Create a separate worktree rooted at the public branch.
   - do not reuse the internal repo working tree
   - create a named sync branch in the worktree
5. Build a narrow patch.
   - prefer one feature commit or a curated file list
   - if the patch includes README or docs, verify those files are actually meant for public exposure
6. Dry-run apply first.
   - use `git apply --check`
   - if it fails, reduce scope instead of forcing the patch
7. Apply and adapt.
   - sync code/config first
   - patch docs separately when the public repo’s wording or structure differs
8. Validate in the public worktree.
   - compile and test only the affected surface first
   - then run broader checks if the change is large enough
9. Commit and push to a public sync branch.
10. Report:
   - what changed
   - what was intentionally skipped
   - any pre-existing failures in the public repo

## Common Pitfalls

### 1. No merge base

Symptom:
- `git merge-base` returns nothing

Meaning:
- the public repo is not a normal descendant of the internal repo

What to do:
- stop thinking in terms of “push my branch there”
- use worktree + patch + selective porting
- avoid rebasing or merging histories unless the user explicitly wants repo unification

### 2. Push rejected on public `main`

Symptom:
- `! [rejected] main -> main (fetch first)`

Meaning:
- the public repo has upstream commits your local branch does not include

What to do:
- fetch public `main`
- compare divergence
- create a sync branch from public `main`
- port the intended changes there

### 3. Full patch applies too much

Symptom:
- diff includes private docs, handoffs, bug notes, runbooks, assets, or branding changes

What to do:
- rebuild the patch using only relevant files
- separate code/config sync from documentation sync
- skip internal-only artifacts by default

### 4. README or docs patch fails

Symptom:
- code patch applies, README patch does not

Meaning:
- public docs evolved independently

What to do:
- do not force-copy internal docs
- manually add only the public-safe explanation the feature needs
- keep wording aligned with the public repo’s tone and structure

### 5. Validation failure unrelated to sync

Symptom:
- tests fail in the public worktree outside the changed area

What to do:
- isolate whether the failure already existed on the public base branch
- report it clearly as pre-existing if confirmed
- do not silently “fix unrelated tests” unless the user asked for that too

## Public/Private Boundary Checklist

Usually safe to sync:

- feature code
- config schema changes
- sample config updates
- public-safe tests
- generic docs explaining public behavior

Usually not safe to sync by default:

- internal runbooks
- handoff docs
- current-plan docs
- bug writeups with internal context
- private keys, endpoints, IDs, or credentials
- private brand assets or marketing copy

## Worktree Notes

- Prefer one worktree per public sync target.
- Name sync branches after the feature, for example `sync/chat-scope-busy-guard`.
- Clean up temporary worktrees after the sync is merged or no longer needed.
- Keep the internal repo branch clean while working in the public worktree.

## Recommended Delivery Pattern

- Push to a public sync branch
- let the user inspect the diff
- open a PR or merge only after review

Avoid direct public `main` pushes unless:

- the user explicitly requests it
- the branch is confirmed fast-forward compatible
- the user accepts the risk of updating the public default branch directly
