# P3 Sky PC 统一体验验证记录

验证日期：2026-08-12

当前范围仅包括 Sky SC 与 Sky the 3rd PC。Sky FC、Evo 和 Kai 不在本阶段计划内。

## 可复现依赖

`kreuzen-legacy` 不再要求本机存在 `../../Aureole`。Calmare、Themélios 和 cp932
固定到 `lmaple0/Aureole` 提交 `9563e4e`，`Cargo.lock` 记录完整 Git revision。
原 modern backend 使用的 `../../sjis` 与 `../../Gospel` 也分别固定到公开仓库提交
`8bf7d1d` 和 `9f89bce`，干净检出不再读取任何兄弟源码目录。

该 Aureole 提交必须先存在于公开远端，Kreuzen 的公开构建才能解析依赖。当前
Aureole 根目录仍未发现许可证文件，因此本地成功构建不等于已经满足二进制发布
授权条件；许可证/来源问题解决前不发布 Release。

## Windows 构建

执行：

```powershell
.\tools\Build-Windows.ps1
```

结果：

- 产物：`out/kreuzen-windows-x64/kreuzen.exe`；
- 大小：7,209,984 字节；
- SHA-256：`313F5317514B4F7F2A0B1D5234E374F45FE4CC4EF0A5C3C15FAB0F6F0C4BA329`；
- 同目录包含英文 README、中文 README 和 P2 验证记录；
- 构建使用 `cargo build --locked --release -p kreuzen-cli`。

该文件是本地验证产物，未创建 GitHub Release。

## Release EXE 副本验证

测试只操作 `target/p3-runtime-20260812` 下的输入、源码、重编译和二次源码目录，
没有写入 Steam 游戏目录。

| 游戏 | 样本 | 二进制一致 | 二次源码稳定 | 说明 |
|---|---|---|---|---|
| Sky SC | `C5313   ._SN` | 否 | 是 | 保留异常 flat 控制流；writer 规范化 |
| Sky the 3rd | `a0028._sn` | 是 | 是 | SHA-256 完全一致 |

SC 原始 SHA-256：
`AC8509EDA976BE8D5F0D63EEBAA6ED452D38282CE96BB85FC04EA0600EC6C2A3`

SC 重编译 SHA-256：
`A4101871AD500B18DF97A12C3958A6E40BA813D728EFBC0A71D1E0D004954216`

the 3rd 原始与重编译 SHA-256：
`D2C1A8316B7FA30C7E286A89CF5F137956C625F3D2CEE4A2F735ED36EC8AB784`

## 尚未完成

- 未把重编译脚本加载到实际游戏进程；
- 未验证对话触发、场景切换、存档读写等运行时行为；
- 未解决 Aureole 二进制再分发的许可证/来源问题；
- 未发布 GitHub Release。
