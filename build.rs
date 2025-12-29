// Windows 资源文件配置
fn main() {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set_manifest_file("app.manifest");
        res.set("ProductName", "宁夏特检院质量监督检查抽签程序");
        res.set("FileDescription", "质量监督检查随机抽签工具");
        res.set("LegalCopyright", "宁夏特检院 © 2024-2025");
        res.set("ProductVersion", "1.0.0");
        res.set("FileVersion", "1.0.0");
        res.set("CompanyName", "宁夏特检院");
        res.set("OriginalFilename", "quality_draw.exe");
        
        // 如果有图标文件，取消下面这行的注释
        // res.set_icon("assets/app.ico");
        
        if let Err(e) = res.compile() {
            eprintln!("winres compile error: {}", e);
        }
    }
}
