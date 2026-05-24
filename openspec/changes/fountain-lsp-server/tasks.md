## 1. 项目初始化

- [x] 1.1 创建 Rust workspace 项目结构（fountain-lsp-core crate）
- [x] 1.2 配置 Cargo.toml，添加 tower-lsp、betterfountain-rust 依赖
- [x] 1.3 设置本地路径依赖 betterfountain-rust（开发阶段）
- [x] 1.4 配置日志系统和错误处理
- [x] 1.5 初始化 LSP Server 基础框架（tower-lsp）

## 2. LSP 基础能力

- [x] 2.1 实现 LSP 协议初始化（initialize/shutdown）
- [x] 2.2 实现文本文档同步（didOpen/didChange/didClose）
- [x] 2.3 配置 LSP Server capabilities
- [x] 2.4 添加解析缓存机制，提高性能

## 3. 文本补全功能（Completion）

- [x] 3.1 实现 @ 触发角色名补全
- [x] 3.2 实现场景标题补全（. -> 内景/外景）
- [x] 3.3 实现转场补全（> -> CUT TO/叠化/淡入等）
- [x] 3.4 实现时间补全（- 后输入：日/夜/DAY/NIGHT）
- [x] 3.5 实现角色描述补全（( -> 画外音/旁白/O.S.）
- [x] 3.6 实现场景位置补全（INT./EXT. 后）
- [x] 3.7 实现标题页字段补全（Title:/Author:/Date:）
- [x] 3.8 实现章节标题补全（#）
- [x] 3.9 实现 note 内联注释补全（【）
- [x] 3.10 实现转场居中语法补全（> <）
- [x] 3.11 实现场景编号变量补全（#${}#）
- [x] 3.12 实现下划线语法补全（——）
- [x] 3.13 实现 E/e -> EXT./EST. 补全
- [x] 3.14 实现 I/i -> INT. 补全

## 4. 文档符号功能（Document Symbols）

- [ ] 4.1 实现 textDocument/documentSymbol 处理器
- [ ] 4.2 解析剧本结构，生成 DocumentSymbol 层次
- [ ] 4.3 实现剧本标题作为根节点
- [ ] 4.4 实现场景标题作为子节点
- [ ] 4.5 实现角色对白块作为叶子节点
- [ ] 4.6 在 detail 字段添加时长信息（格式：2m 30s）
- [ ] 4.7 实现增量更新支持

## 5. 悬停提示功能（Hover）

- [ ] 5.1 实现 textDocument/hover 处理器
- [ ] 5.2 实现场景标题悬停（显示时长、位置、时间）
- [ ] 5.3 实现角色名悬停（显示说话次数、场景数、词数）
- [ ] 5.4 实现对话内容悬停（显示时长、所属角色）
- [ ] 5.5 实现动作描述悬停（显示类型、行号）
- [ ] 5.6 实现转场文本悬停（显示转场类型）
- [ ] 5.7 返回 Markdown 格式内容

## 6. 语法高亮功能

### 6.1 Tree-sitter Grammar
- [x] 6.1.1 创建 tree-sitter-fountain grammar 项目
- [x] 6.1.2 实现 scene_heading 节点解析
- [x] 6.1.3 实现 character 节点解析
- [x] 6.1.4 实现 dialogue 节点解析
- [x] 6.1.5 实现 parenthetical 节点解析
- [x] 6.1.6 实现 action 节点解析
- [x] 6.1.7 实现 transition 节点解析
- [x] 6.1.8 实现 title_page 节点解析
- [x] 6.1.9 实现 section 节点解析（# 开头的章节标题）
- [x] 6.1.10 实现 synopsis 节点解析（= 开头的概要）
- [x] 6.1.11 实现 boneyard 节点解析（/* */ 注释块）
- [x] 6.1.12 实现 note_inline 节点解析（[[ ]] 内联注释）
- [x] 6.1.13 实现 centered 节点解析（> <）
- [x] 6.1.14 实现 page_break 节点解析（===）
- [x] 6.1.15 实现 lyrics 节点解析（~ 开头）
- [ ] 6.1.16 实现 scene_number 节点解析（#1#）
- [x] 6.1.17 实现 separator 节点解析（---）
- [ ] 6.1.18 编写 grammar 测试用例

### 6.2 Highlights.scm
- [ ] 6.2.1 创建 highlights.scm 定义高亮规则
- [ ] 6.2.2 配置场景标题高亮（@title）
- [ ] 6.2.3 配置角色名高亮（@type）
- [ ] 6.2.4 配置对话高亮（@string）
- [ ] 6.2.5 配置括号动作高亮（@emphasis）
- [ ] 6.2.6 配置动作描述高亮（@comment）
- [ ] 6.2.7 配置转场高亮（@keyword）
- [ ] 6.2.8 配置标题页高亮（@property）
- [ ] 6.2.9 配置章节标题高亮（@heading）

### 6.3 LSP Semantic Tokens
- [ ] 6.3.1 实现 textDocument/semanticTokens/full
- [ ] 6.3.2 定义 token 类型映射
- [ ] 6.3.3 实现增量更新支持

## 7. 编辑器集成

### 7.1 Zed Extension
- [ ] 7.1.1 创建 Zed extension 项目结构
- [ ] 7.1.2 实现 extension.toml 配置
- [ ] 7.1.3 配置 Fountain 语言（languages/fountain/）
- [ ] 7.1.4 集成 tree-sitter-fountain grammar
- [ ] 7.1.5 配置 language_server_command 启动 LSP
- [ ] 7.1.6 测试 Zed 集成

### 7.2 VSCode Extension
- [ ] 7.2.1 创建 VSCode extension 项目结构
- [ ] 7.2.2 配置 package.json
- [ ] 7.2.3 配置 languageServerId 指向 LSP Server
- [ ] 7.2.4 测试 VSCode 集成

## 8. 依赖管理

- [ ] 8.1 开发阶段使用本地 path 依赖
- [ ] 8.2 验证 LSP 功能正常工作
- [ ] 8.3 切换为远程 Git 依赖（https://github.com/ArmingLou/betterfountain-rust.git）
- [ ] 8.4 更新 Cargo.lock

## 9. 测试与验证

- [ ] 9.1 编写单元测试覆盖核心解析逻辑
- [ ] 9.2 编写集成测试覆盖 LSP 协议
- [ ] 9.3 测试补全功能各种触发场景
- [ ] 9.4 测试文档符号返回正确结构
- [ ] 9.5 测试悬停提示内���正确性
- [ ] 9.6 测试语法高亮效果
- [ ] 9.7 性能测试（10000 行文档）

## 10. 构建与分发

- [ ] 10.1 配置 release build
- [ ] 10.2 交叉编译支持 macOS/Windows/Linux
- [ ] 10.3 创建 Zed extension 发布流程
- [ ] 10.4 创建 VSCode extension 发布流程
- [ ] 10.5 编写 README 文档