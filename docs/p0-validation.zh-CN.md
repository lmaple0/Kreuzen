# P0 本地验证记录

验证日期：2026-08-12  
分支：`sky-crossbell-support`  
远端状态：未推送

## 自动化测试

- `cargo test --workspace`：通过；
- 现有 Kreuzen 库测试：33 项通过；
- 现有 Kreuzen Syntax 测试：1 项通过；
- 新增 legacy profile 测试：1 项通过；
- 新增 CLI 安装路径检测测试：1 项通过。

上游 Themélios/Calmare 在当前 nightly 上可编译，但自身存在既有 warning；P0
没有修改这些 sibling dependency 的源码。

## 真实脚本 smoke test

| Profile | 输入 | 结果 | 输出 SHA-256 |
|---|---|---|---|
| `sky-sc` | `Trails in the Sky SC/DAT/ED6_DT21/A0019   ._SN` | 自动识别 ED6，成功生成 `.clm` | `ECC09CFC7D728782E1EFB9CEDE1A154EF92C7FF820FBE9411E22E4FB98105146` |
| `zero-kai` | `Trails from Zero/data/scena/a0003.bin` | 自动识别 ED7，成功生成 `.clm` | `EA5DA9B45D8AE263842A44757C68DEB2A94419149B9BEC6B595F54502F69572B` |
| `sky-3rd` | `Trails in the Sky the 3rd/data/ED6_DT21/a0028._sn` | 显式 profile 成功生成 `.clm` | 未作为 P0 必选样本登记 |

SC 的 `A0000._SN` 在 CP932 模式下失败，内容表现为已安装汉化产生的其他编码。
这不是静默忽略项：P0 只承诺 CP932 只读路径，显式 GBK/charmap codec 属于 P1。

## Corpus manifest

manifest 只保存在工作区临时验证目录，不进入 Git：

- Sky SC：709 个 `._SN` 文件；
- ZeroKai 日文场景：338 个 `.bin` 文件；
- manifest 仅记录相对路径、大小和 SHA-256，不复制脚本内容。

## P0 边界

已完成：游戏 profile、modern/ED6/ED7 路由、legacy crate、CP932 只读反编译、
安装路径自动识别和 corpus manifest 工具。

尚未完成且不计入 P0：`.clm` 重编译、批量目录处理、显式 legacy codec、GBK、
charmap、完整 corpus 往返和游戏内验证。
