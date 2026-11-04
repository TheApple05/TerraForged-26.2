# Explicit errors at the Horizon density boundary

The native graph adapter needs failures to reach its caller. The isolated host
patch adds `SampleError::External(&'static str)` while preserving `Clone`, `Copy`
and equality. Its unit test covers scalar, bulk and compiled-graph propagation,
then successful evaluation after resetting the context.

The patch is in the user's own Horizon repository:
[PR 2](https://github.com/TheApple05/horizon/pull/2), commit
`e4d7699a8d4ff5b18a966601b9c3e8346f4571e5`. It changes two tracked files by
51 lines. No Minecraft resources or generated game artifacts are included.
The fork's complete `checks` workflow subsequently passed on that exact head.
PR 2 merged as `104335aa84905450c38f6d17e9490538172522cf`.

Windows `tools/pr_ready.py --quick --base 21679eb68f770d3ac1c2cc1c51a535eea37787ab`
passed public-tree checks, formatting, workspace Clippy with warnings denied,
132 touched Rust tests and zero documentation tests. A separate core-library run
passed 124 tests with three existing ignored tests.

The full command did not pass: 631 Python tests ran, with one failure and 13
skipped. With the stock Windows PATH, the shell parser test selected the default
Docker Desktop WSL distribution, which lacks `/bin/bash`. Putting Git Bash first
fixed that test but exposed a launcher-fallback platform simulation failure.
The launcher failure also occurs on the unchanged baseline. Both identified
tests pass when run under WSL Ubuntu; that is targeted coverage, not a claim that
the complete Python suite passed under WSL.

The second `pr_ready` scope count included temporary untracked Cargo output.
That directory was subsequently moved outside the worktree. The Git diff and
published commit contain only the two intended source files. Failed checks and
their logs remain recorded; no test was disabled or modified to obtain a pass.

The draft for these notes was delegated to DeepSeek and reviewed against the
local results. This patch alone does not activate procedural TerraForged worlds.
