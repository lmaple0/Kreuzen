# Kreuzen 中文说明

Kreuzen 是 Falcom 场景脚本的反编译与重编译工具。本维护分支在原有闪之轨迹
I–IV、创之轨迹和东亰幻都支持之外，接入了空之轨迹与零碧轨迹使用的 ED6/ED7
后端。

当前经过本地完整语料静态验证的范围：

- 空之轨迹 SC（PC）：709 个 `._SN` 文件全部可往返，709 个二次源码稳定；
- 空之轨迹 the 3rd（PC）：368 个 `._SN` 文件全部字节一致；
- 空之轨迹 FC（PC 汉化版）：三套各 491 个 `._SN` 文件全部可往返，1473 次验证
  均无解析、编译或二次解析错误，源码全部稳定；
- 零之轨迹、碧之轨迹 PC/Kai：详见 `docs/p1-validation.zh-CN.md`；
- Sky Evo/Kai：未在当前阶段验证。

这里的“通过”是脚本静态读取、重编译和二次反编译验证，不等于已经完成游戏内
运行测试。

FC 样本来自本机合法安装的汉化版，但该安装同时含语音、EVO 音乐等其他 MOD，
因此只能证明这些实际脚本副本可稳定往返，不能当作干净零售版基线。

## Sky FC / SC / the 3rd 用法

汉化版通常需要显式指定 GBK。建议始终把输出写入独立目录，不要直接覆盖游戏：

```powershell
# 反编译一个 SC 文件
.\kreuzen.exe --game sky-sc --enc gbk --output .\work\C5313.clm '.\input\C5313   ._SN'

# 重编译；游戏类型也会记录在 .clm 内
.\kreuzen.exe --game sky-sc --enc gbk --output '.\work\C5313   ._SN' .\work\C5313.clm

# 处理整个 the 3rd 脚本副本
.\kreuzen.exe --game sky-3rd --enc gbk --output .\work\3rd-clm .\input\ED6_DT21
```

`.clm` 目录和 `._SN` 目录不能混在一起批量处理。工具不会在 GBK/SJIS 之间自动
切换；编码错误会直接报告，以便定位真正的问题。`--legacy-layout` 只影响 ED7，
不影响 Sky 的 ED6 文件。

## 安全验证流程

1. 从游戏目录复制待改 `._SN` 到独立输入目录；
2. 反编译到另一个目录；
3. 修改 `.clm` 后重编译到第三个目录；
4. 再次反编译编译产物，比较两份 `.clm`；
5. 只将确认后的输出作为独立 MOD 或备份副本进行游戏内测试。

不要把“命令成功”当作运行时验证，也不要把汉化或其他 MOD 混合后的本机文件
当作干净原版基线。

## 故障定位

- `Could not detect game`：安装目录名无法识别，请显式传入 `--game sky-sc` 或
  `--game sky-3rd`；
- `Invalid SJIS string`：中文补丁通常应显式使用 `--enc gbk`；
- GBK 解码失败：不要自动改用 SJIS；先确认文件来源和是否需要 `--charmap`；
- `Found both clm and binary`：源文件与二进制混在同一批处理目录，请分开；
- 字节哈希不同：继续做第二次反编译比较。源码稳定只说明结构规范化稳定，仍需
  在游戏副本中验证。

完整计划与边界见 `docs/sky-crossbell-support-plan.zh-CN.md`，Sky 验证数据见
`docs/p2-validation.zh-CN.md` 和 `docs/p4-sky-fc-validation.zh-CN.md`。
