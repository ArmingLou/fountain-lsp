## 为什么

当前没有为 Fountain 剧本写作语言提供原生 LSP（语言服务器协议）支持的工具。编辑器（如 Zed、VSCode）在编辑 `.fountain` 文件时缺乏智能补全、大纲导航、时长统计等开发体验增强功能。betterfountain-rust 已具备完整的解析、outline 结构生成和时长统计能力，现在需要基于这些能力构建一个标准的 LSP Server，让编辑器能够为剧本写作者提供专业级的编辑体验。

## 变更内容

- 新建一个基于 Rust 的 Fountain LSP Server 项目
- 依赖 betterfountain-rust 库实现 Fountain 文本解析、结构分析和时长统计
- 实现以下 LSP 功能：
  - **文本补全（Completion）**：用户输入触发字符（如 `@`）时提供角色名等自动补全选项，支持替换和续接两种补全方式
  - **文档符号（Document Symbols）**：提供标题、场景标题（Scene Heading）及对应时长统计的大纲结构，通过 LSP `textDocument/documentSymbol` 自动对接 Zed/VSCode 的 Outline 面板，在每个符号项的 detail 字段显示时长（如 "2m 30s"）
  - **语法高亮（Syntax Highlighting）**：
  - **Tree-sitter（主要方式）**：创建新的 Fountain Tree-sitter grammar（现有版本 `UserNobody14/tree-sitter-fountain` 存在 bug 不完善），通过 `highlights.scm` 定义高亮规则，自动生效，无需用户配置
  - **LSP Semantic Tokens（补充）**：基于解析结果提供语义级别的语法高亮作为增强，用户需在设置中启用 `"semantic_tokens": "combined"`
  - **悬停提示（Hover）**：用户将鼠标悬停在特定元素上时显示上下文相关信息，如场景显示时长和位置信息，角色显示台词统计（说话次数、出现场景数等）

> **注意**：LSP 协议本身不支持状态栏渲染功能。Zed 编辑器不支持通过 LSP 显示状态栏信息。替代方案是通过 Document Symbols 在 Outline 面板中展示时长统计。
- 优先适配 Zed 编辑器，同时兼容 VSCode
- 开发阶段使用本地路径依赖 betterfountain-rust，完成后切换为远程 Git 依赖

## 功能 (Capabilities)

### 新增功能

- `completion`: 文本补全功能 — 监听触发字符（如 `@` 触发角色名补全），提供上下文相关的补全项列表，支持替换已有文本和续接补全两种模式
- `document-symbols`: 文档符号/大纲功能 — 基于 betterfountain-rust 的 StructToken 树构建 LSP DocumentSymbol 层次结构，通过 `textDocument/documentSymbol` 自动对接 Zed/VSCode 的 Outline 面板，在 `detail` 字段显示各节点的时长（如 "2m 30s"），提供直观的剧本时长分布视图
- `syntax-highlighting`: 语法高亮功能 — 包含两种方式：① Tree-sitter 高亮（主要），需创建新的 Fountain Tree-sitter grammar（现有版本存在 bug 不完善），通过 `highlights.scm` 定义规则自动生效，无需用户配置；② LSP Semantic Tokens 作为增强，用户需在设置中启用 `"semantic_tokens": "combined"`，基于 betterfountain-rust 的 ScriptToken 类型信息（scene_heading、character、dialogue、action 等）提供更精细的高亮
- `hover`: 悬停提示功能 — 根据光标所在 token 的类型显示上下文信息，包括：场景标题显示时长和位置信息（INT./EXT.、日/夜），角色显示台词统计信息（说话次数、出现场景数、台词词数），其他元素显示对应的 token 类型和文本摘要

### 修改功能

（无，本项目为全新项目）

## 影响

- **新建项目**：`/Volumes/ssd/Documents/develop/self/fountain-lsp` 为全新 Rust 项目
- **依赖**：betterfountain-rust（本地路径 `../../third/betterfountain-rust`，后期切换为 `https://github.com/ArmingLou/betterfountain-rust.git`）
- **LSP 协议**：需遵循 LSP 3.16+ 规范，使用 `tower-lsp` 或类似框架实现 LSP 通信
- **编辑器适配**：Zed（主要）和 VSCode（兼容），两者均支持 LSP 的补全、悬停、文档符号和语义高亮功能。状态栏时长显示需由编辑器 extension 层面实现，LSP 无法直接推送
- **构建与分发**：需考虑 Zed extension 和 VSCode extension 的集成方式及二进制分发策略
