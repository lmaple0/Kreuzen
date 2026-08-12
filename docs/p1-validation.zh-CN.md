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
- `tools/Test-LegacyCorpus.ps1` 生成逐文件 CSV 与 JSON 报告，分别统计
  `exact`、`different`、`decompile_error`、`compile_error`。

## 自动化测试

- Aureole `cp932`：3 项通过（GBK + charmap 往返、panic 后作用域恢复、
  跨字符边界的映射碰撞拒绝）；
- Kreuzen workspace：37 项通过（33 library、1 CLI、2 legacy、1 syntax）；
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
| `zero-kai` | 338 | 334 | 0 | 4 | 0 |
| `azure-kai` | 384 | 329 | 8 | 47 | 0 |

Zero 的 4 个失败项为 `r0000.bin`、`r1000.bin`、`r1500.bin`、`r2050.bin`，
均是越界读取。Azure 的失败项主要报 `incorrect sepith table size`，另有 8 个
文件能往返但字节不同。所有文件名、SHA-256 和错误信息已写入本地结构化报告，
没有静默跳过。

这两个安装目录的纯净度尚未得到独立证明，因此这些异常只能作为后续调查清单，
不能直接归因于原版 NISA 数据或 Kreuzen。需要用 Steam 校验后的独立副本复测。

## MOD 专项现状

- Azure Vitality 仓库现有 35 个中文 `.clm` 来自旧
  `CALMARE_RAW_BYTES=1` 流程，不能直接当作新的 Unicode + GBK `.clm` 输入；
  需要一次明确的迁移，不能静默套用；
- Inevitable Zero 的 39 个中文合并稿是 Python 脚本格式，不是 `.clm`，需要从其
  构建输出或合并器边界接入专项回归；
- Evo、旧 PC Zero/Ao 尚无本轮可确认的独立干净样本；
- 尚未进行游戏内加载、任务流程、分支、DP、奖励和贴图验证。

因此本轮完成了 P1 的核心工具链与中文主语料无损往返，但 P1 仍保持进行中，
直到上述样本和 MOD 专项清单完成。
