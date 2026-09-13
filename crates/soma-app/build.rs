//! Tauri 构建脚本：编译期读取 tauri.conf.json，内嵌前端资源并生成上下文代码。

fn main() {
    tauri_build::build()
}
