# paidang-rs-server 设计文档

> Rust 重写 `paidang-worker-server`（Cloudflare Worker / TypeScript）为自托管服务。
> 状态：设计已与用户确认，待评审 → 转入 writing-plans。
> 日期：2026-06-24

## 1. 背景与目标

`paidang-worker-server` 是派单（摄影预约）系统的后端，运行在 Cloudflare Worker 上（Hono + Chanfana + D1/R2/KV）。客户端 `paidang-mini` 是微信小程序，调用 `https://paidang.hkhy.online`。

**重写动机：** Cloudflare 网络卡顿、资源限制。
**目标：** 自托管 Rust 服务，MySQL 替代 D1，腾讯云 COS 替代 R2，解决卡顿与资源限制。

### 已确认决策

| # | 决策 | 选项 |
|---|---|---|
| 1 | 部署形态 | 自托管服务器（Docker 首选，systemd 备选） |
| 2 | DB | MySQL（替代 D1） |
| 3 | 对象存储 | 腾讯云 COS（替代 R2），使用 **qcos** crate |
| 4 | API 兼容 | 大体兼容现有 URL 与 `{success,data}` 信封，允许小幅优化 |
| 5 | 架构方案 | 方案 B：axum + SeaORM + JWT |
| 6 | 现有数据 | fresh start，先不迁移 D1 数据（迁移留待以后） |
| 7 | 鉴权 | JWT（jsonwebtoken），直接上线纯 JWT，不做灰度双头（当前无用户） |
| 8 | 业务触发器 | 从 DB 触发器移到 service 层（slot 锁定/释放逻辑） |
| 9 | 时区 | 固定 **UTC+8** 存储 |
| 10 | 外键 | **表里不做外键绑定**，关联完整性由 service 层 + 索引保证 |
| 11 | 文件下载 | 暂用 Rust 代理流式转发 COS（保持小程序兼容），预签名 URL 留作后续优化 |
| 12 | `/kv` 端点 | 删除（小程序未使用，模板残留） |
| 13 | 测试 | 真实 MySQL（testcontainers-rs）+ 外部服务 mock |
| 14 | 小程序改动 | `request.ts` 存 token + 发 `Authorization` 头 + `login.ts` 去 `user_id` 参数（一次性适配，无灰度） |

## 2. 技术栈

| 关注点 | crate | 理由 |
|---|---|---|
| 运行时 | `tokio` | axum/SeaORM/reqwest 默认底座 |
| Web 框架 | `axum` + `tower` | 主流、中间件清晰 |
| ORM/DB | `sea-orm` + MySQL | entity 模型 + 迁移 + 类型化查询 |
| 鉴权 | `jsonwebtoken` | JWT 签发/校验 |
| HTTP 客户端 | `reqwest` | 微信/Qiniu/COS 调用 |
| COS 客户端 | `qcos` | 腾讯云 COS 对象存储 |
| 配置 | `config` + `dotenvy` | 分环境配置 + 密钥走环境变量 |
| 日志 | `tracing` + `tracing-subscriber` | 结构化日志，替代 console.log |
| 校验 | `validator` + `serde` | 替代 Zod 做请求体校验 |
| OpenAPI | `utoipa` + `utoipa-axum` | 自动生成 `/` 文档，对应 Chanfana |
| 加密签名 | `hmac` + `sha1` | Qiniu HMAC-SHA1 签名 |
| base64 | `base64` | Qiniu 签名编码 |
| 错误 | 自定义 `AppError` enum + `IntoResponse` | 复刻错误信封 |
| 测试 | `#[tokio::test]` + `testcontainers` | 真实 MySQL 集成测试 |
| 迁移 | `sea-orm-migration` | 从 4 个 SQLite SQL 移植 |

### qcos 依赖风险

qcos 是唯一**需先验证可用性**的依赖。实施第一步先做签名/上传冒烟测试；若不可用，fallback 用 reqwest 手写 CAM 签名（与 Qiniu HMAC-SHA1 套路一致）。spec 据冒烟结果确定。

## 3. 项目结构

```
paidang-rs-server/
├─ Cargo.toml
├─ .env / .env.example          # 密钥与配置（不入 git）
├─ config/
│   ├─ default.toml              # 监听地址/端口、JWT 过期、分页默认值
│   └─ production.toml
├─ migration/                   # SeaORM 迁移（4 个 SQLite SQL → MySQL，无外键，UTC+8）
│   └─ src/ m20250101_*.rs ...
├─ src/
│   ├─ main.rs                  # 启动、路由装配、tracing 初始化
│   ├─ config.rs                # 配置加载
│   ├─ app_state.rs             # AppState（DB 连接池、COS client、配置）
│   ├─ error.rs                 # AppError + IntoResponse + 错误码
│   ├─ response.rs              # {success,data} / 分页信封封装
│   ├─ middleware/
│   │   ├─ auth.rs              # JWT 校验，注入 AuthUser{user_id,role}
│   │   ├─ role_guard.rs        # 管理员/摄影师角色守卫
│   │   └─ request_log.rs       # 请求/响应日志（移植现有全局日志中间件）
│   ├─ domain/                  # 按业务域组织（替代 endpoints/）
│   │   ├─ auth/                # {router, login_handler, service, dto}
│   │   ├─ user/                # profile read/update, avatar upload
│   │   ├─ packages/            # +items+gallery CRUD
│   │   ├─ gallery/             # +tags
│   │   ├─ gallery_groups/
│   │   ├─ time_slot_templates/
│   │   ├─ date_slots/          # +day/monthly
│   │   ├─ date_settings/       # +check
│   │   ├─ bookings/            # +stats/today/logs（slot 锁定逻辑在此）
│   │   └─ files/               # upload/list/download(代理)/delete/moderation
│   ├─ entity/                  # SeaORM entity（从 schema 生成）
│   ├─ external/                # 外部集成（trait 抽象，便于测试 mock）
│   │   ├─ wechat.rs            # code2session / token(缓存) / getuserphonenumber
│   │   ├─ qiniu_moderation.rs  # 图审 + HMAC-SHA1 签名（移植 qiniuAuth.ts）
│   │   └─ cos.rs               # qcos 封装：put/get/head/delete/list
│   └─ logs/                    # dev 模式实时监控（移植 logStore.ts）
└─ tests/                       # 集成测试（对应 tests/integration/*）
```

每个 domain 是独立单元：`router`（路由组）、`dto`（请求/响应 + 校验）、`service`（SeaORM 业务逻辑）、`entity`。职责单一、可独立测试。

## 4. 数据模型与迁移

### 4.1 现有 SQLite schema 移植要点

4 个迁移文件移植为 SeaORM 迁移，**无外键**，**UTC+8**。语法改造：

| SQLite | MySQL |
|---|---|
| `INTEGER PRIMARY KEY AUTOINCREMENT` | `INTEGER AUTO_INCREMENT PRIMARY KEY` |
| `datetime('now','localtime')`（create_time） | `DATETIME DEFAULT CURRENT_TIMESTAMP` |
| `datetime('now','localtime')`（update_time）+ 触发器 | `DATETIME DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP` —— **干掉所有 update_time 触发器** |
| `PRAGMA foreign_keys = ON` | 不适用（无外键，`ENGINE=InnoDB`） |
| `INTEGER` 0/1 布尔 | `TINYINT(1)`（SeaORM 映射 bool） |
| `TEXT` 存 JSON（service_items / image_list） | `JSON` 原生类型 |
| `FOREIGN KEY ... REFERENCES` | **移除所有外键约束** |

时区：MySQL `CURRENT_TIMESTAMP` 用系统时区，服务器设 `Asia/Shanghai`（UTC+8）。应用层所有时间按 UTC+8 处理。**不处理 DST，不处理跨时区。**

### 4.2 触发器处理

- **所有 `update_*_trigger`（自动更新 update_time）**：删除，改用 `ON UPDATE CURRENT_TIMESTAMP`。
- **核心业务触发器** `lock_slot_on_booking` / `release_slot_on_booking_status`：**移到 service 层**。语义与现有触发器一一对应：
  - **锁定**：booking 创建（INSERT）且 `status IN ('pending','confirmed')` 且 `slot_instance_id` 非空 → 置 `date_slot.is_booked=1`、`booking_id`。
  - **释放**：booking 状态变更（UPDATE OF status），当新状态 IN (`cancelled`,`refunded`,`completed`) 且旧状态 IN (`pending`,`confirmed`,`in_progress`) 且 `slot_instance_id` 非空 → 置 `date_slot.is_booked=0`、`booking_id=NULL`。
  - 并发安全：用事务 + `SELECT ... FOR UPDATE` 保证 slot 锁定原子性。
  - 集成测试覆盖并发预约同 slot 场景。

### 4.3 关联完整性（无外键约定）

由于表无外键约束，关联完整性由 **service 层显式校验 + 索引** 保证：

- service 层在写子记录前校验父记录存在（如建 booking 前校验 photographer_id/package_id 有效）。
- 保留现有索引（idx_*）以保证查询性能。
- 级联删除（如 `ON DELETE CASCADE`）改为 service 层显式删除关联记录。
- 集成测试覆盖孤儿场景。
- 写一份「关联完整性由应用层保证」的约定文档（置于 `docs/`）。

### 4.4 表清单（13 张，移植自现有 4 个迁移）

`user`、`user_profile`、`package`、`package_item`、`package_gallery`、`gallery_group`、`gallery`、`gallery_tag`、`time_slot_template`、`date_slot`、`date_setting`、`booking`、`booking_log`。字段与现有 schema 一一对应，仅做上述语法/类型改造与外键移除。

## 5. API 设计

### 5.1 响应契约（兼容现有）

- 成功：`{ "success": true, "data": <T> }`
- 分页：`{ "success": true, "data": { "list": [...], "total": N, "page": N, "pageSize": N } }`
- 失败：`{ "success": false, "errors": [{ "code": N, "message": "..." }] }`

### 5.2 错误码

| code | HTTP | 含义 |
|---|---|---|
| 7000 | 500 | 内部错误 |
| 7001 | 400 | 输入校验失败 |
| 7002 | 404 | 资源不存在 |
| 7003 | 401 | 未登录 / token 无效 |
| 7004 | 403 | 已登录但无权限（角色不足） |

`7003`/`7004` 为新增（JWT 鉴权所需）。现有 `InputValidationException`=7001、`NotFoundException`=7002、`ApiException`=7000 语义保留。

### 5.3 端点清单

| 路径 | 方案 | 鉴权 |
|---|---|---|
| `/auth/login` | 保留，登录成功签 JWT | 公开 |
| `/user/profile`（read/update） | 保留 | JWT |
| `/user/avatar`（upload） | 保留 | JWT |
| `/packages/*` | 保留完整 CRUD | 读公开 / 写管理员 |
| `/gallery/*`、`/gallery-groups/*` | 保留完整 CRUD | 读公开 / 写管理员 |
| `/time-slot-templates/*` | 保留 | JWT（摄影师自己的） |
| `/date-slots/*` | 保留（含 day/monthly） | JWT |
| `/date-settings/*` | 保留（含 check） | JWT |
| `/bookings/*` | 保留（含 stats/today） | JWT |
| `/booking-logs/*` | 保留 | JWT |
| `/files/*` | 保留（upload/list/download/delete），R2→COS | JWT |
| `/kv/*` | **删除** | — |
| `/`（OpenAPI 文档） | 保留，utoipa 生成 | 公开 |
| `/logs`、`/logs/api` | 保留，仅 dev 模式 | — |

## 6. JWT 鉴权

### 6.1 登录流程（移植 `login.ts`）

1. 收 `{ code, nickname?, avatar_url?, phone?, phone_code? }`
2. 调微信 `code2session` 拿 `openid`/`session_key`/`unionid`
3. phone：优先 `phone`，回退 `phone_code` 调微信 `getuserphonenumber`（access_token 带内存缓存 + 过期提前 5min，移植现有缓存逻辑）
4. 按 `openid` 查/建 `user` + `user_profile`
5. **签 JWT**：claims `{ sub: user_id, role, openid, exp }`，用 `JWT_SECRET`
6. 返回 `{ ...user, token, is_new }`（`is_new` 保留兼容）

### 6.2 鉴权中间件（`middleware/auth.rs`）

- 从 `Authorization: Bearer <token>` 提取并校验 JWT
- 校验失败 → 7003/401
- 成功 → 注入 `AuthUser { user_id, role }` 到请求扩展
- 无需鉴权的路由（登录、公开读、OpenAPI、/logs）跳过

### 6.3 角色守卫（`middleware/role_guard.rs`）

- axum extractor/中间件，管理员端点（packages/gallery 写操作）校验 `role >= 2`
- 摄影师端点（`time-slot-templates`、`date-slots`、`date-settings`、`bookings` 作为摄影师操作、`user` 自身资料）校验 `role >= 1` 且访问自己资源（service 层校验 owner，即资源 `photographer_id`/`user_id` 等于当前 `AuthUser.user_id`）
- 角色值：0 普通用户，1 摄影师，2 管理员（沿用现有 `user.role`）

### 6.4 身份来源规则（小幅优化，影响小程序）

现有后端从 `X-User-Id` 头取身份，但部分端点（`/user/profile`、`/bookings/stats`、`/bookings/today` 等）还接受客户端在 query/body 里传 `user_id`/`photographer_id`。重写后统一规则：

- **身份一律从 JWT 派生**（`AuthUser.user_id`），不再信任客户端传的 `user_id`。
- `/user/profile`（read/update）：用 `AuthUser.user_id`，**移除客户端 `user_id` 参数**。小程序 `login.ts` 第 61 行 `?user_id=` 改为不传。
- `/bookings/stats`、`/bookings/today`：`photographer_id` 仍由客户端传（管理员/摄影师查看某人档期），但 service 层校验：当前用户 role >= 2，**或** `photographer_id == AuthUser.user_id`（role >= 1），否则 7004。
- `/bookings`、`/date-slots`、`/date-settings`、`/time-slot-templates` 的写操作：owner 字段（`photographer_id`）用 `AuthUser.user_id`，不接受客户端传（防越权替他人操作）。

这些属于"小幅优化"，小程序对应改动并入 §6.5。

### 6.5 小程序适配（一次性，无灰度）

改动文件：`paidang-mini/miniprogram/utils/request.ts` + `paidang-mini/miniprogram/pages/login/login.ts`。

- `request()`：登录后存 `token`，从发 `X-User-Id` 改为发 `Authorization: Bearer <token>`。
- `uploadFile()`：同样改头。
- `login.ts`：登录成功后存 `token`（替代/并存 `user_id`）；预加载资料的 `?user_id=` 去掉。

改动集中在 2 个文件。由于当前无用户，直接上线纯 JWT，不做 `X-User-Id` 灰度兼容。

## 7. 外部集成

### 7.1 微信（`external/wechat.rs`）

- `code2session` / `get_access_token`（内存缓存 + 过期提前 5min）/ `get_user_phone`
- 用 reqwest 调用，trait 抽象便于测试 mock
- 配置：`WX_APPID`、`WX_SECRET`

### 7.2 Qiniu 图审（`external/qiniu_moderation.rs`，移植 `qiniuAuth.ts` + `moderation.ts`）

- HMAC-SHA1 签名（`hmac` + `sha1` crate）、base64url（`base64` crate）
- 图审 API `POST https://ai.qiniuapi.com/v3/image/censor`，scenes：pulp/terror/politician
- `shouldModerate`（MIME + size 判断）/ `summary`（结果汇总）逻辑移植
- upload 流程不变：先审后传 COS，`block` 则跳过上传返回标记
- data URI 用 `application/octet-stream` 前缀（现有修复 `202aa19`）
- 配置：`QINIU_ACCESS_KEY`、`QINIU_SECRET_KEY`（缺失则 moderation 返回 unknown）

### 7.3 COS（`external/cos.rs`，基于 qcos）

- put / get（流式）/ head / delete / list，替换所有现有 `R2_BUCKET.*` 调用
- 上传：`POST /files`（multipart）→ 图审 → COS put
- 下载：`GET /files/*` → 代理流式转发 COS 字节（暂用代理，保持小程序兼容）
- 列表/删除：对应 `fileList.ts`/`fileDelete.ts`
- 配置：`COS_SECRET_ID`、`COS_SECRET_KEY`、`COS_BUCKET`、`COS_REGION`

### 7.4 文件下载决策

**暂用代理下载**：`GET /files/*` 由 Rust 服务读 COS 并流式返回。小程序零改动、URL 兼容、可控。代价是流量过 Rust（自托管可接受）。
**后续优化**：预签名 URL（小程序直连 COS，省 Rust 流量），作为后续优化项，当前不做。

## 8. 配置与密钥

- **配置**（`config` crate）：监听地址/端口、JWT 过期时间、分页默认值、连接参数（非敏感）。
- **密钥**（环境变量，不入库不入 git）：`WX_APPID` `WX_SECRET` `QINIU_ACCESS_KEY` `QINIU_SECRET_KEY` `COS_SECRET_ID` `COS_SECRET_KEY` `COS_BUCKET` `COS_REGION` `JWT_SECRET` `DATABASE_URL`。
- **`.env.example`** 提供模板，`dotenvy` 加载。
- DB 连接池：SeaORM `Database::connect`，配置 pool size。

## 9. 错误处理

- **`AppError` enum**（`error.rs`）：`Internal`(7000/500)、`InputValidation(String)`(7001/400)、`NotFound`(7002/404)、`Unauthorized`(7003/401)、`Forbidden`(7004/403)、`External(String)`(500)。
- **`IntoResponse`**：统一产出 `{ success:false, errors:[{code,message}] }`，复刻现有 Chanfana 全局错误处理器。
- **请求校验**：`validator` 在 handler 入口校验 DTO，失败转 `InputValidation`。
- **panic 兜底**：`tower::catch_panic` 或全局兜底，不暴露堆栈，返回 7000。
- **业务文案**：错误信息带上下文（如 booking 冲突返回"与已有预约冲突 (...) 预约号"），service 层抛 `InputValidation` 带业务文案（移植 `bookingCreate.ts` 三段校验文案）。

## 10. 日志与监控

- **`tracing`** 替代 `console.log`：结构化日志 `[REQ]`/`[RES]`（移植到 `middleware/request_log.rs`）。
- **请求日志中间件**复刻现有：记录 method/url/query/reqBody/status/duration/resBody(截断 500 字)，跳过 `/logs` 自身。
- **`/logs` 实时监控**（移植 `logStore.ts`）：dev 模式下 in-memory 环形缓冲（500 条）+ 长轮询 `waitForLogs`（30s）。生产模式禁用此端点。
- **`/logs` HTML 页**：移植现有 HTML（暗色主题 + 长轮询 JS）。

## 11. 测试策略

对应现有 vitest/miniflare 集成测试，但用**真实 MySQL**：

- **testcontainers-rs** 起隔离 MySQL 容器：跑迁移 → 测试 → 销毁。
- 每个 domain 一个集成测试（对应现有 `tests/integration/*.test.ts`）。
- 关键覆盖：
  - auth：微信 mock → 登录建用户 → JWT 签发校验
  - bookings：**slot 锁定/释放逻辑**（核心，从触发器移到 service 后必须测）+ 时间冲突检测 + 黑名单校验（移植 `bookingCreate.ts` 三段校验）
  - files：upload → moderation block/正常 → COS（mock）
  - 鉴权中间件：token 无效/过期/角色不足各路径
- **外部服务用 mock**：微信/Qiniu/COS 用 trait 抽象 + 测试替身，避免真实调用。

## 12. 部署

- **首选 Docker**：多阶段构建（builder 编译 → distroless/alpine 运行），单二进制。`Dockerfile` + `docker-compose.yml`（含 MySQL）。
- **备选 systemd**：裸机直接跑二进制 + `.service` 单元。
- **配置注入**：环境变量（12-factor），`docker-compose` 用 `env_file`。
- **数据库迁移**：应用启动时跑 `sea-orm-migration migrate`（或独立 `migrate` 二进制）。
- **反代 + TLS**：Rust 服务监听内部端口，nginx/Caddy 反代到 `paidang.hkhy.online` + TLS。

## 13. 风险与缓解

| 风险 | 影响 | 缓解 |
|---|---|---|
| **qcos 签名正确性** | 文件上传/下载全挂 | 实施第一步做冒烟测试；不可用则手写 CAM 签名 fallback |
| **slot 锁定逻辑从触发器移到 service** | 预约时段并发锁定竞态 | service 层事务 + `SELECT ... FOR UPDATE` 保证原子性；集成测试覆盖并发 |
| **微信 access_token 内存缓存** | 多副本部署时各实例缓存不一致 | 当前自托管默认单实例，内存缓存（移植现有逻辑）；多副本时需换 Redis 共享缓存，spec 标注 |
| **无外键关联完整性** | service 层遗漏校验致脏数据 | service 层显式校验父记录存在；级联删除改显式；集成测试覆盖孤儿场景；约定文档 |
| **UTC+8 固定存储** | 跨时区/海外用户时间偏差 | 当前国内业务可接受；spec 标注「不处理 DST，不处理跨时区」 |
| **小程序一次性切 JWT（无灰度）** | 上线瞬间小程序旧版无 token | 当前无用户，可接受；上线需同步小程序发版 |

## 14. 实施里程碑

1. **脚手架 + 配置 + DB schema**：axum + SeaORM + MySQL 迁移（4 SQL 移植，无外键，UTC+8）+ qcos 冒烟测试
2. **鉴权中轴**：JWT 签发/校验中间件 + role guard + 微信登录 + 小程序 `request.ts` 适配
3. **核心业务域**：bookings（含 slot 锁定逻辑）+ date_slots + date_settings + time_slot_templates + booking_logs
4. **内容域**：packages + gallery + gallery_groups
5. **文件域 + 外部集成**：files（qcos + Qiniu 图审 + 微信手机号）
6. **日志/监控 + 部署**：tracing + /logs + Docker + 反代
7. **集成测试补齐 + 切流准备**：testcontainers 覆盖 + 上线验证

每个里程碑独立可测、可部署。

## 15. 不在本期范围

- D1 → MySQL 数据迁移（以后再做）
- COS 预签名 URL 直连下载（后续优化）
- `/kv` 端点（删除）
- 灰度双头鉴权（当前无用户，不做）
- 多副本部署的 Redis 共享缓存（单实例阶段不需要）
- 跨时区 / DST 处理
