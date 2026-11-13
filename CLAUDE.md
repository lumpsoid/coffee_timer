# Coffee Timer

Runs multi-step coffee brewing recipes: a per-step countdown with a sound and
vibration at each step boundary, screen kept awake while brewing. Recipes and
settings are stored locally.

<!-- agent-kit:rules begin. Written by /repo-rules from agent-kit/conventions/agent-rules.md; edit it there, not here. Put repo-specific exceptions under "## Repo overrides" outside this block. -->
## Agent rules

These rules apply to every agent working in this repo. A rule under
**Repo overrides** elsewhere in this file wins over this block.

### Code

Follow @CODE-PRINCIPLES.md. Project-specific rules (stack, layout, gates)
live in this file, outside this block.

### Commits

[Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/):
`<type>(<scope>): <subject>`.

- **type**: `feat`, `fix`, `refactor`, `perf`, `test`, `docs`, `build`, `ci`,
  `chore`, `revert`.
- **scope**: the module or feature touched, lower-case. Leave it out when the
  change has no single home.
- **subject**: imperative, lower-case, no trailing period, 72 characters at
  most.
- **body**: why, not what. Rationale, design decisions and gotchas go here,
  where they travel with the code.
- **breaking**: `!` after the scope and a `BREAKING CHANGE: <what>` footer.
- One logical change per commit, with its gate green.

### Branches

- Never commit to `main` directly. Start each piece of work on a new
  branch cut from an up-to-date `main`, named `<type>/<slug>`
  (`feat/offline-sync`, `fix/empty-queue`).
- To land, rebase first, re-run the full gate on the rebased branch, then
  merge with a merge commit:

  ```sh
  git rebase main
  # full gate, green
  git switch main && git merge --no-ff <branch> && git branch -d <branch>
  ```

- No squash merges, no fast-forward merges. Never push or force-push unless
  asked.

### Tasks

Tasks live in the `agent-plan` store, outside the repo, so they can't be
committed by mistake. Never put a plan, task list or handoff in the repo.

- A track of work is one **meta task** `<track>-meta`, plus one **task** per
  unit of work, `<track>-<NN>-<slug>` (`sync-03-conflict-ui`).
- `agent-plan <name>` prints a plan's path, `agent-plan --list --name-prefix
  <track>-` lists a track, and `agent-plan <name> --append` adds stdin to the
  end of a plan.
- Read the meta task first. It is an index: open a task only when its
  "Open it for:" line says it holds what you need.
- **Starting a task:** set its Status to `in progress` and point the meta's
  **Active task** at it.
- **Finishing a task:** tick its acceptance criteria, add a Log line with
  the commit shas, set Status to `done (<date>)`. In the meta, update its
  Tasks entry with a one-line result and "Open it for:", copy leftovers into
  **Follow-ups** tagged `(<NN>)`, and move **Active task** on.
- A decision that crosses tasks goes in the meta's **Cross-task decisions**.
  A decision local to one task stays in that task.
- Durable facts (rationale, gotchas, API shape) go in commit bodies or this
  file, not only in a plan. When the track is finished, archive every task
  and then the meta: `agent-plan --archive <name>`.

Meta task (`<track>-meta`):

```markdown
# Meta: <track title>

**Context:** <why this track exists, in a paragraph>

Index only. Each task lives in its own plan (`agent-plan <name>`).
Only decisions that cross tasks are kept here.

**Active task:** <NN-slug, or none and why>

## References
- <design plan, fixture, external repo, doc> - <path or plan name>

## Cross-task decisions
- <decision> - <why> (<who decided, date> if the user did)

## Tasks
1. `<track>-01-<slug>` - **todo | in progress | done (<date>) | deferred (<why>)**.
   <one-line result once done>
   Open it for: <what only that plan holds>

## Not planned yet
<work the track will need later; add a task when it starts>

## Follow-ups
Open items left by finished tasks. Each names its source task.
- (<NN>) <item>
```

Task (`<track>-<NN>-<slug>`):

```markdown
# Task <NN>: <title>
Meta: `<track>-meta` · Status: **todo | in progress | done (<date>)** · Commits: <shas>

## Why
<what made this task necessary: the finding, bug or decision behind it>

## Goal
<the outcome, not steps>

## Decided
- <decision local to this task> - <why>

## Open (decide first)
- <question to settle before coding; move it to Decided once answered>

## Acceptance criteria
- [ ] <a concrete check: a test, a command and its output, an observed behaviour>

## Non-goals
<what this task leaves to which other task>

## Log
- <date>: <sha> <subject>; <what was learned>
```
<!-- agent-kit:rules end -->

## Stack

- **Rust** — all business logic and state.
- **Flutter** — GUI and platform effects only. No business logic in Dart.
- **[rinf](https://github.com/cunarist/rinf)** — the Rust↔Flutter bridge.
  Layering and signal rules: [rinf bridge](./docs/rinf-bridge.md).

## Architectural commitments

1. Rust owns state, in feature controllers that each own one state object;
   shared data lives in a shared controller passed into actions. Rust sends
   view state to Flutter as `RustSignal`s (whole state per feature; diffs
   for the recipe list); Dart replaces its view state from them. Dart sends
   user intents as `DartSignal`s.
2. The core is usable without Flutter: exercised by `cargo test` and a CLI.
   If the CLI cannot do something without Flutter, logic leaked out of core.
3. Behavior carried over from the legacy app (step bounds, brew metrics,
   labels, timer behavior) is fixed; change it only deliberately. The legacy
   bugs fixed during the port are listed in the port plan; don't reintroduce
   them.
4. No panics in Rust (`unwrap`/`expect`); return `Result`.

## Gates

- `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`
  and `cargo test`
- `rinf gen` after changing any signal struct, and on a fresh checkout
  before `flutter analyze` (the generated bindings are gitignored)
- `flutter analyze` / `flutter test` for the GUI
- `scripts/check.sh` runs them all, as CI does

## Behavior reference

The port is complete; the code is the specification.

- [docs/README.md](./docs/README.md) — what the app does, build/run, gates, CLI.
- [docs/architecture.md](./docs/architecture.md) — controllers, catalog-change
  flow, timer, handshake, startup, error surfacing.
- [docs/modules.md](./docs/modules.md) — module-by-module guide.
- [docs/port-plan.md](./docs/port-plan.md) — record of port decisions and of
  every legacy bug fixed.
