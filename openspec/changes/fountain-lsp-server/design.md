## 上下文

本项目旨在为 Fountain 剧本写作语言构建一个 LSP（语言服务器协议）Server，提供给 Zed（主要）和 VSCode（兼容）编辑器使用。

**当前状态**：Fountain 语言缺乏原生 LSP 支持，编辑器无法提供智能补全、大纲导航、时长统计等功能。

**约束条件**：
- 使用 Rust 实现
- 依赖 betterfountain-rust 库进行文本解析和时长统计
- 现有 Tree-sitter grammar 存在 bug，需创建新版本
- 需适配 Zed 编辑器（主要目标）
- 开发阶段使用本地路径依赖，完成后切换为远程 Git 依赖

**利益相关者**：剧本写作者、编剧工具开发者

## 目标 / 非目标

**目标**：
- 实现完整的 LSP Server，支持补全、文档符号、悬停、语义高亮功能
- 创建新的完善的 Fountain Tree-sitter grammar
- 实现自动补全（`@` 触发角色名补全）
- 实现文档符号（Outline 面板显示场景标题和时长）
- 实现悬停提示（场景/角色显示上下文信息）
- 提供 Tree-sitter 语法高亮（主要）和 LSP Semantic Tokens（补充）

**非目标**：
- 不实现调试功能
- 不实现代码格式化（Formatter），Fountain 格式本身不需强制格式化
- 不实现 LSP 协议以外的通信方式（如 stdin/stdout 以外的 IPC）

## 决策

### 1. LSP 框架选择

**决策**：使用 `tower-lsp` 作为 LSP 框架

**理由**：
- 纯 Rust 实现，与 betterfountain-rust 同语言，集成方便
- 异步运行时使用 tokio，与 betterfountain-rust 一致
- 社区成熟，文档完善
- 支持 LSP 3.16+ 规范

**替代方案考虑**：
- `lsp-server`：更轻量，但异步支持较弱
- `godel`：已停止维护

### 2. 项目结构

**决策**：采用 workspace 结构，包含两个 crate

```
fountain-lsp/
├── fountain-lsp-core/      # 核心 LSP Server 实现
├── tree-sitter-fountain/   # 新的 Tree-sitter grammar（可选独立）
└── Cargo.toml
```

**理由**：
- 分离关注点：core 负责 LSP 逻辑，tree-sitter 可单独维护
- 便于后续拆分为独立仓库

### 3. Tree-sitter Grammar 实现方案

**决策**：基于 betterfountain-rust 的解析逻辑重新设计 Tree-sitter grammar

**理由**：
- betterfountain-rust 已有完整解析器，覆盖所有 Fountain 元素
- 参考其 token 类型设计 grammar 可确保完整性
- 现有 grammar 的 bug 正是由于解析逻辑不完善

**grammar 节点设计**（完整列表）：

参考 `/Volumes/ssd/Documents/develop/third/betterfountain/syntaxes/fountain.tmLanguage.json` 实现 Tree-sitter grammar：

| Token Type | 说明 | tmLanguage 映射 | 高亮 scope |
|-----------|------|-----------------|-----------|
| `title_page` | 标题页 | `#title_page` | `keyword.title.fountain` |
| `title_page_font` | 字体配置 | `#title_page_font` | `entity.name.tag` |
| `title_page_hidden` | 隐藏配置 | `#title_page_hiden` | `entity.name.tag` |
| `section` | 章节标题 | `#sections` | `support.variable.magic.python` |
| `synopsis` | 概要 | `#synopses` | `constant.numeric.scene.fountain` |
| `scene_heading` | 场景标题 | `#scene_headings_and_action` | `variable.scene.fountain` |
| `scene_number` | 场景编号 | `#scene_headings_and_action` | `constant.numeric.scene.fountain` |
| `transition` | 转场 | `#transitions_and_action` | `variable.transitions.fountain` |
| `transition_to` | TO 转场 | `#transitions_to_and_action` | `variable.transitions.fountain` |
| `centered` | 居中 | `#center_and_action` | `token.info-token` |
| `character` | 角色名 | `#dialogue` | `constant.character.fountain` |
| `character_forced` | 强制角色名 | `#dialogue` | `keyword.operator.fountain` |
| `parenthetical` | 括号动作 | `#dialogue` | `string.parenthetical.fountain` |
| `dialogue` | 对话 | `#dialogue` | `string.fountain markup.italic.fountain` |
| `action` | 动作描述 | `#block_action` | 默认 |
| `action_forced` | 强制动作 | `#action_force` | `keyword.operator.fountain` |
| `note_inline` | 内联注释 | `#notes` | `comment.block.note.fountain` |
| `boneyard` | 注释块 | `#comments` | `comment.block.fountain` |
| `page_break` | 分页符 | `#pagebreaks` | `token.error-token` |
| `lyrics` | 歌词行 | `#lyrics` | `markup.italic string.lyrics.fountain` |
| `underline` | 下划线 | `#underline` | `markup.underline.fountain` |
| `bold` | 粗体 | `#markup` | `markup.bold.fountain` |
| `bold_italic` | 粗斜体 | `#markup` | `markup.bold.italic.fountain` |
| `italic` | 斜体 | `#markup` | `markup.italic.fountain` |

**Tree-sitter Grammar 结构设计**：

```javascript
// grammar.js 核心结构
module.exports = grammar({
  name: 'fountain',
  
  rules: {
    document: $ => repeat($._element),
    
    _element: $ => choice(
      $.title_page,
      $.section,
      $.synopsis,
      $.scene_heading,
      $.transition,
      $.centered,
      $.dialogue_block,
      $.action,
      $.page_break,
      $.lyrics,
      $.note_inline,
      $.boneyard,
      $.separator,
    ),
    
    // 标题页 - 匹配 Title:, Credit:, Author: 等
    title_page: $ => seq(
      /(?i)\s*(title|credit|author|source|draft date|date|contact|copyright|notes|revision)\s*:/,
      $._title_content
    ),
    
    // 章节标题 - 匹配 # ## ### 等
    section: $ => seq(
      /#+/,
      $.section_text
    ),
    
    // 概要 - 匹配 = 开头
    synopsis: $ => seq(
      '=',
      $._line_content
    ),
    
    // 场景标题 - 匹配 . INT. EXT. 等
    scene_heading: $ => seq(
      choice(
        /[.]\s*(?=\w|[（（])/,  // .内景 .外景
        /(?i)(int|ext|est|int[.\/]?ext|i[.\/]?e)[.\s]+/  // INT. EXT. EST.
      ),
      optional($.scene_location),
      optional(seq('-', $.time_of_day)),
      optional(seq('#', $.scene_number, '#'))
    ),
    
    // 转场 - 匹配 > 开头或 TO: 结尾
    transition: $ => choice(
      seq('>', $._line_content),
      seq($.transition_text, 'TO:')
    ),
    
    // 居中 - 匹配 > text <
    centered: $ => seq(
      '>',
      $._center_content,
      '<'
    ),
    
    // 对话块 - 角色名 + 对话
    dialogue_block: $ => seq(
      $.character,
      optional($.parenthetical),
      $.dialogue
    ),
    
    // 角色名
    character: $ => choice(
      seq($.character_name, optional($.character_cue)),  // JOHN (O.S.)
      seq('@', $.character_name)  // @JOHN 强制角色
    ),
    
    // 动作 - 默认类型
    action: $ => $._line_content,
    
    // 强制动作 - ! 开头
    action_forced: $ => seq('!', $._line_content),
    
    // 分页符 - === 连续
    page_break: $ => /={3,}/,
    
    // 歌词 - ~ 开头
    lyrics: $ => seq('~', $._line_content),
    
    // 内联注释 - [[ ]] 
    note_inline: $ => seq('[[', $.note_content, ']]'),
    
    // 注释块 - /* */
    boneyard: $ => seq('/*', $.boneyard_content, '*/'),
    
    // 强调标记
    emphasis: $ => choice(
      $.bold_italic,  // ***text***
      $.bold,         // **text**
      $.italic,       // *text*
      $.underline     // _text_
    ),
    
    _line_content: $ => /[^\n]+/,
    _title_content: $ => /[^\n]+/,
    scene_location: $ => /[^\-#\n]+/,
    time_of_day: $ => choice('日', '夜', 'DAY', 'NIGHT', 'MORNING', 'AFTERNOON', 'EVENING', 'DUSK', 'DAWN'),
    character_name: $ => /\p{Lu}[^\p{Ll}\r\n]*/,
    character_cue: $ => seq('(', /[^)]+/, ')'),
  }
});
```

**Highlights.scm 映射**（参考 tmLanguage 的 name 字段）：

```scheme
// highlights.scm
(title_page_keyword) @keyword

(scene_heading) @variable
(scene_number) @constant
(time_of_day) @type

(character) @constant
(character_forced) @keyword
(parenthetical) @string
(dialogue) @string

(transition) @variable
(transition_to) @variable
(centered) @token

(action) @comment
(action_forced) @keyword

(section) @heading
(section_marker) @keyword

(synopsis) @constant
(page_break) @error
(lyrics) @string

(note_inline) @comment
(boneyard) @comment

(bold) @strong
(italic) @emphasis
(bold_italic) @strong @emphasis
(underline) @underline
```

### 4. 补全触发机制（业务逻辑）

**决策**：基于 VSCode 扩展 Completion.ts 迁移逻辑

参考 `/Volumes/ssd/Documents/develop/third/betterfountain/src/providers/Completion.ts` 中的实现：

| 触发条件 | 补全内容 | 示例 |
|---------|---------|------|
| 输入 `@` | 当前场景角色 + 所有角色 | `@JOHN` |
| 输入 `.` 或`。` 或 `.(` | 场景标题模板 | `.(内景) `, `.(外景) ` |
| 输入 `>` 或`》`| 转场模板 | `>叠化`, `>淡入`, `> <`（居中） |
| 场景标题中输入 `-` | 时间补全 | `日`, `夜`, `DAY`, `NIGHT` |
| 场景标题位置 | 场景位置补全 | 已有位置列表 |
| 角色名后输入 `(` 或 `（` | 角色描述 | `(画外音)`, `(旁白)`, `(O.S.)` |
| 输入 `E` 或 `e` | 场景开头 | `EXT. `, `EST. ` |
| 输入 `I` 或 `i` | 场景开头 | `INT. ` |
| 输入 `【` 或 `[` | note 插入 | `[[ ]]` |
| 输入 `#` | 场景编号变量 | `#${var}#` |
| 标题页区域 | 标题页字段 | `Title:`, `Author:`, `Date:` |
| 输入 `——`  或 `_`| 下划线语法 | `_text_` |

### 4. 编辑器集成方式

**Zed 集成**：
- 创建 Zed extension（Rust + WASM）
- 通过 `zed_extension_api` 的 `language_server_command` 启动 LSP Server
- 使用本地 grammar：`file://` 路径引用

**VSCode 集成**：
- 通过 `languageServerId` 配置指向 LSP Server
- 二进制分发：用户下载编译好的 LSP 二进制并配置路径

### 5. 补全触发机制

**决策**：监听 `@` 字符触发角色名补全

**实现方式**：
- LSP Completion Provider 监听 `triggerCharacters: ["@"]`
- 解析当前文档，收集所有角色名（从 ScriptToken 中提取）
- 提供两种补全模式：
  - **替换模式**：用选中的角色名替换已输入的 `@` + 前缀
  - **续接模式**：在 `@` 后直接插入角色名

### 6. 文档符号与时长显示

**决策**：通过 LSP `textDocument/documentSymbol` 返回层次结构

**参考实现**：基于 VSCode 扩展 Outline.ts（`/Volumes/ssd/Documents/develop/third/betterfountain/src/providers/Outline.ts`）

**Outline 结构**：

| 节点类型 | 说明 | LSP SymbolKind | 显示信息 |
|---------|------|----------------|---------|
| `SceneTreeItem` | 场景标题 | `SymbolKind.Class` | 时长 [Xm Ys] |
| `DialogueTreeItem` | 角色对白 | `SymbolKind.Variable` | 时长 [Xm Ys] |
| `SectionTreeItem` | 章节标题 | `SymbolKind.Namespace` | 时长 [Xm Ys]，支持多层级（#-#####） |
| `NoteTreeItem` | 书签/注释 | `SymbolKind.Constant` | 注释内容 |
| `SynopsisTreeItem` | 概要 | `SymbolKind.Interface` | 概要文本 |
| `NoteRootTreeItem` | 根节点 | `SymbolKind.Module` | 分类（NOTES/Bookmarks） |

**实现方式**：
- 基于 betterfountain-rust 的 `StructToken` 树结构构建 DocumentSymbol 层次
- 根节点：剧本标题（来自 title_page）
- 子节点：场景标题（Scene Heading）
- 叶子节点：角色对白���
- 每个节点的 `detail` 字段显示时长（如 "2m 30s"）
- 支持与光标位置关联的自动展开（reveal）

**数据结构映射**：
```
StructToken (from betterfountain-rust)
├── text: 节点文本
├── line: 行号 -> LSP range.start.line
├── section: 是否为章节标题
├── isscene: 是否为场景
├── ischartor: 是否为角色对白
├── isnote: 是否为注释
├── isBookmark: 是否为书签
├── durationSec: 时长（秒）
├── children: 子节点列表
├── notes: 注释列表
└── synopses: 概要列表
```

### 7. 依赖管理

**决策**：开发阶段使用 path 依赖，完成后切换为 git 依赖

**理由**：
- 本地开发调试方便
- betterfountain-rust 已在 GitHub 维护，可直接切换为远程依赖

## 风险 / 权衡

- **Zed 闭源且更新频繁** → 需持续关注 Zed API 变化，建议在 extension 中添加版本检查
- **Tree-sitter grammar 开发工作量** → 可分阶段完成，先实现基本高亮，后续迭代完善
- **LSP 性能** → Fountain 文件通常较小，性能应可控；但需注意解析缓存策略
- ** Zed vs VSCode 差异** → 某些 LSP 功能在 Zed 中支持度不同（如 Semantic Tokens 默认关闭），需做好兼容性检测

- **grammar 测试覆盖** → 需构建完善的测试用例库，覆盖边界情况（空文件、多重嵌套、特殊字符等）