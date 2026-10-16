# React 迁移对照

这里保存迁移前的 React 实现、类型配置和依赖清单，仅供审查差异。生产入口是 `web/src/main-leptos.js` 与 `web/leptos/src`，默认构建和验收不会执行这些文件。原 JavaScript 单元用例对应的请求、分页、图表和时间语义已迁至 Rust 原生用例与实际 WASM 浏览器验收。
