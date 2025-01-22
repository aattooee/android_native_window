use std::env;
use std::fs;
use std::path::Path;
use reqwest::blocking::get;

fn main() {
    // 设置目标文件名和下载地址
    let lib_name = "libnative-window-control.a";
    let url = "https://github.com/aattooee/android_native_control_support/releases/download/android14-supported/libnative-window-control.a";
    let target_dir = env::var("OUT_DIR").unwrap(); // 获取 Cargo 输出目录
    let target_path = Path::new(&target_dir).join(lib_name);

    // 检查目标文件是否存在，如果不存在则下载
    if !target_path.exists() {
        // 发送 HTTP 请求下载文件
        let mut response = get(url).expect("Failed to download file");
        let mut out_file = fs::File::create(&target_path).expect("Failed to create file");

        // 将下载的内容写入目标文件
        std::io::copy(&mut response, &mut out_file).expect("Failed to write data to file");

        println!("Downloaded library to: {:?}", target_path);
    }

    // 告诉 Cargo 在哪里查找库文件
    println!("cargo:rustc-link-search=native={}", target_dir);

    println!("cargo:rustc-link-lib=static=native-window-control");

    // 如果你需要根据文件变化重新编译，可以添加此行
    println!("cargo:rerun-if-changed=build.rs");
}
