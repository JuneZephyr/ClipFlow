# 在 Windows 上安装 ClipFlow

ClipFlow 是一个普通的 Windows 程序，但它必须在 Windows 电脑上打包；在 Mac 上做好的 DMG 不能直接给 Windows 用。
Mac 和 Windows 的程序格式不同，这一步没法在 Mac 上完成。下面是最省事的做法。

## 需要的东西（都免费）

1. 一台 Windows 10 或 Windows 11 电脑。
2. Rust：打开 https://rustup.rs ，下载 `rustup-init.exe`，一路点默认选项安装。
   Windows 会顺带提示安装 “Visual Studio Build Tools”，请按提示安装（只勾选默认的 C++ 桌面开发即可）。
3. `app` 这个文件夹（整个拷到 Windows 上，放在 Documents 里就行）。

## 打包（大约 1–5 分钟）

1. 打开 `app\windows` 文件夹。
2. 双击 `build-windows.bat`，等窗口里出现 “Done”。
3. 会得到 `app\windows\ClipFlow-windows.zip`。
   把它解压到任意位置，双击里面的 `ClipFlow.exe` 就能用了。

第一次打开时 Windows 可能提示 “已保护你的电脑”，点 “更多信息 → 仍要运行”。
这是因为程序是在本机打包的、没有购买数字签名证书，内容是安全的。

## 做成安装程序（可选）

如果想要“下一步、下一步”的安装界面，可以再装免费的 Inno Setup（https://jrsoftware.org/isinfo.php），
用它把 `windows\out\ClipFlow.exe` 包成一个 `ClipFlow-Setup.exe`。这一步不影响程序本身的使用。

## 自动打包（可选，适合放到 GitHub）

如果以后把项目放到 GitHub，可以在仓库里新建 `.github/workflows/build-windows.yml`，
内容使用 `app/windows/github-actions-build-windows.yml`，之后每次推送都会自动生成 Windows 版。

## 说明

- 记录和设置保存在 `%LOCALAPPDATA%\ClipFlow\state.json`，只在这台电脑上。
- 界面默认是英文，可以在左侧“设置（Settings）→ 语言”里切换成中文。
- AI 只有在你填了地址和密钥后才会工作；不填也能正常记剪贴板。
- 本次在 Mac 上已做好并在 Windows 上可用的部分：文件和设置路径、系统字体、剪贴板读取、截图模式。
  真正的 Windows 安装包需要按上面的步骤在那台电脑上生成，并顺手打开试一下。
