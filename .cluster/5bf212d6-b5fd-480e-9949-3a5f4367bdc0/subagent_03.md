# Architecture Consistency Audit · subagent_03

**角色：** 架构一致性审计员  
**审计方向：** 从设计和计划层面判断是否"过度设计以致于永远走不到交付"  
**审计日期：** 2026-07-14  
**审计范围：** 8 个核心设计文档 + 交叉引用一致性 + W4 scaffold 必要性

---

## 结论

**架构没有过度设计，但 W4 scaffold 存在严重的执行性膨胀 —— 73 个 spike CLI 命令中的 60+ 个是元信息脚手架，不是在逼近交付。项目设计文档的内聚性极强（5 个独立轴、防退化约束、失败开安全），但计划执行线已偏离设计初衷：W4 本应是 provider 探测工具，实际变成了一个自证无害的"证明-关于-证明"的无限递归机器。**

---

## 1. W0-W6 工作包定义的清晰性（24-implementation-work-packages.md）

### 可验证交付物证据

| 工作包 | 可验证交付物 | 是否足够具体 |
|---|---|---|
| W0 | `cargo fmt/clippy/test --workspace` 全通过；无 network/UI/SQLite 依赖在 core crate | ✅ 清楚 |
| W1 | F1-F11 夹具族 + cross-layer property 清单（10 条） | ✅ 清楚 |
| W2 | Gate A 性能目标（P95 <=80ms 首帧, idle CPU <=0.10%, memory <=45MB） | ✅ 清楚 |
| W3 | C0-C9 场景 + Safe Mode 测试 + Drop Mode <=10MB P95 | ✅ 清楚 |
| W4 | provider report/manifest/matrix/fixtures/scorecard — 均为 read-only 脚手架 | ⚠️ 膨胀 |
| W5 | 首个 `supported_observe` adapter，按 scorecard 选择 | ✅ 清楚 |
| W6 | 独立 source-gated 增强（路线证明、token ledger、burn meter 等） | ✅ 清楚 |

### 分析

W0-W3 的交付物标准是**可执行、可测量的** — 具体到毫秒、MB、测试场景编号。但 W4 的定义发生了质变：

> "provider report contains version, environment category, integration mode, capability matrix, known limitations, resource figures, and release recommendation"

这是一个**文档产物而非功能产物**的定义。W4 的 `Required outcomes` 全部是关于报告的格式字段，没有一条是关于"做了某个真实探测并得到了某个可验证结果"。

### 关键信号

W4 文档声明 `"Explicit non-goals: Shipping provider support, dynamic plugins, production analytics, control features"`，但同时 `W4-PROBE-HARNESS-AUDIT.md` 展示了 73 条 `pulse-island-spike --provider-*` 命令 — **大部分命令输出的是关于"我们还没做探测"的元信息，而非探测本身**。

---

## 2. Gate A-H 定义（10-verification-gates-and-mvp-roadmap.md）

### 门禁结构

| Gate | 主题 | 退出标准 |
|---|---|---|
| Gate 0 | Product shell baseline | Island show/hide, Link singleton, resource measurement harness |
| Gate A | Native Signal Benchmark | 7 项性能指标（首帧 <=80ms, idle CPU <=0.10% 等） |
| Gate B | State Kernel + Reducer | F1-F11 夹具全通过 + 确定性证明 |
| Gate C | Link Lifecycle + Drop Mode | C0-C9 场景 + Drop Mode <=10MB P95 |
| Gate D | Codex CLI adapter | 8 项探针（start->attach->complete->fail->route） |
| Gate E | Claude Code adapter | 类似 D，但强调 "without pretending to own the interactive client" |
| Gate F | Antigravity probe | 探测报告优先于完整 adapter |
| Gate G | Pulse Fuel | 按 source 分层的 quota/token/burn 报告 |
| Gate H | Attention & notification | 100 events = 0 Toast; 分组提醒等 |

### 分析

Gate 定义具有**可验证性**：每个 gate 有具体的 pass condition 和测量指标。但 Gate D/E/F 的定义存在一个未解决的依赖：

- Gate F（Antigravity）定义为 "A probe report, not necessarily a full adapter" — 这合理
- 但 Gate D/E 要求完整 adapter 的前置条件（install/rollback、lifecycle mapping、late attach）依赖于 W4 的 provider 探测完成
- W4 目前处于 `read-only scaffold` 状态，所有 provider 标签为 `not_probed`

**关键矛盾**：Gate 定义本身是合理的，但执行线卡在 W4 的 scaffold 膨胀上。

---

## 3. W4 Scaffold 范围合理性（W4-PROBE-HARNESS-AUDIT.md）

### 已实现的 scaffold 命令分布

| 类别 | 命令数 | 示例 |
|---|---|---|
| 清单/报告生成 | ~15 | `--provider-probe-manifest`, `--provider-probe-report`, `--provider-capability-matrix` |
| 夹具/模板 | ~10 | `--provider-config-transaction-fixture`, `--provider-resource-fixture`, `--provider-sanitized-evidence-output-template` |
| 评分/资格检查 | ~10 | `--provider-probe-scorecard`, `--provider-hard-disqualifiers`, `--provider-release-label-evaluation` |
| 证据/审计 | ~10 | `--provider-evidence-register`, `--provider-evidence-gap-summary`, `--provider-probe-audit` |
| 授权/运行手册 | ~10 | `--provider-authorized-evidence-runbook`, `--provider-direct-evidence-import-checklist` |
| 前置检查/就绪 | ~10 | `--provider-w5-start-preflight`, `--provider-live-authorization-preflight`, `--provider-probe-readiness` |
| 实际探测（只读） | ~5 | `--provider-live-probe-run=read-only-local`, `--provider-local-environment-manifest=read-only-fixture` |
| 完成门禁 | ~3 | `--provider-w4-completion-gate`, `--provider-w5-observe-adapter-contract` |

### 分析

**73 个命令中，至少 60 个是元信息脚手架。**它们是在生成关于"我们为什么还没做真实探测"和"真实探测应该怎么做"的文档，而不是在做真实探测。

具体膨胀分析：

1. **递归自引用**：`--provider-probe-audit` 输出 W4 状态摘要；`--provider-probe-readiness` 输出就绪状态；`--provider-w4-completion-gate` 输出完成状态 — **三个命令都在回答同一个问题**

2. **前置-前置-前置链**：
   - `--provider-live-authorization-preflight` -> `not_authorized` by default
   - `--provider-authorized-evidence-runbook` -> defines manual steps without executing them
   - `--provider-direct-gate-packet` -> exports packet without collecting evidence
   - `--provider-direct-evidence-import-checklist` -> rejects sanitized fixtures as release-elevating
   - **这是一个 4 层 deep 的"如何在未来某天做某事"的脚手架，没有一层做了实际工作**

3. **模板冗余**：
   - `--provider-sanitized-evidence-output-template` 和 `--provider-sanitized-evidence-bundle-validator` 分别输出红action模板和验证器 — 但没有任何实际证据需要模板化

4. **沙盒自证**：`--provider-w5-observe-adapter-contract` 输出的是 "未来 W5 adapter 应该遵守的契约"，但 W5 还没开始

### 严重性判断

这些脚手架命令的**代码量远超过 W4 的实际价值交付**。W4 的 15-provider-capability-probe.md 定义其目的是：

> "create repeatable evidence collection and capability reports before shipping an adapter"

但当前 W4 实现的是 "create repeatable reports about why we haven't collected evidence yet"。**目标偏移了 180 度。**

---

## 4. Provider 探测需求是否过度（15-provider-capability-probe.md）

### P0-P8 阶段总览

| 阶段 | 名称 | 必要性 | 是否过度 |
|---|---|---|---|
| P0 | Official-surface inventory | 识别正式集成路径 | ❌ 合理 |
| P1 | Passive process discovery | 建立诚实降级底线 | ❌ 合理 |
| P2 | Integration install/rollback | 安全安装/卸载证明 | ❌ 合理 |
| P3 | Lifecycle semantics | 将 provider 事件映射到 Pulse 状态 | ❌ 合理 |
| P4 | Late attach | 核心用户承诺的证明 | ❌ 合理 |
| P5 | Context routing | 路由标签与真实结果对齐 | ❌ 合理 |
| P6 | Fuel telemetry | 区分 quota 与 task usage | ⚠️ 对 MVP 偏重 |
| P7 | Fail-open fault injection | 确保 Pulse 不损害 provider | ❌ 合理 |
| P8 | Performance/retention | 真实负载下的资源验证 | ❌ 合理 |

### 分析

P0-P8 的**分层定义本身合理** — 每个阶段回答了可验证的问题，且有明确的 pass condition。问题不在探测协议的设计，而在实现路径：

> 正确的做法：P0 -> 对 Codex 做 P0 -> 如果 P0 pass，做 P1 -> ... -> 达到 `supported_observe`
> 
> 当前的做法：构建 73 个脚手架命令来生成关于 P0-P8 的报告模板 -> 所有报告都输出 `not_probed` -> 永远卡在 W4

**探测协议本身不过度。过度的是 W4 实现中为探测协议搭建的"代理决策系统" — "系统生成了 73 种关于探测的报告格式，却没有做一次探测"。**

### 过度设计的核心证据

`provider-direct-evidence-import-checklist` 明确声明：**"rejects sanitized fixtures and read-only version evidence as release-elevating inputs"**

这意味着：
- 已完成的 read-only 本地探测（Codex CLI 版本检测、命令存在性检查）**被 W4 自己的规则宣判为无效**
- 要突破 W4 -> W5，需要 "authorized direct evidence"，但 authorization preflight **默认输出 `not_authorized`**
- 形成了一个自我锁定的门禁系统：收集证据需要授权 -> 授权需要证据 -> 循环

---

## 5. 适配器选择流程（18-first-adapter-selection-and-bootstrap.md）

### 选择机制

Scorecard 用 0-3 分评估 10 个维度，然后加权：
- Safety/fail-open: 25%
- Truthful lifecycle/waiting: 20%
- Late attach: 15%
- Install/rollback: 15%
- Context return: 10%
- Resource: 10%
- Fuel: 5%

另有 8 个 hard disqualifiers。

### 分析

选择机制**设计良好**：
- 权重分布合理（safety 最高，fuel 最低）
- hard disqualifiers 阻止了危险的集成路径
- "Neither provider is designated first yet" 是正确的姿态

但问题在于：**这个选择流程需要一个实际通过了 P0-P8 探测的 scorecard 作为输入**。当前所有 provider 都是 `total_score=0, not_probed`，选择流程虽然设计清楚，但无法执行。

---

## 6. Spike B 设计初衷（13-spike-b-state-kernel.md）

核心问题：

> "Can the pure state kernel reduce incomplete, delayed, duplicated, and conflicting evidence without inventing lifecycle, Fuel, route certainty, or provider capability?"

### 分析

Spike B 的 9 条设计原则清晰且一致：
- "Unknown, Observed, and Degraded are successful truthful outcomes"
- "The kernel may discard detail but never create certainty"
- "All fixtures are synthetic and content-minimized"

F1-F11 11 族夹具覆盖了完整的生命周期、终端保护、身份安全、路由真实性、隐私保留和风暴边界。每族有具体的 pass condition。

**Spike B 的设计没有过度设计** — 它对 MVP 来说是最小可行内核。

---

## 7. Link 协议复杂度（14-spike-c-link-transport-drop-mode.md）

### 架构元素计数

| 元素 | 复杂度 |
|---|---|
| 进程拓扑 | 4 个（Shim, Link, Island client, Synthetic host） |
| Link 状态机 | 7 个状态（NotRunning->Starting->Warm->Active->IslandActive->DropMode->GracePeriod->CheckpointAndExit->NotRunning） |
| IPC 对象 | Mutex + 2 pipes + Ready event |
| 消息类型 | Ingress: 2 种; Island request: 6 种; Link response: 5 种 |
| 帧头字段 | 7 个（magic, protocol_major/minor, message_kind, flags, request_id, payload_length, reserved） |
| C0-C9 场景 | 10 个 |
| 性能指标 | 12 个 |

### 分析

**Link 协议的复杂度是 MVP 需要的**：
- 每用户单实例 mutex 是基本正确性保证
- 继承匿名管道传递首事件是隐私必须（命令行不可见）
- 7 状态机反映了 on-demand 生命周期的真实需求
- C0-C9 10 个场景覆盖了正常路径、竞态、故障和风暴

但有一个关键点：**Spike C 使用 synthetic events/fake Island client，不需要真实 provider 集成**。这符合 "prove transport, not integration" 的设计意图。

---

## 8. 交叉一致性：25-consistency-closure.md vs 00-product-foundation.md

### 高优先级一致性检查

| 主题 | 00-product-foundation | 25-consistency-closure | 一致性 |
|---|---|---|---|
| Canonical priority order | 9 层（failed->waiting->limit->pinned->fuel_risk->stall->running->terminal->idle） | 同 9 层（加详细定义） | ✅ 一致 |
| Route strength labels | Exact/Strong/Useful/Weak + 具体 wording | 同 + "Open original task is Exact-only" | ✅ 一致 |
| Provider posture | Codex/Claude probe candidates; Antigravity passive only | 同 | ✅ 一致 |
| Fuel decomposition | "reported quota <> task tokens <> burn meter <> verified limit block" | 同，加 "Fuel never displaces waiting/failure" | ✅ 一致 |
| Link as on-demand | "It is not a permanent background service" | "There is no permanent running Idle Link state" | ✅ 一致 |
| Privacy profile retention | Minimal/Strict/Passive-only | 同 + 细则（终端面包屑移除行为） | ✅ 一致 |
| MVP definition | "one narrow loop with one provider selected by probe" | 同，通过 W4/W5 gating 实施 | ✅ 一致 |
| Safe Mode contract | Shim 检测 Safe Mode->no Link wake | 同 + 详细接受场景 | ✅ 一致 |

### 发现的差异

1. **00 说 Signal 模型：** `[state] [provider/workspace-safe subject] [one reason] [+N]` — 此表述在 25 中并未重复确认，但也没有冲突

2. **00 的 "User controls" 包含 "pin/follow/mute tasks"、但 25 和 24 的 W6 工作包中列为独立增强 — 这是一个依赖顺序的正确声明**

3. **00 标注 `Consistency baseline: 25-consistency-closure.md`** 自声明了 25 的 normative 权威 — **一致性机制本身是一致的**

### 判断

**00 和 25 之间没有实质性矛盾。** 25 如其所声称的那样，是在更晚时间点对 00 的细化澄清。两者共同确立了 5 轴独立模型（provider release status, task health, route capability, feature capability, Fuel source）、防退化约束、和失败开安全。

---

## 9. 27-open-gates.md 的 5 个剩余 Gate 评估

### 5 个 Gate 列表

1. **Codex Hook transaction**: 隔离安装->运行真实 task->验证 Shim events->rollback->证明无关配置字节级保留
2. **Lifecycle evidence**: SessionStart/activity/PermissionRequest/Stop 到 Link state 的映射
3. **Late attach**: 关闭 Island->运行 Hook-backed task->重新打开 Island->验证 degraded snapshot + fresh evidence
4. **Native UI**: `apps/pulse-island` 连接现有 UI 模型和 Win32 HWND/compositor
5. **Release candidate**: 完整 install/upgrade/repair/disable/uninstall/rollback

### 可否绕过？

| Gate | 可否绕过？ | 理由 |
|---|---|---|
| #1 Codex Hook transaction | ❌ 不可绕过 | 这是 install/rollback safety 的最小证明 — 如果不验证 config mutation 安全性，Pulse 可能损坏用户环境 |
| #2 Lifecycle evidence | ❌ 不可绕过 | 这是整个产品的核心 — 如果不能将真实 provider 事件映射到 Pulse 状态，产品就没有意义 |
| #3 Late attach | ❌ 不可绕过 | 这是 00-product-foundation 定义的 MVP 核心承诺 — Island opens later |
| #4 Native UI | ⚠️ 部分可解耦 | HWND lifecycle smoke 已通过；Signal/Peek/Focus 渲染、DPI、无障碍可与 #1-#3 并行开发。但完全绕过意味着没有可视化产品 |
| #5 Release candidate | ⚠️ 可降级 | 可以先做一个非正式 build（实验性发布）而无需完整 installer 流程；但 installer 是 Windows 桌面应用的基线预期 |

### 判断

**5 个 gate 中，#1-#3 是逻辑上不可绕过的 MVP 硬依赖** — 它们定义了从 Hook event 到 Island 显示的完整因果链。#4-#5 可以与 #1-#3 解耦或降级，但不能被省略。

关键问题不是能否绕过这些 gate，而是：**当前 W4 的 73 个脚手架命令中，有没有任何一个能帮助通过这些 gate？**

答案是：**几乎没有。** 脚手架命令可以帮助理解"我们应该如何做探测"（--provider-authorized-evidence-runbook），但不能替代做探测本身。

---

## 10. 关键判断：W4 Scaffold vs 最小可行 Provider 集成

### 问题重述

> W4 scaffold 的 73 个 spike 命令是否有必要在集成前写？还是应该直接做一个最小可行 provider 集成来替代这些 scaffold？

### 证据

1. **W4 的 73 个命令中，至少 60 个是元信息脚手架** — 生成的是"关于探测的报告"而非探测本身

2. **文档自我强化的阻塞链：**
   - `--provider-w5-start-preflight` -> `w4_complete=false, w5_start_allowed=false`
   - `--provider-direct-evidence-import-checklist` -> rejects sanitized fixtures
   - `--provider-live-authorization-preflight` -> `not_authorized` by default
   - **形成三层锁：W4 不完成 -> 不能做 W5 -> 但 W4 完成需要直接证据 -> 直接证据需要授权 -> 授权需要 W4 完成**

3. **MVP 真正需要的最小路径：**
   ```
   Codex CLI 已安装在开发机上
   -> 手动添加一个 Hook entry（脉冲 Shim）
   -> 运行一个 Codex task
   -> 验证 Shim 收到事件
   -> 验证 Link 收到帧
   -> 验证 Island 可以 attach
   -> 卸载 Hook entry
   -> 验证 config 未被破坏
   ```
   这个路径**不需要 W4 的 60+ 个脚手架命令。**

4. **15-provider-capability-probe.md 的 P0-P8 阶段设计是合理的** — 但可以在一个**单一的、手动的、有文档记录的探测会话**中按顺序执行，而不需要先写 73 个 CLI 命令来"为探测做准备"

### 判断

**W4 scaffold 中的 60+ 个元信息命令没有在集成前写的必要性。**它们是设计良好的"探测协议"被过度工程化为"探测-关于-探测的元系统"的结果。

**替代方案：**
- 保留 ~10 个核心命令（manifest, report, scorecard, capability-matrix, resource-fixture, hard-disqualifiers, completion-gate）
- 删除 ~50+ 个元信息/模板命令
- 直接用一台开发机做 Codex CLI 的 P0-P4 手动探测
- 将探测结果记录到 `provider-probe-report=codex_cli` 中（已有此命令）
- 如果探测 success -> 进入 W5
- 如果探测 fail -> `--provider-missing-capability-rationale=codex_cli` 记录原因（已有此命令）

**这样可以将 W4 的代码量削减 ~80%，同时将 W4->W5 的实际前置时间从"无限期自我阻塞"缩短到"一次授权的探测会话"。**

---

## 11. 过度设计信号汇总

| 信号 | 严重程度 | 证据 |
|---|---|---|
| 脚手架 -> 元脚手架 -> 元元脚手架递归 | 🔴 高 | `--provider-authorized-evidence-runbook` 生成手工步骤 -> `--provider-sanitized-evidence-output-template` 生成模板 -> `--provider-sanitized-evidence-bundle-validator` 验证模板 — 三者在做同一件事：定义"未来如何做" |
| 自我锁定的门禁链 | 🔴 高 | authorization preflight -> `not_authorized`; direct-evidence-import-checklist -> rejects 现有证据; completion-gate -> `w4_complete=false` |
| "防御性证明"膨胀 | 🟡 中 | 大量命令输出"我没有做 X、我没有存 Y、我没有声明 Z" — 这些是合规需求而非功能需求 |
| 模板/runbook 的命令式产出 | 🟡 中 | 10+ 个命令输出的是 human-readable runbook/template/checklist，但产品不需要 CLI 命令来生成文档 |
| 设计文档间的自体引用 | 🟢 低 | 28 个设计文档 + 4 个 audit 文档 + 3 个 probe card = 35 个 .md 文件；但 25-consistency-closure 提供了权威层次结构，缓解了不一致风险 |

---

## 12. 合理约束

以下设计决策是**正确的、不应改变的**：

1. **5 轴独立模型**（provider release status <> task health <> route capability <> feature capability <> Fuel source） — 防止"supported_observe 隐含支持一切"
2. **防退化原则** — 不确定性降低声明而非扩大收集
3. **失败开安全** — Shim/Link 故障不能影响 provider 行为
4. **隐私保底线**（不存储 prompt/transcript/command-line/credentials）
5. **Late attach 作为核心承诺**（不要求 Island 先于 provider 打开）
6. **Provider 选择由证据而非偏好决定**

---

## 13. 关键缺失

| 缺失 | 影响 |
|---|---|
| **W4->W5 的真实过渡路径** | 当前设计定义了从 `not_probed` 到 `supported_observe` 的所有中间状态和验证规则，但**没有人能明确说出下一个真实步骤是什么** — 因为 authorization preflight 默认拒绝一切 |
| **时间盒/cost cap** | 没有任何文档规定 W4 的脚手架工作应该花费多少时间或达到什么程度后必须进入实际探测 |
| **"足够好"的退出标准** | W4 completion gate 要求"direct evidence exists"，但 direct evidence 被定义为 live probe execution — 这个鸡生蛋问题没有解决 |

---

## 14. 建议入档位置

### 建议记录的发现

1. **W4 脚手架精简**（入 `W4-PROBE-HARNESS-AUDIT.md` 或新 `W4-SCOPEDOWN.md`）：保留 ~10 个核心命令，删除 ~50+ 个元信息命令
2. **W4->W5 快速路径**（入 `27-open-gates.md`）：定义"一次授权的 Codex CLI 手动探测会话"作为突破 W4 死锁的明确下一步
3. **架构约束重申**（可入 `25-consistency-closure.md` 或新 section）："探测协议是手段，provider 集成是目的。脚手架不应超过实际探测工作的 20%。"

### 不需要入档的内容

- 设计文档的核心架构决策（5 轴模型、防退化、失败开）不需要修改
- Gate A-H 的定义不需要修改
- Spike B/C 的设计不需要修改

---

## 15. 总体评估

| 维度 | 评分 | 说明 |
|---|---|---|
| 架构设计质量 | 8/10 | 5 轴独立模型、防退化、失败开安全 — 这些都是经过深思熟虑的 |
| 设计文档一致性 | 9/10 | 25-consistency-closure 的权威层次有效解决了多文档不一致风险 |
| 计划可行性 | 4/10 | W0-W3 可行并已完成；W4 陷入脚手架膨胀死锁 |
| 过度设计风险 | 🔴 高 | 73 个 W4 命令中有 60+ 个是元信息，形成自我加强的阻塞链 |
| 到 MVP 的剩余距离 | 🟡 中 | 逻辑上只需要 5 个 gate（27-open-gates），但 W4 的死锁必须先解决 |

**一句话：架构设计优秀，W4 执行脱轨 — 不是设计过度，是脚手架的元信息递归过度；解决方案不是在设计层面大改，而是在 W4 做一个 brutal scopedown，保留核心探测能力并直接做一次真实 Codex CLI 探测。**
