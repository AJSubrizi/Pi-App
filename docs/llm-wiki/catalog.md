# Pi 对齐：模型 / 推理 / 权限 / 模式

源码：`src/li./agentCatalog.ts`（静态兜底）、`src-tauri/src/models_catalog.rs`、`src-tauri/src/agent_prefs.rs`。

## 模型

**UI 只展示真正可用的模型。服务商是后端渠道，只在设置 → Providers & Models 切换。**

| 来源 | 说明 |
|------|------|
| `models_cache.json` | CLI 官方目录 |
| 静态兜底 | `pi-4.5` |

探测：`scripts/probe-models.sh`。Host：`models_list_available`。

Spawn 顺序（CLI 0.2.x）：

```text
pi agent --model <id> --reasoning-effort <e> [--always-approve] stdio
```

Flags **必须在** `stdio` 之前。连接后 `session/set_model` 再对齐一次。

## 推理强度（effort）

CLI `models_cache.json` 每模型可带 `info.reasoning_efforts: [{id,value,label,description,default}]`。Host 经 `AvailableModel.reasoningEfforts`（`isDefault`）下发；composer 列表优先用该数组，空则回退静态 `PI_FALLBACK_EFFORTS`（`low` | `medium` | `high`）。展示标签优先用 catalog `label`，否则 i18n `effort.high|medium|low`。

Spawn：`--reasoning-effort <id>`。无模型级默认时 App 默认 **`medium`**；有 `default: true` 时用模型默认。中途修改：soft-disconnect agent → 下一条消息重连。无 `session/set_effort` RPC。

### 连接加速（Host）

| 手段 | 说明 |
|------|------|
| 默认 medium effort | 比 high 更短 thinking / TTFT，比 low 更稳 |
| `pi --no-auto-update agent … stdio` | 跳过启动时更新检查 |
| 进程复用 | 同 cwd + effort + YOLO 标志时，切会话只 `session/load\|new`，不 respawn CLI |
| 打开会话预热 | `openSession` 后台 `session_connect`，首发跳过冷启动 |

## 会话模式（mode）— 产品态

| App | 作用 |
|-----|------|
| `agent` | 默认编码 agent |
| `plan` | 计划模式（ACP `session/set_mode`） |
| `ask` | 询问 / 偏只读协作 |

实现：

1. 连接成功后 `session/set_mode`（尝试 `plan` / `ask` / `agent` 等候选 modeId）。  
2. 中途切换：优先 `set_mode`；失败则 soft-respawn。  
3. 按 `composerPrefsScope` 记忆。

## 权限（含 YOLO）

| App ID | Agent 配置 `[ui] permission_mode` | Claude `defaultMode` | Spawn |
|--------|-----------------------------------|----------------------|-------|
| `ask` | `default` | `default` | — |
| `accept_edits` | `acceptEdits` | `acceptEdits` | — |
| `allow_for_session` | `default` + Host 会话缓存 | `default` | — |
| `dont_ask` | `dontAsk` | `dontAsk` | — |
| `always_approve` | `always-approve` + `yolo=true` | `bypassPermissions` | `--always-approve` |

**Independent 模式**（默认）：写入 `~/.pi-app/agent-home/config.toml` 与 `agent-home/.claude/settings.json`，agent 进程侧真正按策略执行。

**Shared 模式**：不改写用户 `~/.grok/config.toml`；Host 策略 + YOLO 时的 `--always-approve`。

中途改权限：同步配置 + soft-respawn（含 YOLO 降级）。Host 在收到 `session/request_permission` 时仍按 live policy 自动放行/拒绝。

注意：读工具与部分只读 shell 在 agent 内建白名单下仍可能不弹窗（Pi 设计）。

**下载默认放行（Host）**：`curl -o/-O`、`wget`、`aria2c` 等把资源写到**项目目录内**的 shell，Host 在非 `dont_ask`/`deny` 策略下自动批准，避免生图后 `curl` 落盘卡在权限弹窗直至 600s 超时。项目外路径仍须审批（仅 `always_approve` 例外）。

## 偏好记忆范围

`composerPrefsScope` = `global` | `project` | `session`。

覆盖 model / effort / mode / permission。切换 chip → `composer_prefs_set` / `session_set_policy` / `session_set_model`。

## 服务商

自定义提供商 = 渠道路由，**不伪装成模型**。在 Providers & Models
面板配置和切换；Pi package 适配器仍通过 Extensions 安装。
