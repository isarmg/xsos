# Leptos 业务数据强类型化

本次实际接入 DTO 的路径：实例列表/详情、激活前设备核对、主机统计、创建和重命名请求。原来在这些组件之间传递的 `Value` 已换为结构体，UI 直接访问字段。

`models.rs` 是浏览器使用字段的 projection。完整遥测、硬件报告、日志等仍沿用现有严格 `Value` 校验与展示，避免引入依赖服务端运行时的 model，也不重新定义现有硬件报告协议。统计以 u64 保存，nullable 的监控错误显式为 Option。

所有 typed loader/mutation 都先调用原来的 `contracts::validate` / `valid_mutation_response`，再构造 DTO。保留路径与查询参数绑定、revision、页数/条数限制、nullable、未知键策略及整数边界。`models.rs` 不是新增服务端协议。原认证、取消和轮询生命周期沿用原实现；响应不合法继续映射 `platform.invalid_response`。

结构化请求经过 `encode_json_body`，错误显式显示且不发送。分页支持 typed page 的 `PageCursors`（适用时），不再为分页把整页 DTO 转回 JSON；cursor 状态读取借用页面，只复制 cursor 字符串，避免克隆整页列表。

原生校验命令：

```sh
CARGO_BUILD_JOBS=2 cargo test --manifest-path web/leptos/Cargo.toml --lib --offline
```

新增单测覆盖实际 DTO 与既有严格守卫组合，而不是仅测试 derive 能否反序列化。浏览器回归与发布构建由本轮总体验证记录统一列出；本文件不将原生测试等同于浏览器验收，也不声称存在未测量的性能加速。
