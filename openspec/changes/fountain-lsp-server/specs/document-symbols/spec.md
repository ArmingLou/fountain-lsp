## 新增需求

### 需求:文档符号查询
LSP Server 必须提供文档符号查询功能，返回 Fountain 文档的层次结构。

#### 场景:返回剧本标题
- **当** 客户端发送 `textDocument/documentSymbol` 请求
- **那么** LSP Server 必须返回剧本标题页中的标题作为顶级 DocumentSymbol

#### 场景:返回场景标题
- **当** 客户端发送 `textDocument/documentSymbol` 请求
- **那么** LSP Server 必须返回文档中所有场景标题（Scene Heading）作为子级 DocumentSymbol

#### 场景:返回角色对白块
- **当** 客户端发送 `textDocument/documentSymbol` 请求
- **那么** LSP Server 必须返回每个角色及其对白作为子级 DocumentSymbol

#### 场景:符号层级结构
- **当** 客户端发送 `textDocument/documentSymbol` 请求
- **那么** 返回的 DocumentSymbol 必须形成层次结构：剧本标题 → 场景标题 → 角色对白

### 需求:时长信息显示
每个 DocumentSymbol 必须在 detail 字段中包含对应的时长信息。

#### 场景:场景时长显示
- **当** 返回场景标题的 DocumentSymbol
- **那么** detail 字段必须包含该场景的时长（如 "2m 30s"）

#### 场景:角色台词时长显示
- **当** 返回角色对白的 DocumentSymbol
- **那么** detail 字段必须包含该角色台词的总时长（如 "45s"）

#### 场景:总时长显示
- **当** 返回剧本标题的 DocumentSymbol
- **那么** detail 字段必须包含剧本的总时长（如 "1h 23m"）

### 需求:符号位置映射
DocumentSymbol 必须正确映射到文档中的位置。

#### 场景:范围精确
- **当** 返回任意 DocumentSymbol
- **那么** 必须包含正确的 `range` 字段，指向文档中的确切位置

#### 场景:选择范围
- **当** 用户在 Outline 面板中点击某个符号
- **那么** 编辑器必须跳转到该符号在文档中的位置

### 需求:增量更新
当文档内容发生变化时，文档符号必须能够增量更新。

#### 场景:文档变更通知
- **当** 客户端发送 `textDocument/didChange` 通知
- **那么** LSP Server 必须更新其内部缓存的符号信息

#### 场景:增量请求响应
- **当** 客户端在文档变更后发送 `textDocument/documentSymbol` 请求
- **那么** LSP Server 必须返回更新后的符号列表

## 修改需求

（无）

## 移除需求

（无）