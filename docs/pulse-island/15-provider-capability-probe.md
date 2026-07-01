# Pulse Island · Provider Capability Probe Protocol

**Status:** Research-to-integration gate  
**Applies to:** Codex CLI, Claude Code, Antigravity CLI, and any future provider adapter  
**Depends on:** `07-adapter-ecosystem.md`, `08-integration-hook-protocol.md`, `10-verification-gates-and-mvp-roadmap.md`, `14-spike-c-link-transport-drop-mode.md`  
**Last updated:** 2026-07-01

---

## 1. Purpose

Before Pulse advertises an integration, it must prove the integration against the real provider surface. A compile-successful adapter, a process detector, or an appealing demo does not establish support.

This protocol turns provider research into a repeatable capability decision:

```text
Official evidence
+ clean install / rollback
+ live lifecycle exercise
+ late attach exercise
+ context-route exercise
+ fault and privacy exercise
+ resource measurement
= capability-by-capability release decision
```

The probe exists to protect the product from two failures:

1. claiming a provider capability that is unavailable or unsafe in a normal user session
2. withholding a useful low-capability observation mode because richer control is unavailable

---

## 2. Probe output

Every probe produces a capability report, not a binary “supported / unsupported” verdict.

```text
ProviderProbeReport
├── provider and tested version
├── Pulse version / test build
├── Windows version / terminal host / shell
├── official evidence register
├── tested integration modes
├── capability matrix
├── lifecycle mapping matrix
├── late-attach result
├── context-route result
├── Fuel source result
├── install/update/uninstall result
├── failure / fail-open result
├── privacy review
├── resource measurements
├── known degradations
├── recommended release label
└── decision log
```

### 2.1 Permitted release labels

| Label | Meaning |
|---|---|
| `not_probed` | No usable evidence yet. No product claim. |
| `process_observed` | Process-level presence only. |
| `experimental_attached` | Some lifecycle attachment works under documented constraints; explicit opt-in. |
| `supported_observe` | Required P0 observation, late attach, rollback, and truthfulness gates pass. |
| `supported_fuel` | A particular usage capability is verified in addition to observation. |
| `supported_control` | A narrowly scoped formal control capability passed a separate safety review. Not P0. |

A provider can hold multiple labels per capability. For example: `supported_observe`, `supported_fuel` unavailable, `supported_control` not_probed.

---

## 3. Research constraints

### 3.1 Evidence policy

A product capability needs one of:

- documented official API or SDK behavior
- documented official Hook or integration behavior
- verified official CLI/App behavior in a reproducible test
- provider-local structured data whose format and usage are explicitly validated by the probe

Evidence must record:

```text
source type
source location
published/updated date if available
provider version tested
exact capability claim supported
known constraints
```

### 3.2 Prohibited research paths

The probe must not use:

- private or reverse-engineered endpoints
- browser cookie or credential extraction
- process memory inspection
- terminal OCR or screen scraping
- simulated mouse/keyboard control as an API substitute
- hidden CLI flags, trust bypasses, or unsafe permission mode
- decompilation or packet interception to find undocumented controls

A provider with no suitable source stays lower-capability. The probe must not invent a workaround that violates the product boundary.

### 3.3 Test account and workspace hygiene

- use a dedicated test workspace with synthetic source files only
- use a disposable or appropriately authorized test account where needed
- never check secrets, user prompts, session data, provider config, or raw transcripts into the repository
- store raw evidence only outside source control, with an explicit local retention policy
- commit only sanitized derived fixtures and report summaries

---

## 4. Probe environment manifest

Every run begins with a concise environment manifest.

```text
ProbeEnvironment
├── provider_name
├── provider_version
├── integration_surface_version
├── Windows build
├── shell / terminal host
├── PowerShell or command shell version
├── Pulse build commit
├── adapter build flags
├── test workspace fixture revision
├── account mode (redacted category only)
└── timestamp
```

The manifest records compatibility context without storing credentials, account identifiers, full filesystem paths, or command history.

---

## 5. Probe phases

### P0: Official-surface inventory

Goal: identify candidate formal integration paths before code is written.

For each provider, record candidates under these headings:

```text
lifecycle Hook
local API / app-server / SDK
session identity
waiting / permission event
terminal completion/failure event
workspace identity
context/deep-link route
session token telemetry
quota / reset telemetry
formal control capability
user-level config install method
rollback/uninstall path
```

Output is an evidence register. It does not yet enable any product capability.

### P1: Passive process discovery

Goal: establish the honest fallback floor.

Test:

- start provider normally in a synthetic workspace
- observe process identity, PID, parent relation, start time, and safe workspace association if available
- exit provider normally and abnormally

Pass condition:

- Pulse can create `Observed` state without claiming lifecycle semantics it cannot prove.

### P2: Integration installation and rollback

Goal: prove that Pulse can add and remove its own integration safely.

Test:

```text
existing user-level config with unrelated entries
→ install Pulse-owned integration entry
→ validate provider still starts normally
→ update Pulse entry
→ uninstall Pulse entry
→ compare unrelated config preservation
```

Pass condition:

- no project-level configuration modification by default
- no unrelated entry deletion/reordering
- no trust bypass requirement
- clean rollback works after success and intentionally interrupted install

### P3: Lifecycle semantics

Goal: map provider evidence to Pulse lifecycle states.

For each observed provider event, record:

```text
provider event name/source
accepted input fields
identity strength
mapped NormalizedEvent kind
source rank
lifecycle effect
attention effect
freshness policy
safe summary rule
unknown/ambiguous behavior
```

Required exercises:

- session/task starts
- normal active work
- ordinary quiet interval
- user-needed or permission request if formally observable
- normal completion
- clear failure
- provider process/session exit without a formal outcome
- adapter/source disconnect and recovery

Pass condition:

- every claimed Pulse lifecycle transition is tied to tested evidence
- quiet activity never becomes a false failure
- exit without outcome becomes `terminated` or degraded, not completed

### P4: Late attach

Goal: test the central user promise under normal launch behavior.

Test:

```text
install Pulse integration
→ start provider normally while Island is closed
→ let task become active
→ start Island / fake Island client later
→ verify task state and identity
→ restart Island / fake client
→ verify reconnection
```

Required negative test:

```text
start provider before Pulse integration/Link existed
→ open Pulse later
→ verify only supported cold-discovery level is shown
```

Pass condition:

- no provider restart, interruption, duplicated work, or synthetic new session
- state labels accurately downgrade when previous lifecycle evidence is missing

### P5: Context routing

Goal: prove route labels against real outcomes.

Test strongest candidate first:

1. exact provider session/thread
2. original terminal or application window
3. workspace
4. provider surface
5. process details

For every route:

```text
route evidence
route strength
label shown to user
launch action
actual target observed
fallback behavior
failure behavior
```

Pass condition:

- `Open original task` only reaches the exact original context or a strongly verified original window
- lower-quality routes use honest labels
- no route starts a new agent task

### P6: Fuel telemetry

Goal: distinguish provider quota from local task usage.

Run separately for:

```text
session token counters
token deltas
quota window percentage
quota reset time
provider rate/usage limit block
```

For each source, prove:

- source provenance
- session/account scope
- update cadence
- reset behavior
- stale behavior
- counter rollover/reset behavior
- privacy impact

Pass condition:

- no session token total is shown as account quota
- no account quota is claimed without a supported source
- limit block maps to lifecycle `limited` only when it demonstrably blocks work

### P7: Fail-open and fault injection

Goal: ensure Pulse cannot harm normal provider behavior.

Inject:

- Link absent
- Shim timeout
- malformed incoming event
- Link crash
- Island crash
- pipe disconnection
- storage write failure
- adapter source error
- integration config corruption after install

Pass condition:

- provider continues native behavior
- waiting/approval remains provider-native when Pulse cannot safely bridge it
- Pulse degrades its own health/state without fabricating failure

### P8: Performance and retention

Goal: validate the real adapter under realistic event volume.

Measure:

- Link Drop Mode memory/CPU
- active Link memory/CPU
- event-to-snapshot latency
- adapter event rate and coalescing behavior
- breadcrumb size and checkpoint frequency
- Link exit behavior

Pass condition:

- adapter remains within relevant Gate A–H budgets
- no raw event/history accumulation appears
- no background polling loop is introduced without explicit justification

---

## 6. Capability matrix template

Use this table for each provider and each tested integration mode.

| Capability | Evidence source | Probe result | Identity strength | Health ceiling | User-visible wording | Release label |
|---|---|---|---|---|---|---|
| Discover process |  |  |  |  |  |  |
| Discover session |  |  |  |  |  |  |
| Observe running |  |  |  |  |  |  |
| Observe waiting |  |  |  |  |  |  |
| Observe completion |  |  |  |  |  |  |
| Observe failure |  |  |  |  |  |  |
| Observe safe title |  |  |  |  |  |  |
| Open exact context |  |  |  |  |  |  |
| Open workspace |  |  |  |  |  |  |
| Open official usage |  |  |  |  |  |  |
| Observe session tokens |  |  |  |  |  |  |
| Observe quota snapshot |  |  |  |  |  |  |
| Observe quota limit |  |  |  |  |  |  |
| Control decision |  |  |  |  |  |  |
| Control stop/steer/resume |  |  |  |  |  |  |

Blank cells are not “probably yes.” They are `not_probed` or `unavailable`.

---

## 7. Provider-specific probe cards

### 7.1 Codex CLI

Probe only formal, documented candidate surfaces.

Questions to answer:

- Which Hook events are available at user scope and which include a safe session identity?
- Which lifecycle states can be mapped with terminal evidence?
- Can a normal terminal-launched task be observed after Island opens later, when Link was installed beforehand?
- Which current-context route is actually reliable for a task launched outside Pulse?
- Which usage/limit source is supported and what scope does it represent?
- Are formal control methods limited to Pulse-managed sessions rather than independently running terminal work?

Do not assume that a stored conversation/thread record permits safe take-over of a live independently launched terminal task.

### 7.2 Claude Code

Probe official user-level Hooks and any formal local surfaces.

Questions to answer:

- Which hooks provide session identity, workspace, lifecycle, completion, failure, and waiting signals?
- Can a Hook notify Pulse without changing native permission behavior?
- Can the original terminal/window be reliably focused, or is workspace routing the safe ceiling?
- What local token/session telemetry is available without treating it as account quota?
- Which account usage source, if any, is sufficiently formal to show quota?

Do not claim arbitrary external interactive-session control unless a formal public control path is proven.

### 7.3 Antigravity CLI

Treat as a capability investigation, not a pre-approved adapter.

Questions to answer:

- Is there an official integration/Hook registration mechanism?
- Is a stable session identity provided?
- Are lifecycle events available externally?
- Is any exact-context/deep-link route available?
- Is usage/token/quota data formally exposed?
- Can a minimal user-level integration be installed and rolled back safely?

Until these answers are proven, the release ceiling is `process_observed` or `workspace-ready` only where independently verifiable.

---

## 8. First-adapter selection

The first provider adapter should be selected by measured integration quality, not brand priority or perceived popularity.

Score only verified findings:

| Dimension | What earns score |
|---|---|
| Safe ingress | documented user-level lifecycle source with fail-open behavior |
| Identity | stable session ID plus process/workspace correlation |
| Truthfulness | explicit completion/failure/waiting semantics |
| Late attach | reliable breadcrumb + reconnect result |
| Context return | exact or strong route, then useful workspace fallback |
| Fuel | trustworthy scoped usage source |
| Install safety | targeted update/rollback with no unrelated config mutation |
| Resource profile | stays within Link budget under event traffic |
| Fault behavior | provider unaffected by Pulse faults |

A provider that only offers high-quality Hook observation and workspace routing may beat a provider with more theoretical control if the latter cannot meet truthfulness or fail-open requirements.

---

## 9. Evidence retention

### Commit to repository

- sanitized probe report
- capability matrix
- sanitized event mapping fixtures
- test harness results expressed as categories/metrics
- known limitation statements
- release decision

### Keep local only, if needed during active research

- provider documentation exports where licensing permits
- synthetic test run recordings
- provider configuration backups under user-local protected storage
- redacted error captures

### Never retain

- customer/project source code
- full prompts/transcripts
- account credentials/cookies/tokens
- raw terminal buffers
- private endpoint traffic
- unredacted real workspace paths

---

## 10. Probe acceptance and decision

At the end of a provider probe, choose one explicit outcome:

```text
A. Proceed to narrow Adapter implementation
B. Ship Process Observed / Passive mode only
C. Offer Experimental observation behind opt-in
D. Defer provider pending official integration support
E. Reject integration path as unsafe or incompatible
```

The report must explain why capabilities are absent. “Not supported yet” is a product-quality outcome when it prevents a false claim.

---

## 11. Design invariants

1. A provider adapter begins with evidence, not wishful abstraction.
2. Every user-visible capability has a tested source, ceiling, and failure behavior.
3. A provider's missing control API does not invalidate its observation value.
4. A provider's missing safe lifecycle surface limits Pulse claims rather than changing Pulse's privacy rules.
5. Real task content never becomes the price of integration research.
6. A first adapter is chosen by verified reliability, not by feature-count theater.
