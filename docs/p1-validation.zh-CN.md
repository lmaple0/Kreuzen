# P1 Crossbell 本地验证记录

验证日期：2026-08-12  
分支：`sky-crossbell-support`  
远端状态：未推送

## 本轮实现

- ED6/ED7 `.clm` 可重新编译为 `._sn` / `.bin`；
- legacy backend 显式接收 CP932 或 GBK codec，并可叠加 `HEX=GLYPH` charmap；
- codec 通过线程作用域安装到 Aureole 读写链，panic 时也会恢复，不依赖
  `CALMARE_RAW_BYTES` 改变整个进程；该环境变量只在 Aureole 中保留为旧调用方的
  兼容回退，Kreuzen 不使用它；
- CLI 支持 legacy 单文件和目录双向处理；`--enc utf8` 对 ED6/ED7 明确报错；
- ED7 布局由 `--legacy-layout native|themelios` 显式指定，不自动探测，也不在
  解析失败后切换布局；`native` 对应 NISA/原 MOD 的原生 Zero/Azure 文件，
  `themelios` 对应现有 Themélios/Calmare 及相关旧汉化工具生成的文件；
- `tools/Test-LegacyCorpus.ps1` 生成逐文件 CSV 与 JSON 报告，分别统计
  `exact`、`different`、`decompile_error`、`compile_error`。

## 自动化测试

- Aureole `cp932`：3 项通过（GBK + charmap 往返、panic 后作用域恢复、
  跨字符边界的映射碰撞拒绝）；
- Kreuzen workspace：38 项通过（33 library、1 CLI、3 legacy、1 syntax）；
- legacy 目录 smoke test：`.bin → .clm → .bin` 成功且 SHA-256 一致。

## 简体中文完整语料

本机 `data_cn` 已安装中文 More Portraits，因此这里验证的是实际兼容语料，
不是未经修改的官方干净语料。

| Profile | 编码 | 文件总数 | 字节一致 | 差异 | 反编译失败 | 重编译失败 |
|---|---:|---:|---:|---:|---:|---:|
| `zero-kai` | GBK | 294 | 294 | 0 | 0 | 0 |
| `azure-kai` | GBK | 355 | 355 | 0 | 0 | 0 |

抽样文件 `c0000.bin` 在两作中也分别完成单文件反编译和重编译，原文件与输出
SHA-256 完全一致。报告保存在本地 `target/p1-corpus-*-cn`，不进入 Git。

## 本机 NISA 日文安装语料

| Profile | 文件总数 | 字节一致 | 差异 | 反编译失败 | 重编译失败 |
|---|---:|---:|---:|---:|---:|
| `zero-kai` (`native`) | 338 | 214 | 124 | 0 | 0 |
| `azure-kai` (`native`) | 384 | 329 | 55 | 0 | 0 |

Zero 原仓库工具以 7 字节 sepith 行成功读取 338/338 个 NISA 场景，证明这不是
Inevitable Zero 构建器私造的布局。Kreuzen 改为在 `native` 模式下按指针读取
Zero 战斗子表，不再把函数与字符串表之间的整个区域猜作连续 sepith 表。
Azure 的字符串表前 0～7 字节零填充也已显式校验和接受。两套语料均实现
338/338、384/384 结构往返；字节差异项属于当前写入器的布局归一化，未冒充
字节一致。所有文件名、SHA-256 和结果均写入本地结构化报告。

这两个安装目录的纯净度尚未得到独立证明，因此这些异常只能作为后续调查清单，
不能直接归因于原版 NISA 数据或 Kreuzen。需要用 Steam 校验后的独立副本复测。

## MOD 专项现状

- Azure Vitality 的 35 个中文 `.clm` 已通过显式 GBK + Themélios 布局编译。
  Kreuzen legacy 入口明确支持 CRLF；`c0200.clm` 的损坏 `Fork flat` 已恢复成
  等价的结构化无限循环，Themélios 同时补上指向函数末尾的标签定义；最终
  35/35 字节往返一致；
- Inevitable Zero 的 39 个中文合并稿已可直接重建为原生 GBK 场景；使用
  `--legacy-layout native` 验证为 36 个字节一致、3 个布局归一化差异、0 个
  反编译或重编译失败；
- 尚未进行游戏内加载、任务流程、分支、DP、奖励和贴图验证。

因此 P1 的核心工具链、中文主语料和两个 MOD 专项结构回归均已完成。尚未进行的
游戏内任务流程验证属于候选包验证，不以静态往返代替。Evo 与旧 PC Zero/Ao
已按项目需求移出 P1 验收范围，现有 profile 仅保留实验性路由，不要求补充样本。
