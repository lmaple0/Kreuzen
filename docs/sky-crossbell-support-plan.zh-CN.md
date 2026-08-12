# Sky / Crossbell 支持实施计划

状态：P0、P1 已完成；P2 的 SC / the 3rd 静态 corpus 验收已完成；FC 已移出当前计划

工作分支：`main`
分支治理：fork 的新维护线为默认 `main`；原上游历史保存在 `legacy`

## 1. 目标与边界

目标是在 `lmaple0/Kreuzen` 中维护一条独立开发线，让同一个命令行工具能够处理：

- 空之轨迹 FC / SC / the 3rd（ED6）；
- 零之轨迹 / 碧之轨迹（ED7）；
- Kreuzen 已支持的闪之轨迹、创之轨迹和东京幻都脚本。

首要目标是可靠的场景脚本反编译、编辑和重编译。归档、图片、语音、字体及游戏资源表不纳入第一阶段；它们只在场景脚本验证确实依赖时单独接入。

不把“能够解析一个文件”等同于游戏支持完成。每个游戏/版本只有通过批量往返验证，并记录无法字节往返的例外后，才能在支持列表中标为完成。

## 2. 当前调查结论

### 2.1 Kreuzen 现状

当前 `kreuzen::Scena`、文件头解析、表块、指令规格和 `.krz` 语法是按 CS1 之后的脚本族设计的。Sky/Crossbell 与其拥有不同的文件头、指针宽度、场景实体表、函数表和指令集，不能通过单纯扩充 `Game` 枚举实现。

现有 GBK/charmap 实现仍有价值，但必须下沉为可被现代和旧引擎后端共同使用的文本编解码上下文，不能继续依赖全局环境变量。

### 2.2 可复用实现

本地 `Aureole` 代码已经包含：

- `themelios/src/scena/ed6.rs`：ED6 场景二进制读写；
- `themelios/src/scena/ed7.rs`：ED7 场景二进制读写；
- `themelios-scena/src/code/insn.rs`：FC/SC/3rd、Zero/Ao 及 Evo/Kai 变体的指令规格；
- `calmare`：ED6/ED7 的 `.clm` 文本打印和解析；
- 游戏模型：`Fc/FcEvo/FcKai`、`Sc/ScEvo/ScKai`、`Tc/TcEvo/TcKai`、`Zero/ZeroEvo/ZeroKai`、`Ao/AoEvo/AoKai`。

因此第一版不应重写 ED6/ED7。应先把 Themélios/Calmare 作为 legacy backend 接入 Kreuzen，再逐步消除旧实现中的全局状态和不安全编码捷径。

### 2.3 当前样本条件

| 游戏族 | 本地样本 | 已知限制 |
|---|---|---|
| Zero NISA/Kai | `data/scena`、`data/scena_us`、`data_cn/scena` | `data_cn` 已混入中文补丁和 More Portraits，不能当原始基线 |
| Azure NISA/Kai | `data/scena`、`data/scena_us`、`data_cn/scena` | 同上 |
| Zero/Azure Evo | 已解密资源位于 `D:\idm_download\ed7_kiseki_evo_psv` | 可用于格式验证；繁中自定义字符映射需单独验证 |
| Sky SC | 本机已安装 | 已安装汉化/语音等修改，不是干净原版 |
| Sky the 3rd | 本机已安装 | 需要从归档中只读提取场景样本 |
| Sky FC | 当前未发现本机安装 | 在宣称 FC 完成前必须补充合法测试样本 |

游戏完整脚本不进入仓库。测试语料使用本机路径清单和 SHA-256 manifest；仓库内仅保留人工构造、足够小的格式 fixture。

## 3. 推荐架构

采用“双后端、单入口”，暂不强行统一两套 AST 或文本语法：

```text
kreuzen-cli
  ├─ modern backend: 现有 kreuzen（CS1+，.dat ↔ .krz）
  └─ legacy backend: ED6/ED7 adapter（._sn/.bin ↔ .clm）
```

建议新增 `kreuzen-legacy` crate，职责限定为：

1. 将 CLI 的游戏/版本/编码参数转换为 Themélios/Calmare 类型；
2. 提供 ED6、ED7 场景的读、写、打印和解析入口；
3. 使用显式 `TextCodec`，支持 CP932、GBK、UTF-8、charmap；
4. 把旧后端错误转换为 Kreuzen 的统一诊断；
5. 不让 legacy 类型渗入现有 modern backend。

第一版保留 `.clm`，因为它已经覆盖 ED6/ED7 全量结构。立即把所有旧脚本转换成 `.krz` 会同时引入格式移植和语法移植两种风险，难以判断错误来源。等二进制往返稳定后，再决定是否设计统一源格式。

开发期可先使用本地 `Aureole` path dependency。公开推送前必须决定可复现依赖方式，并确认旧仓库代码的许可证/授权和来源说明；当前本地 Aureole 根目录未发现许可证文件，因此不应未经确认直接复制大段源码进 Kreuzen。

## 4. 游戏与版本命名

CLI 使用清楚、稳定的名字，并为旧 Calmare 缩写保留兼容别名：

| 正式名称 | 兼容别名 |
|---|---|
| `sky-fc` / `sky-fc-evo` / `sky-fc-kai` | `fc` / `fc_e` / `fc_k` |
| `sky-sc` / `sky-sc-evo` / `sky-sc-kai` | `sc` / `sc_e` / `sc_k` |
| `sky-3rd` / `sky-3rd-evo` / `sky-3rd-kai` | `tc` / `tc_e` / `tc_k` |
| `zero` / `zero-evo` / `zero-kai` | `zero` / `zero_e` / `zero_k` |
| `azure` / `azure-evo` / `azure-kai` | `ao` / `ao_e` / `ao_k` |

不根据 `data_cn`、`data_us` 自动推断游戏版本、编码或 ED7 二进制布局。目录命名在
不同汉化补丁中含义不一致。布局通过 `--legacy-layout native|themelios` 显式指定；
解析失败时不自动切换，避免掩盖真正的结构错误。

## 5. 分阶段实施

### P0：基线与适配层骨架

- 保留现有 modern backend 的全部测试；
- 新增 `EngineFamily::{LegacyEd6, LegacyEd7, Modern}` 或等价路由类型；
- 新增 `kreuzen-legacy` crate 和只读 decompile 最小入口；
- 将游戏名称、文件扩展名和输出扩展名集中到一个 profile 表；
- 建立本地 corpus manifest 工具，只记录相对分类、大小和 SHA-256；
- 记录旧 Aureole 依赖的来源、提交和发布前授权检查项。

验收：现有 34 项测试无回归；至少一个 ED6 和一个 ED7 文件能被正确路由并生成 `.clm`，但此时不宣称支持完成。

### P1：Crossbell 主线

状态（2026-08-12）：核心读写、显式 CP932/GBK/charmap、目录处理和结构化
corpus 报告已完成；本机简体中文 `data_cn/scena` 的 ZeroKai 294 个文件、
AoKai 355 个文件均达到逐字节往返一致。NISA PC 的 ZeroKai 338 个与 AoKai
384 个文件也已实现无解析/编译失败的结构往返，非字节一致项保留在报告中。
Inevitable Zero 与 Azure Vitality 专项均已完成静态结构回归，因此 P1 工具链验收完成。按当前项目需求，
Evo 和旧 PC Zero/Ao 不再作为 P1 验收条件；相关 profile 仅保留实验性路由。

优先顺序：`ZeroKai` → `AoKai`。Evo 与旧 PC `Zero/Ao` 不列入本阶段验收。

- 接通 ED7 二进制读写和 `.clm` 编译；
- 将 GBK/charmap 改造成 legacy backend 可显式传入的 codec；
- 替换 `CALMARE_RAW_BYTES` 全局开关；raw-byte 模式仅保留为诊断/恢复手段；
- 覆盖 NISA PC 日文、英文和中文兼容文件；
- 对 Zero/Azure 分别生成成功、失败、非字节往返清单；
- 用 Inevitable Zero / Azure Vitality 实际涉及的场景文件做重点回归。

验收：目标目录批量反编译成功；反编译→重编译按文件比较；所有差异都有结构化报告，不允许静默丢文本、指令或表项。

### P2：Sky PC 主线

状态（2026-08-12）：本机汉化版 SC 的 709 个文件与 the 3rd 的 368 个文件
均完成反编译、重编译和二次反编译稳定性验证，解析/编译/二次解析错误均为 0。
SC 有 472 个字节一致、237 个规范化差异，但 709 个文件的源码均在第二轮稳定；
the 3rd 为 368 个全部字节一致。按当前维护范围，FC 暂时忽略，不作为 P2/P3
完成条件，也不把 SC/3rd 的结果外推为 FC 已完成。详见 `p2-validation.zh-CN.md`。

当前范围：`Sky SC` → `Sky the 3rd`。Sky FC 暂不推进。

- 接通 ED6 `._sn` 场景格式；
- 用 Factoria 或现有只读工具提取归档中的场景测试集，绝不覆盖游戏安装目录；
- 验证英文/日文/现有中文补丁所需编码和字体映射；
- 建立 SC 与 the 3rd 各自的指令差异和非往返例外表。

验收：SC 与 the 3rd 分别独立通过 corpus 测试；FC 不计入当前验收。

### P3：Sky PC 统一体验

状态（2026-08-12）：不再补 FC 或 Sky Evo/Kai。公开构建依赖已固定到 Aureole
验证提交；CLI 帮助、中文 README、故障诊断和 Windows 构建脚本已完成。release
EXE 已在 SC 与 the 3rd 独立脚本副本上完成编译/重载验证，详见
`p3-validation.zh-CN.md`。尚未进行游戏进程内验证或发布 Release。

- 完善路径/可执行文件名检测，但不覆盖显式 `--game`；
- 评估 `.clm` 与 `.krz` 是否值得统一；
- 补齐 CLI 帮助、中文 README、迁移说明和故障诊断；
- 构建 Windows 可执行文件并进行真实副本上的编译/重载测试。

## 6. 文本编码原则

- codec 必须作为参数沿读写调用链传递，禁止通过环境变量改变全进程行为；
- GBK 是编码，charmap 是游戏字体槽映射，两者分开建模；
- 繁转简/OpenCC 属于本地化内容转换，不属于脚本格式解析，不默认启用；
- 未映射字符、映射前缀冲突和保留字节碰撞必须报错；
- 对 PSV 繁中资源，保留原文字符和字符映射来源，不用猜测替换。

## 7. 测试与完成标准

每个游戏/版本至少包含以下检查：

1. 二进制读取成功；
2. AST/文本打印成功；
3. 打印结果重新解析成功；
4. 重编译二进制成功；
5. 原文件与重编译文件做 SHA-256 和逐字节比较；
6. 非字节一致时，对二次反编译结果做结构比较并登记原因；
7. 中文、日文、英文和自定义字库字符各有覆盖；
8. 对输出副本做游戏内关键流程验证。

静态解析、哈希往返和游戏内验证分别报告，不能互相替代。

## 8. 首轮实施清单（已完成）

P0 已按以下顺序完成：

1. 给 CLI 抽出游戏 profile/后端路由，不改变现有行为；
2. 加入 `kreuzen-legacy` 空壳和 path dependency 原型；
3. 选取 Sky SC 与 ZeroKai 各一个只读样本建立 smoke test；
4. 接通只读反编译；
5. 再接重编译和 corpus runner；
6. 完成 P0 验收后才进入中文 codec 改造。

## 9. Git 与发布约束

- 日常开发与发布来源固定为 `main`；原上游历史保留在 `legacy`；
- 不再创建 `agent/*` 或旧的临时开发分支；
- P0/P1 未完成前不创建 Release；
- 推送前先整理提交，使 GBK/charmap、legacy adapter、Crossbell、Sky 各自可审查和回退；
- 不再以原上游合并为完成条件，fork 的构建、文档和测试必须能够独立成立。
