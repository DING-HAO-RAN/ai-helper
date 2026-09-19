use ai_helper_lib::{app, cli, storage};

fn main() {
    // 启动时在程序所在文件夹下初始化创建配置、存储与文件索引缓存文件
    storage::init_app_workspace();

    let args: Vec<String> = std::env::args().collect();
    // 优先拦截命令行请求，毫秒级响应外部工具与 AI agent
    if cli::handle_cli_args(&args) {
        return;
    }

    // 若无命令行参数，说明是作为 GUI 桌面应用启动：
    // 在 Windows 下释放控制台黑框窗口，确保界面无黑框干扰
    #[cfg(target_os = "windows")]
    unsafe {
        windows_sys::Win32::System::Console::FreeConsole();
    }

    // 启动赛博朋克桌面主程序与悬浮球管理运行时
    app::run_desktop_app();
}
