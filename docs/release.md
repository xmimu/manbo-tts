# 发布流程

## 前置条件

- 代码已在 `dev` 分支开发完毕并测试通过
- `Cargo.toml`、`tauri.conf.json`、`App.vue` 中的版本号已同步更新
- `README.md` 更新日志已补充本次变更内容

## 版本号更新位置

每次发布前需同步修改以下三处：

| 文件 | 字段 |
|------|------|
| `src-tauri/Cargo.toml` | `version` |
| `src-tauri/tauri.conf.json` | `version` |
| `src/App.vue` | 标题栏 `brand-version` 文字 |

## 发布步骤

```bash
# 1. 切换到 master 并合并 dev
git checkout master
git merge dev

# 2. 推送 master
git push origin master

# 3. 打 tag（与版本号一致，加 v 前缀）
git tag v0.1.2
git push origin v0.1.2
```

推送 tag 后，GitHub Actions 自动触发构建。

## CI 构建流程

`.github/workflows/release.yml` 在 `windows-latest` 上执行：

1. 安装 Node.js 20 + pnpm 9
2. `pnpm install` 安装前端依赖
3. 安装 stable Rust 工具链
4. `tauri-action` 构建并打包
5. 自动创建 GitHub Release，上传 `.msi` 和 `.exe`

首次构建因 Rust 依赖缓存未建立，耗时约 20~30 分钟；后续有缓存约 10~15 分钟。

构建进度可在仓库 **Actions** 标签页查看。

## 发布后

- 确认 GitHub Releases 页面安装包可正常下载
- 将 `master` 的变更同步回 `dev`：

```bash
git checkout dev
git merge master
git push origin dev
```
