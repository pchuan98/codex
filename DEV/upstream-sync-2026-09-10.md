# Upstream sync: 2026-09-10

## Result

- Previous dev: `896fabdf1`; actual upstream parent: `e1eb98461` (2026-09-07).
- Updated main: `ea53c8d4f` (2026-09-10); rebased fork: `962a1ff4f`.
- 206 upstream commits introduced into dev. Local main was stale at `2230d6446`; its 1,416-commit advance is not the number of new commits in dev.
- Backup: `backup/dev-before-upstream-20260910`. No push performed.
- Upstream diff: 1,281 files, 103,423 insertions and 24,413 deletions.

## Behavior changes

| Area | Change | Representative PRs |
| --- | --- | --- |
| Remote compaction | Streamed Responses compaction replaces the legacy compact endpoint; CPA compatibility needs verification. | #44255, #44273 |
| Models | Default-off API-key model discovery; model caches scoped to provider/auth identity. | #44392, #43906 |
| Voice | Experimental, default-off voice conversations; dedicated controls, configurable mute and runtime packaging fixes. | #44331, #43683, #43690 |
| Memory | Opt-in v2 with isolated storage, extraction/consolidation prompts and dual-writing support. | #43797, #43813, #43827 |
| TUI | Streaming reasoning summaries, transcript fixes, agents navigation and worktree owner/deletion UI. | #43921, #43889, #44360, #43942 |
| Threads | Persist/recover daemon threads and add coordinated attachment operations. | #44283, #44314, #44350 |
| MCP | OAuth failure reporting, reconnect/cancellation fixes and native user verification. | #44359, #44238, #44346 |
| Execution | Credential providers, Windows/WSL sandbox fixes; remove Windows sandbox-add-read-dir command. | #44056, #44327, #44286, #44259 |
| Guardian | Dedicated policy crate and consolidated review orchestration with input budgets. | #44227, #44252, #44281 |
| SDK | Python history/per-turn options, external messages and publishing improvements. | #44084, #44086, #44067 |

## Conflict resolution and fork review

Only `codex-rs/tui/src/chatwidget/streaming.rs` conflicted. Kept upstream reasoning accumulation/activity wording and restored `record_custom_status_line_delta(&delta)` before accumulation.

The one-commit range-diff shows only changed upstream context in streaming and turn_runtime. The custom patch remains otherwise equivalent. Automatic application is not runtime validation.

- Re-review CPA remote compaction first: the previous endpoint compatibility claim no longer applies. Updated `DEV/cpa.md` accordingly.
- Re-review composer/footer integration with the new voice strip and reasoning activity display; custom status line patch retained.
- Resume provider filter and SSE rate-limit selection source files were unchanged upstream in this interval; their fork patches remain intact.
- Telemetry initialization patch retained; upstream adds terminal/multiplexer metric labels. No new public SQLite tracing disable switch was established by this scoped review.
- Model discovery is opt-in; CPA routing/cache behavior should be reviewed before enabling it.

## Validation

- `git diff --check main HEAD`: passed after rebase.
- `git range-diff 896fabdf1^..896fabdf1 main..HEAD`: reviewed; differences limited to upstream context described above.
- `git merge-base --is-ancestor main HEAD`: passed.
- `git ls-files -u`: empty after rebase.
- `git status --porcelain`: clean after rebase, before writing this report and the CPA documentation correction.
- No tests, formatters, snapshots or just commands executed. No local test/snapshot changes authored; upstream versions were brought in by rebase.
- No build or live CPA request executed. Compilation and runtime compatibility remain unverified.

## Complete introduced commit list

Grouped by subject keywords for navigation; original English commit subjects are preserved. Each commit appears exactly once. Functional availability may depend on feature flags, platform and service support.

### Authentication and execution (22)

- 2026-09-09 [1bff94edb](https://github.com/openai/codex/commit/1bff94edb6800974c3eed77e4716ba66804ed619) Bind remote-control sessions to their authentication owner (#44341)
- 2026-09-09 [f11d0dd01](https://github.com/openai/codex/commit/f11d0dd0120ed936629acb57385090e5317e3903) Prevent filesystem-root read denies in the Windows sandbox (#44327)
- 2026-09-09 [f71543813](https://github.com/openai/codex/commit/f71543813fc0fbf652a6efd86f65c8d723eef7db) Block WSL interop escapes from restricted filesystem sandboxes (#44286)
- 2026-09-09 [a3ba42b01](https://github.com/openai/codex/commit/a3ba42b0108db0bbff97e97f97b3fae8ea3e973e) Remove the Windows `/sandbox-add-read-dir` slash command (#44259)
- 2026-09-09 [ed4ca07ba](https://github.com/openai/codex/commit/ed4ca07ba68595b41acfc304ab987f348c1c4e98) Handle credential provider source remapping across config layers (#44241)
- 2026-09-09 [634ebc186](https://github.com/openai/codex/commit/634ebc1865c6ac840ed3ba118f040d527bf4b55d) Support credential brokering in plaintext HTTP tunnels (#44089)
- 2026-09-09 [56d3e8192](https://github.com/openai/codex/commit/56d3e8192f9dbe8741f739e2538c9d69c661480a) Refactor credential-broker tunnel protocol detection (#44077)
- 2026-09-09 [38cbebaf3](https://github.com/openai/codex/commit/38cbebaf3fe3e81a94bf462079e7cf9659fc9e50) Support configured credential providers across shell snapshots (#44072)
- 2026-09-09 [f45115a13](https://github.com/openai/codex/commit/f45115a13746bca56274551136cb9bc6dbcbf203) Preserve credential broker destinations across environment filtering (#44068)
- 2026-09-09 [5a9aec40a](https://github.com/openai/codex/commit/5a9aec40a5e893bbcf22dc1301b9f547fb356889) Extend configured credential brokerage to embedded aliases (#44066)
- 2026-09-09 [1bfd38389](https://github.com/openai/codex/commit/1bfd383890b1bd7fab0d1b3d5e05129a9e293d8a) Add configurable credential providers to the network proxy (#44056)
- 2026-09-09 [9ba1d9eb5](https://github.com/openai/codex/commit/9ba1d9eb5bbbd87ba2fc528d91ad239eea975ee9) Extract credential broker environment and registry helpers (#44049)
- 2026-09-09 [ec512d234](https://github.com/openai/codex/commit/ec512d23470c6afe30c975d570d181f34813b9a2) Harden credential handling in shell snapshots and replay (#44040)
- 2026-09-09 [a548463b7](https://github.com/openai/codex/commit/a548463b78773802b00b0a226b89ff48876a4e1e) Handle copied credentials in the broker and shell snapshots (#44038)
- 2026-09-09 [fe52d795c](https://github.com/openai/codex/commit/fe52d795c95321429c24f64be684856557491b47) Add AWS credential export commands for Amazon Bedrock (#44028)
- 2026-09-08 [c53f342fe](https://github.com/openai/codex/commit/c53f342fecfb00a8d068024516f6483da8c852b4) Add executor-context filesystem permission helpers (#43939)
- 2026-09-08 [5e3f0ee94](https://github.com/openai/codex/commit/5e3f0ee94b0719ab3d0d05cffaa75163e87668f6) Avoid Windows sandbox setup for irrelevant proxy port changes (#43930)
- 2026-09-08 [900b1e4ce](https://github.com/openai/codex/commit/900b1e4cec1c4eada106a8071fc89c11c50de9a6) Add tracing for project instructions and filesystem sandbox operations (#43913)
- 2026-09-08 [2e220af1f](https://github.com/openai/codex/commit/2e220af1f63e0dc8cd0282a283de9ec2ce5dfeb9) Protect shell snapshots when credential brokerage is enabled (#43909)
- 2026-09-08 [5a65fd87d](https://github.com/openai/codex/commit/5a65fd87d84215b24ea79100f8d47238f578a5f2) Close active network proxy connections on teardown (#43884)
- 2026-09-08 [ce254df05](https://github.com/openai/codex/commit/ce254df05a3162a93d8f3357ff4dd86582c534b7) Add canonical permission translation for MXC execution requests (#43853)
- 2026-09-08 [b090e901f](https://github.com/openai/codex/commit/b090e901f8702f63a5fedbcdcf3f4d021d10a8a0) Add staged enterprise OIDC login and coordinated logout (#43844)

### Build and other changes (31)

- 2026-09-09 [d390f0a09](https://github.com/openai/codex/commit/d390f0a09cac7834d54b723c1f3ffcdf9641f31e) Return to the agent command center after archiving on shared servers (#44337)
- 2026-09-09 [0df6366a8](https://github.com/openai/codex/commit/0df6366a87dbadf5376cdd26b7675935ef76f893) Add bounded tool-result metadata support to executed tool calls (#44336)
- 2026-09-09 [0adfc1f2f](https://github.com/openai/codex/commit/0adfc1f2f2022b8c6fbc6526b953c4dd1865f7fa) Return the prompt hash in upload responses (#44325)
- 2026-09-09 [0735c5197](https://github.com/openai/codex/commit/0735c519789d300097554425cc3d7cf3f2d718a1) Block goals after three empty automatic continuation turns (#44320)
- 2026-09-09 [45eec73b1](https://github.com/openai/codex/commit/45eec73b115f3a281eeb5c7df3d1e698105c3e79) Add opt-in provisioned macOS CLI release candidates (#44307)
- 2026-09-09 [fa7af3883](https://github.com/openai/codex/commit/fa7af3883df4d14861f825f9a6aadbe1ffebe63d) Allow user-requested goal pauses through `update_goal` (#44290)
- 2026-09-09 [d117c2eb0](https://github.com/openai/codex/commit/d117c2eb02bb808cddc13f0a44767549b5577853) Expand MXC volume grants and resolve deny globs (#44289)
- 2026-09-09 [bb71d758c](https://github.com/openai/codex/commit/bb71d758cdfe0fda6626a6cfb08e048d920a35cd) Add telemetry for the Windows system config namespace (#44284)
- 2026-09-09 [aa88a0333](https://github.com/openai/codex/commit/aa88a0333c751711901ce078f4b272a6a970f9b9) Preserve tool output truncation budgets across resume and fork (#44248)
- 2026-09-09 [205f3671e](https://github.com/openai/codex/commit/205f3671e14e8306919501cdef36ca2e5d360f5a) Use captured step settings for tool planning and execution (#44242)
- 2026-09-09 [4f2449b4b](https://github.com/openai/codex/commit/4f2449b4b21988d5015ce6edf755fbd6a37a4908) Measure total exec-server request duration including queueing (#44207)
- 2026-09-09 [0d46c252b](https://github.com/openai/codex/commit/0d46c252b3f29f10bacf0ef58a17a1aa5d17ead3) Encapsulate executed tool call metadata recording (#44002)
- 2026-09-09 [dafb6781e](https://github.com/openai/codex/commit/dafb6781ee76b06ed41bdbe004484087925e22a6) Heap-allocate the resume future in the legacy history test (#43966)
- 2026-09-09 [8c72f2ff5](https://github.com/openai/codex/commit/8c72f2ff564f5328b0c757c394a7e165f4c7a693) Use curly apostrophes in protocol error messages (#43961)
- 2026-09-09 [808b3411f](https://github.com/openai/codex/commit/808b3411fdd0dd05d7a9f1c221a5bc87934943ac) Cache protected shell snapshots and harden capture cleanup (#43954)
- 2026-09-09 [929389f59](https://github.com/openai/codex/commit/929389f59696171f494e3105e56853ad2ec26c98) Preserve per-image generation IDs in image generation analytics (#43953)
- 2026-09-09 [5b682c987](https://github.com/openai/codex/commit/5b682c987528bb3be560ef75f3759ae89160d949) Show configured app-server updater settings in doctor (#43948)
- 2026-09-08 [6ab3ae532](https://github.com/openai/codex/commit/6ab3ae532346a7e900bcef8323a0a18f3764b504) Stabilize subagent and unified exec test fixtures (#43936)
- 2026-09-08 [f419c3214](https://github.com/openai/codex/commit/f419c3214ab84ce6305c86f5e8f1b34475f1c611) Remove the repository devcontainer configurations (#43915)
- 2026-09-08 [1530f828c](https://github.com/openai/codex/commit/1530f828cbaea015bc0fc53c0486e2f889a677f3) Preserve complete shell snapshot exports through filtering and replay (#43907)
- 2026-09-08 [4fd2c460d](https://github.com/openai/codex/commit/4fd2c460dd3e02495c1d0aae8bce4ccbd59915a9) Extract Windows deny-read glob scan planning into protocol (#43903)
- 2026-09-08 [94e4b3d0b](https://github.com/openai/codex/commit/94e4b3d0bd5c2cddfea3b12d1a2d52095854eba7) Preserve `__oailb` routing cookies in ChatGPT HTTP clients (#43895)
- 2026-09-07 [530383e36](https://github.com/openai/codex/commit/530383e36de9c74cd79177a0c31d35609019134f) Warn when the connected Codex service is older than the CLI (#43622)
- 2026-09-07 [769a6a5bc](https://github.com/openai/codex/commit/769a6a5bcd57138effa9f29773737e132f676b9b) Record the launched app-server executable identity in PID files (#43552)
- 2026-09-07 [cc737efd6](https://github.com/openai/codex/commit/cc737efd65b9317f425d093669486dfc5658a20b) Preserve the multi-agent version when forking at a turn cutoff (#43540)
- 2026-09-07 [53ba408a2](https://github.com/openai/codex/commit/53ba408a2f5502313792e46e5730a056e233bcab) Fix jemalloc tools and compiler flags for Bazel musl builds (#43533)
- 2026-09-07 [dbe2f6d52](https://github.com/openai/codex/commit/dbe2f6d52812a99ad929f6f342fa95d9da15b6ed) Expose a stable executor build identity in environment metadata (#43513)
- 2026-09-07 [6750f5bd1](https://github.com/openai/codex/commit/6750f5bd1356fe1553c0fcc9f2632704f3055946) Treat zombie processes as inactive in the Unix PID backend (#43504)
- 2026-09-07 [d665e3bbc](https://github.com/openai/codex/commit/d665e3bbc81b013baa73d067507169c20395b988) Include unloaded children in multi-agent v2 environment context (#43491)
- 2026-09-07 [c84003c7e](https://github.com/openai/codex/commit/c84003c7e18dac40750ebb826a6c72e1c5493b9a) Add diagnostic labels to shell snapshot capture metrics (#43454)
- 2026-09-07 [c0b628571](https://github.com/openai/codex/commit/c0b6285711f788cfb65c6e46c6f23fa0f2a8b3ec) Pin V8 release manifests and prevent published release replacement (#43444)

### Guardian and approvals (41)

- 2026-09-10 [0447e4a1f](https://github.com/openai/codex/commit/0447e4a1fd2ceedfd50e3781adafc03f1cf2f46c) Remove path-bearing fields from Guardian review analytics (#44352)
- 2026-09-10 [c6a59ef92](https://github.com/openai/codex/commit/c6a59ef923c19935003ad55ef8e141a4fe5b9e04) Support native verification in MCP tool continuations (#44346)
- 2026-09-09 [742472c52](https://github.com/openai/codex/commit/742472c52587a80576cf461b0a7dd484682a0750) Set turn triggers for guardian and memory requests (#44298)
- 2026-09-09 [72348693e](https://github.com/openai/codex/commit/72348693eca4599ee3657dc2c039335256f4ccde) Enforce the async Guardian classifier's complete input budget (#44293)
- 2026-09-09 [fcd90d8f0](https://github.com/openai/codex/commit/fcd90d8f07ab558dc4a5d44ca85f9c6ae67d13e1) Enforce complete request budgets for Guardian reviews (#44281)
- 2026-09-09 [2617ed2e1](https://github.com/openai/codex/commit/2617ed2e1c4b9fb59a9058fe28a8f9e78bc81878) Move synchronous Guardian orchestration into the reviewer extension (#44252)
- 2026-09-09 [e8e7103cb](https://github.com/openai/codex/commit/e8e7103cb96e41b1b14f0aff3ca18ea50555fdcb) Extract Guardian review policy into a dedicated crate (#44227)
- 2026-09-09 [2bba3a29a](https://github.com/openai/codex/commit/2bba3a29a0537a1869d76ec2982b8d5e8d02ee9f) Use explicit histogram buckets for Guardian context metrics (#44181)
- 2026-09-09 [17e64839e](https://github.com/openai/codex/commit/17e64839eb1e30632eef4a0147862345fccb61cc) Add aggregate budget enforcement for Guardian context (#44166)
- 2026-09-09 [d3ffbbed5](https://github.com/openai/codex/commit/d3ffbbed5a5bec8393268860c46ad0198890601f) Add Guardian context cost and request token telemetry (#44164)
- 2026-09-09 [3d3df0a0c](https://github.com/openai/codex/commit/3d3df0a0cad5d3d8d3340b633787e9dd304ea463) Raise Guardian's action review limit to 200,000 bytes (#44060)
- 2026-09-08 [82d4a9891](https://github.com/openai/codex/commit/82d4a989124d6786b1145f871a7ca610cdc220a4) Add cancellation for native user-verification RPCs (#43925)
- 2026-09-08 [dd112a9fd](https://github.com/openai/codex/commit/dd112a9fd56dd18a73e593edaa9c2faddcec1106) Keep Guardian reviewers on summary-based compaction (#43912)
- 2026-09-08 [cbfa321ec](https://github.com/openai/codex/commit/cbfa321ecdf8a7d455c911e126e45244f925c2f3) Wait for parent idle before rollback in guardian fork tests (#43842)
- 2026-09-08 [0337192df](https://github.com/openai/codex/commit/0337192dfd10e12ac633dcd159fa6d6120dbfe11) Centralize Guardian transcript policy in context profiles (#43806)
- 2026-09-08 [0034ef93a](https://github.com/openai/codex/commit/0034ef93a7ae15e38652f09a55ed31adefb8e39f) Centralize Guardian context composition (#43805)
- 2026-09-08 [537195129](https://github.com/openai/codex/commit/5371951292bbff1cc27a68c839ba95bfec444375) Batch non-user history eviction to preserve Guardian transcript deltas (#43798)
- 2026-09-08 [d6489472f](https://github.com/openai/codex/commit/d6489472f3c15e87d2d7763a5fde033545c530f8) Enable user verification for the bundled TUI on supported devices (#43715)
- 2026-09-08 [c7f81afc1](https://github.com/openai/codex/commit/c7f81afc191d74ef6e96a5add25eee05662d2215) Enable MCP user verification in the TUI (#43712)
- 2026-09-08 [95327467c](https://github.com/openai/codex/commit/95327467c3af9533ac25b171b3496b951fe425ed) Add TUI request bookkeeping for user verification (#43708)
- 2026-09-08 [54e04f25d](https://github.com/openai/codex/commit/54e04f25dbfe342bf84809d1880dbca32cb43cf6) Add a TUI user verification prompt component (#43702)
- 2026-09-08 [e7637306b](https://github.com/openai/codex/commit/e7637306bc9246a3e42e407cb94f96b7ed345e3e) Add macOS user verification with Secure Enclave signing (#43624)
- 2026-09-07 [d75ed505d](https://github.com/openai/codex/commit/d75ed505d7dd1e73d58defae19b1c2dc1f3e1da6) Move Guardian REPL evidence rendering into the shared context registry (#43602)
- 2026-09-07 [b4373e53a](https://github.com/openai/codex/commit/b4373e53ab79df7baadc6805dea54a060b820307) Move Guardian image selection into shared context sections (#43601)
- 2026-09-07 [f5331dc23](https://github.com/openai/codex/commit/f5331dc237fd496f23afb6c56702d3df40a53bdb) Move trusted skill evidence into the Guardian context registry (#43599)
- 2026-09-07 [255423956](https://github.com/openai/codex/commit/255423956117c82ca558b87173eb76379c7b0664) Move trusted tool metadata into shared Guardian context (#43597)
- 2026-09-07 [0b9b5ecff](https://github.com/openai/codex/commit/0b9b5ecff3b5951bc6c86c40aa91da3f7d6e86b1) Centralize bounded Guardian review evidence in guardian-context (#43595)
- 2026-09-07 [98a5cb46b](https://github.com/openai/codex/commit/98a5cb46b110dcf813b45373f46a2763c4711436) Manage synchronous Guardian reviewers through the thread manager (#43570)
- 2026-09-07 [ca6fb194b](https://github.com/openai/codex/commit/ca6fb194b695dda38d8ccfcb5871b4dbd334b960) Wire app-server user verification RPCs to the native provider (#43568)
- 2026-09-07 [b7ad941b1](https://github.com/openai/codex/commit/b7ad941b1fc1418484fec810f6cbade64b95034c) Add user-verification provider abstractions and RPC adapters (#43547)
- 2026-09-07 [81f23bc18](https://github.com/openai/codex/commit/81f23bc18610badfe5d494dbb9c2410592d2d651) Move Guardian permission context into the shared section registry (#43538)
- 2026-09-07 [93ac34141](https://github.com/openai/codex/commit/93ac341410b9698c8b4badd5df5b7561d93c4ef9) Preserve Guardian context sections and share planned-action rendering (#43534)
- 2026-09-07 [1e66885a1](https://github.com/openai/codex/commit/1e66885a16161048215a3782ecdd1739aab0aabf) Discount an approval's own code-mode wrapper from Guardian score lag (#43527)
- 2026-09-07 [f326857cf](https://github.com/openai/codex/commit/f326857cf405fb254cf6c8f38766daff074fca6e) Restrict MCP user verification and add workspace-scoped identity (#43524)
- 2026-09-07 [d70044072](https://github.com/openai/codex/commit/d70044072c05e8a5c8b16cac76c75368a860c454) Expose shared Guardian reviewer helpers through `guardian_review` (#43490)
- 2026-09-07 [16ff14c26](https://github.com/openai/codex/commit/16ff14c266179e6a762dc8081e9dab73a96683e0) Retain inherited Guardian instructions in standalone forks (#43478)
- 2026-09-07 [aa12ab45d](https://github.com/openai/codex/commit/aa12ab45df0af59423fe21d4b9064539a5bc2244) Recover missing Guardian root instructions in acceptance order (#43472)
- 2026-09-07 [db0568dbb](https://github.com/openai/codex/commit/db0568dbbb853ce2c377a27a94b5546d4a4d2ec3) Remove legacy Guardian approval review paths (#43462)
- 2026-09-07 [8260619cb](https://github.com/openai/codex/commit/8260619cb6dbb23fc43fd9b8567e98fb103aebd4) Centralize Guardian context mode and checkpoint policy (#43458)
- 2026-09-07 [ce5c4133b](https://github.com/openai/codex/commit/ce5c4133bd0494ca2c1bc8593e1be87787c293f4) Route MCP elicitations through the shared approval decision path (#43447)
- 2026-09-07 [5b85aea97](https://github.com/openai/codex/commit/5b85aea9797ce75152d3dfa3651b58f1bbd7d218) Keep Guardian review evidence consistent and reject stale approvals (#43442)

### MCP, plugins and Code Mode (10)

- 2026-09-10 [d996b4f02](https://github.com/openai/codex/commit/d996b4f02a204b36060de4711e106e8ecc7be9ac) Report OAuth authentication failures in MCP status snapshots (#44359)
- 2026-09-09 [b5544d573](https://github.com/openai/codex/commit/b5544d5732f40431c57bf6ca3bed5b127cdf27a0) Persist disabled plugin IDs in thread settings (#44332)
- 2026-09-09 [eb680c055](https://github.com/openai/codex/commit/eb680c055860c41897af44ac8fcf1af56ce8ffd9) Give hosted Codex Apps an independent MCP protocol opt-in (#44318)
- 2026-09-09 [3436cad5a](https://github.com/openai/codex/commit/3436cad5abbe9199c061880421b16d96a9ba702b) Fix MCP elicitation cancellation and reset state on reconnect (#44238)
- 2026-09-09 [20f109ead](https://github.com/openai/codex/commit/20f109eadb9b45360e6ca4f1dee2e82c83a48f7a) Reuse MCP bindings while cached servers remain dormant (#44121)
- 2026-09-09 [5ac0b8768](https://github.com/openai/codex/commit/5ac0b8768d952280940682da09aed0279603f550) Surface MCP reconnect signals when expired OAuth tokens cannot refresh (#43947)
- 2026-09-08 [6d377e96e](https://github.com/openai/codex/commit/6d377e96eb5e57504c8387ebbbbbac38825a1fa8) Propagate Apps tool refreshes to existing threads (#43900)
- 2026-09-08 [9d83c48e5](https://github.com/openai/codex/commit/9d83c48e5c4761c4fe29995305914021dcfbe7cd) Preserve thread identity in code-mode tool dispatch traces (#43894)
- 2026-09-08 [c1f1467f3](https://github.com/openai/codex/commit/c1f1467f3028bd433c8f2063ecc28dd5be206df6) Handle undefined values before JSON serialization in code mode (#43873)
- 2026-09-08 [44ab72674](https://github.com/openai/codex/commit/44ab72674e0f0d8fc5f87a01389912d617793206) Close MCP stderr readers on client teardown (#43870)

### Memory (6)

- 2026-09-08 [2cbbf0c9b](https://github.com/openai/codex/commit/2cbbf0c9b542a36a1c3284b5e804917635b6f666) Add memory dual writing and v2 readiness reporting (#43827)
- 2026-09-08 [553df1c69](https://github.com/openai/codex/commit/553df1c691fe8bf7747e50da22f1342984495ae0) Add dedicated memory v2 consolidation and read prompts (#43813)
- 2026-09-08 [e7f5de0a6](https://github.com/openai/codex/commit/e7f5de0a6ac99ac2d194baa782fc8dea9e236b4f) Move v2 extraction chunking into the memory writer (#43808)
- 2026-09-08 [74d3a5bf1](https://github.com/openai/codex/commit/74d3a5bf1046f004ee33a200ee497dc7593a5687) Add summary-only extraction for memory v2 (#43800)
- 2026-09-08 [6924ce636](https://github.com/openai/codex/commit/6924ce636b2186948f2332ac9460b5871b50aa2f) Prioritize human evidence in memory v2 extraction (#43799)
- 2026-09-08 [3f76e88a4](https://github.com/openai/codex/commit/3f76e88a480f15258eab3512e7eafca77bbe80ee) Add configurable memory versions with isolated storage (#43797)

### Models and compaction (18)

- 2026-09-10 [ea53c8d4f](https://github.com/openai/codex/commit/ea53c8d4f78e3f2c9ae2bafcb387d677f33d8b6a) Add opt-in model discovery for OpenAI API keys (#44392)
- 2026-09-09 [5d3f8752f](https://github.com/openai/codex/commit/5d3f8752fc49662356832373db448f2b4bb21c34) Preserve prewarmed reasoning effort across replay and early rollback (#44285)
- 2026-09-09 [9caddc5cf](https://github.com/openai/codex/commit/9caddc5cf5bf4df5f114498e23bece90eaedb37b) Surface environment startup failure reasons to the model (#44277)
- 2026-09-09 [f5c5d9b2b](https://github.com/openai/codex/commit/f5c5d9b2b093583a1251c5f5cff0bf52addef914) Avoid duplicate reasoning effort updates during turn recovery (#44276)
- 2026-09-09 [1ac689cc7](https://github.com/openai/codex/commit/1ac689cc7de0e637d35271edbbc132d1553d1767) Remove the unused legacy remote compaction implementation (#44273)
- 2026-09-09 [3dc1e2a58](https://github.com/openai/codex/commit/3dc1e2a58406dc69db5812539adfee7d89fa9ef7) Always use streamed remote compaction for supported providers (#44255)
- 2026-09-09 [eb7bd64ef](https://github.com/openai/codex/commit/eb7bd64ef935cedb6a2c90c77c78d6677f1efb45) Remove retired model entries while preserving migration prompts (#44250)
- 2026-09-09 [6eecd04fc](https://github.com/openai/codex/commit/6eecd04fc165cd4573fcbc220fb0b49d3dee7fa5) Normalize image detail for the receiving model (#44249)
- 2026-09-09 [b64de2f3a](https://github.com/openai/codex/commit/b64de2f3ad718ebee731aa8a5e1d4ace0594efc5) Use the originating model when recording conversation history (#44243)
- 2026-09-09 [8ff4aa8ee](https://github.com/openai/codex/commit/8ff4aa8ee402a0ab855c203d0436665cc0cc5470) Use captured step model settings for extension context (#44202)
- 2026-09-09 [b4507997e](https://github.com/openai/codex/commit/b4507997e0bbbaab718bd5e30df1ea3d8ce85893) Use captured step settings when building model context (#44200)
- 2026-09-08 [f046cf35d](https://github.com/openai/codex/commit/f046cf35df0978a628d78a7d4bb6b1d3362a27dd) Scope model catalog caches to the current provider and auth identity (#43906)
- 2026-09-08 [f31bd3adf](https://github.com/openai/codex/commit/f31bd3adffcf6c514193e0355429d56dea208d27) Persist provider and auth identity with model catalog caches (#43897)
- 2026-09-08 [35d9e4bc4](https://github.com/openai/codex/commit/35d9e4bc4d7a44dc84d86a54f4e815ea5cad6986) Preserve reasoning effort through compaction and reset it on success (#43796)
- 2026-09-08 [31ccaf40c](https://github.com/openai/codex/commit/31ccaf40c2298bb3286c8fe274e1e21c498bb70a) Pin request reasoning effort while configuration overrides are active (#43795)
- 2026-09-07 [8e694e955](https://github.com/openai/codex/commit/8e694e955ae02ca737230a5468c55d5847074072) Exclude base instructions from the bundled model catalog (#43604)
- 2026-09-07 [4f1a2bb5f](https://github.com/openai/codex/commit/4f1a2bb5ffed8fd28518925c1d4085ead158e304) Preserve fork runtime versions without loading full model context (#43545)
- 2026-09-07 [f3f53ee94](https://github.com/openai/codex/commit/f3f53ee949eeaa9b6050699a783b94fe4ee8ff0d) Wait for thread idle before rollback in model-switching tests (#43456)

### SDK (7)

- 2026-09-09 [1a4096e27](https://github.com/openai/codex/commit/1a4096e273e80da30947e57fdfa45be92858ca91) Add untrusted external messages to the Python SDK (#44086)
- 2026-09-09 [8afccec87](https://github.com/openai/codex/commit/8afccec87aa15f73ee7fc35a3a4e7834afc5ef62) Expose Python SDK history selection and per-turn options (#44084)
- 2026-09-09 [b4d42052c](https://github.com/openai/codex/commit/b4d42052cd0fe621cec83dd35582676904ce958c) Publish Python packages after stable CLI releases (#44067)
- 2026-09-09 [26ce6649a](https://github.com/openai/codex/commit/26ce6649a256c00f89121cf56bdefb5c90577319) Build Python SDK artifacts before publishing the runtime (#44061)
- 2026-09-09 [96c2b4377](https://github.com/openai/codex/commit/96c2b4377f5f0b1363ba4d4057fbbb7c6dccd9d8) Gate Python SDK publishing on runtime availability and verify PyPI files (#44055)
- 2026-09-09 [c55db1b8d](https://github.com/openai/codex/commit/c55db1b8d9b876230f9fb22be78d77fc0b526d0c) Test Python SDK against the built CLI and installed runtime (#44053)
- 2026-09-09 [45134c046](https://github.com/openai/codex/commit/45134c04632b971776d88edfdea71c051ebe4161) Generate Python SDK types from repository app-server schemas (#44032)

### Threads, storage and lifecycle (31)

- 2026-09-10 [5a9eb145c](https://github.com/openai/codex/commit/5a9eb145c4c05fcfc7158d7c25b80e1322eccae1) Update the forked-thread hook test to use `StartThreadOptions` (#44377)
- 2026-09-10 [2df0b747b](https://github.com/openai/codex/commit/2df0b747ba49c187223824dd84aaf2749d866066) Add thread attachment operations with coordinated deletion (#44350)
- 2026-09-10 [e444aa99d](https://github.com/openai/codex/commit/e444aa99d76beeb8b5b0610b793c0cdefe58da58) Distinguish forked sessions in session-start hooks (#44349)
- 2026-09-09 [130d6e4fb](https://github.com/openai/codex/commit/130d6e4fba5612e2e6e0724e6463b510380498da) Add paginated thread attachment listing to the state runtime (#44330)
- 2026-09-09 [e1b23086a](https://github.com/openai/codex/commit/e1b23086acbf905592036c587cb37a12d68c441e) Restore saved threads when the managed daemon restarts (#44314)
- 2026-09-09 [434efa95e](https://github.com/openai/codex/commit/434efa95e62941640471d3bb75449d31ba18980b) Honor shared Retry-After deadlines for remote control (#44311)
- 2026-09-09 [7c88f037d](https://github.com/openai/codex/commit/7c88f037d935b4e78311027c32da4f046fd84058) Record thread recovery candidates on managed daemon shutdown (#44299)
- 2026-09-09 [87cf20ee4](https://github.com/openai/codex/commit/87cf20ee491a035036f2d905926df4a0d35951cc) Isolate the hook pipe I/O timeout test from shell startup files (#44297)
- 2026-09-09 [885113aa1](https://github.com/openai/codex/commit/885113aa1d68ccbc567b83698feac3cd42163088) Prevent command hooks from hanging on blocked stdin (#44288)
- 2026-09-09 [c1840dc55](https://github.com/openai/codex/commit/c1840dc55e3cbb7ef27ccc1fb20b38cdd0e9ef68) Persist loaded threads before managed daemon shutdown (#44283)
- 2026-09-09 [a2e83a783](https://github.com/openai/codex/commit/a2e83a783e492a40a3b8d72589ed0e0caf6922a7) Continue rollout searches when a compressed rollout cannot be searched (#44226)
- 2026-09-09 [ce2c2759e](https://github.com/openai/codex/commit/ce2c2759ebee2d64565922f6f7365082284f9570) Release persistent writers when session startup is cancelled (#44183)
- 2026-09-09 [0df1daf52](https://github.com/openai/codex/commit/0df1daf5269b51c768c428e6bfded32210cc8cc9) Attach compressed rollouts to diagnostic reports as JSONL (#44175)
- 2026-09-09 [73a1148c9](https://github.com/openai/codex/commit/73a1148c9c775c2a4616ce5096291740a00ed68a) Coordinate rollout compression with active thread writers (#44138)
- 2026-09-09 [283f34387](https://github.com/openai/codex/commit/283f34387b7e16bd524d8f3a431f77aa7395471d) Use `StartThreadOptions` across thread fork APIs (#44043)
- 2026-09-09 [c3eeaae9a](https://github.com/openai/codex/commit/c3eeaae9a3d401f7e7ade0817b7b27299af7d2a1) Gate new app-server work during graceful shutdown (#43959)
- 2026-09-09 [102e1763b](https://github.com/openai/codex/commit/102e1763b90e70d79a0bba5d262cf71f904583c1) Keep app-server thread RPCs active until delegated work completes (#43950)
- 2026-09-09 [589874be8](https://github.com/openai/codex/commit/589874be811a4034026dbae27d328776c25bb73c) Add transactional thread attachment mutations to the state runtime (#43949)
- 2026-09-08 [7c098d874](https://github.com/openai/codex/commit/7c098d87418a357bafb241a579a18938801989e0) Gate new turn submissions on host shutdown admission (#43943)
- 2026-09-08 [9d88e9ae0](https://github.com/openai/codex/commit/9d88e9ae086128b83292f4ad9868aadb0ed6b0d9) Rename thread artifacts to attachments in the state database (#43927)
- 2026-09-08 [78932f449](https://github.com/openai/codex/commit/78932f4493995cb1236bcad411c926a9f5c179b2) Expose the queued event count on `CodexThread` (#43918)
- 2026-09-08 [cfd5d77d6](https://github.com/openai/codex/commit/cfd5d77d63f53303aad63ef0886d3b159996a1af) Detach Unix hook commands from the controlling terminal (#43876)
- 2026-09-08 [6515a72db](https://github.com/openai/codex/commit/6515a72db7a82e8cdebed940aad4ba1a159ce245) Preserve runtime workspace roots across thread resume (#43848)
- 2026-09-08 [df522cae1](https://github.com/openai/codex/commit/df522cae16b4f4350d6a9e13bc10cbe9650b0f9b) Limit app-server storage metrics to session directories (#43790)
- 2026-09-07 [a51608398](https://github.com/openai/codex/commit/a51608398d53b6d23ed98b8287de415b35f1eea5) Make the managed app-server shutdown grace period configurable (#43572)
- 2026-09-07 [daca1fab8](https://github.com/openai/codex/commit/daca1fab840303152d7b488a4b403dee2f85b62f) Add an explicit app-server daemon update command (#43562)
- 2026-09-07 [7d8e2dd6c](https://github.com/openai/codex/commit/7d8e2dd6c5b29e1e80709cb051b163794edf2f27) Make app-server daemon automatic updates configurable (#43542)
- 2026-09-07 [c9c7b73c4](https://github.com/openai/codex/commit/c9c7b73c4faa49b02a8f644c31670ff26276c29e) Ensure the standalone updater runs on managed daemon starts (#43529)
- 2026-09-07 [adee0b04f](https://github.com/openai/codex/commit/adee0b04fa27a8ba5d2e3612b900363cffe72930) Preserve standalone release pins during daemon updates (#43521)
- 2026-09-07 [9f70e348e](https://github.com/openai/codex/commit/9f70e348e0227980de97e361cce830236fb18317) Allow internal sessions to fork from selected history (#43495)
- 2026-09-07 [d0a8dcd15](https://github.com/openai/codex/commit/d0a8dcd15721adb02a3227d08d3cde888e5546e5) Limit archive rollout reads to requested threads (#43494)

### TUI and worktrees (21)

- 2026-09-10 [a62e98d18](https://github.com/openai/codex/commit/a62e98d18c6550e3bea152ed1b89d1e931dca961) Return focus to the agents overview composer on Escape (#44360)
- 2026-09-10 [e2a9ee05f](https://github.com/openai/codex/commit/e2a9ee05f4f6e805bf705131ddee87e9cc22e35d) Extract shared footer hint wrapping in the TUI (#44354)
- 2026-09-10 [3ef3cecd2](https://github.com/openai/codex/commit/3ef3cecd2002c7841519df37417426d66fd5cc3d) Open tasks with Right from the agents overview (#44344)
- 2026-09-09 [2808a9c34](https://github.com/openai/codex/commit/2808a9c348ee90a6fc94aee1570dd3fdf2c0b021) Clear pending TUI questions when accepting a new prompt (#44328)
- 2026-09-09 [c77c34ed3](https://github.com/openai/codex/commit/c77c34ed33877a6e5b3759703d01d3b223274cbf) Reduce TUI stack usage during session transitions (#44176)
- 2026-09-09 [2ce38ae6d](https://github.com/openai/codex/commit/2ce38ae6d84a0aa96768742a5ac888a02ff622c7) Support image attachments in agents overview background tasks (#44027)
- 2026-09-09 [b83105710](https://github.com/openai/codex/commit/b83105710695b70b6d96a64d1e4612bdf68d5f92) Clear stale transcript history when switching threads (#43994)
- 2026-09-09 [4e09b0c1f](https://github.com/openai/codex/commit/4e09b0c1f1134e918160cef043619898f462d9cc) Increase the TUI thread capability test stack to 12 MiB (#43956)
- 2026-09-08 [973dcd80f](https://github.com/openai/codex/commit/973dcd80fcbe2864ab2229d67f10180ded2e2bc7) Show worktree owner details and add confirmed deletion (#43942)
- 2026-09-08 [1032738aa](https://github.com/openai/codex/commit/1032738aa09b0f0d0d45f7e24e4a96fff219926a) Tag TUI startup metrics with terminal and multiplexer categories (#43937)
- 2026-09-08 [9ec33e192](https://github.com/openai/codex/commit/9ec33e19260907aad207185a624bd8b936d5afa8) Show streaming reasoning summaries in the TUI status row (#43921)
- 2026-09-08 [095da4b7e](https://github.com/openai/codex/commit/095da4b7e8b70b01afb5c6131ef926dcb8c0d85d) Fix transcript viewer restoration and half-page scrolling (#43889)
- 2026-09-08 [dd9512c00](https://github.com/openai/codex/commit/dd9512c0008859f6c95fb2dae2290a2cb94b38fb) Include completed commentary in the `/copy` picker (#43846)
- 2026-09-08 [49a9d7899](https://github.com/openai/codex/commit/49a9d789997ab40d1d75b117644f796988d32e18) Make older app-server notices configurable in the TUI (#43698)
- 2026-09-07 [4b0f44d30](https://github.com/openai/codex/commit/4b0f44d3046f5212e618d0b5fb5225a2f1988989) Add worktree classification to thread telemetry (#43621)
- 2026-09-07 [c977cc0c1](https://github.com/openai/codex/commit/c977cc0c19e704aa60e6c9b85107869f87b8b46f) Add a stable TUI/app-server version comparison helper (#43619)
- 2026-09-07 [7d2c58e6e](https://github.com/openai/codex/commit/7d2c58e6e0811d326accdfe9913417a44893dd2f) Recover missed tmux resize notifications in the TUI (#43603)
- 2026-09-07 [a44454656](https://github.com/openai/codex/commit/a44454656459437fc8e2ffa9eca0646537b1fdfd) Remove a stale transcript field assignment from the TUI (#43584)
- 2026-09-07 [411034232](https://github.com/openai/codex/commit/4110342321bb19b0053190750a0a8b76427b13ad) Group adjacent computer actions in the TUI (#43576)
- 2026-09-07 [333c41eef](https://github.com/openai/codex/commit/333c41eef6b9ba3697fe913973fd58afe32d1ef5) Show completion timestamps after successful TUI turns (#43558)
- 2026-09-07 [b1205c12d](https://github.com/openai/codex/commit/b1205c12d584c36613ab24a49aaf351e7b3fdb9e) Set `recursion_limit` to 256 for app-server, exec, and TUI (#43519)

### Voice (19)

- 2026-09-09 [e722303e3](https://github.com/openai/codex/commit/e722303e38ad8e4ffb69c098e86108b895a99a68) Expose voice conversations in experimental features (#44331)
- 2026-09-09 [ccf470c06](https://github.com/openai/codex/commit/ccf470c060d1c86c9c7f8d42dc3dabf507b56c9a) Preserve voice indicator styles during composer sparkle effects (#44198)
- 2026-09-09 [9e868bd9d](https://github.com/openai/codex/commit/9e868bd9dc007c05e84a98e0b1f4e31dc98c5e6a) Handle empty voice arguments in macOS release packaging (#44101)
- 2026-09-09 [7b9e7d99b](https://github.com/openai/codex/commit/7b9e7d99bdeb3c2f3041a2fcc219cddb64a50a0f) Make staged macOS voice runtimes writable before packaging (#44080)
- 2026-09-09 [129fd2168](https://github.com/openai/codex/commit/129fd21687fbd4ac48133b7abfdcaf52cb6cb01f) Reject empty audio payloads in data URLs (#44070)
- 2026-09-09 [85c2d4d92](https://github.com/openai/codex/commit/85c2d4d921688324334f2d91a62529c08d17d617) Fix voice runtime release builds and packaging (#44062)
- 2026-09-09 [7aba21885](https://github.com/openai/codex/commit/7aba21885159559819f2400b1d2259844beb531e) Refresh workspace lockfile before building macOS voice releases (#44025)
- 2026-09-09 [721f46a07](https://github.com/openai/codex/commit/721f46a07ab48f00b5e7cdbf2efb78b993d100de) Bundle signed voice resources in macOS releases (#43983)
- 2026-09-08 [fba22e9a2](https://github.com/openai/codex/commit/fba22e9a2aca9798e86982fd1f7b8ccb08e34807) Track voice session lifecycle metrics in the TUI (#43934)
- 2026-09-08 [ef6c05820](https://github.com/openai/codex/commit/ef6c0582028ff8e1228e6031465c1c2d9d5d51b6) Disable clock synchronization in the voice audio sink (#43704)
- 2026-09-08 [6b6fdc357](https://github.com/openai/codex/commit/6b6fdc3572c328e0ba7d67a5eca9e558738d22c7) Preserve split-flap animation state when voice transcripts scroll (#43699)
- 2026-09-08 [9a3af22d0](https://github.com/openai/codex/commit/9a3af22d01bc9abcafb1e3aa01cb4738a2caf37a) Stabilize realtime voice meter sampling across redraws (#43695)
- 2026-09-08 [45305dd22](https://github.com/openai/codex/commit/45305dd229c01e6cb6e122f6559e4f9b805bae6b) Make the voice mute shortcut configurable in the TUI (#43690)
- 2026-09-08 [98c7c0415](https://github.com/openai/codex/commit/98c7c0415bf4af42fa291406c3ac730363741bc9) Move voice controls into a dedicated composer strip (#43683)
- 2026-09-08 [3caf9f958](https://github.com/openai/codex/commit/3caf9f9586baedb4158a7b91545ead3dd320c348) Style spoken prompts and link workspace files in voice transcripts (#43676)
- 2026-09-08 [4e93cf9b4](https://github.com/openai/codex/commit/4e93cf9b4e4e86f49473478c8288426cb6d6b119) Animate live voice transcripts with split-flap tiles (#43656)
- 2026-09-08 [4b0d9669c](https://github.com/openai/codex/commit/4b0d9669cc46ba97bf85fa6312431b630d80498d) Add voice mute shortcut and recording activity indicators (#43651)
- 2026-09-08 [6fee98cc8](https://github.com/openai/codex/commit/6fee98cc85a69ee44a856121cac0f64505cf48d0) Expand TUI regression coverage for realtime voice conversations (#43645)
- 2026-09-07 [b01c3986f](https://github.com/openai/codex/commit/b01c3986fd2e79b8a477a08d81430f52f22bc0dc) Add live WebRTC voice conversations to the TUI (#43581)
