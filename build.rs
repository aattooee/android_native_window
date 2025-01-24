use std::env;
use std::path::Path;
use std::process::Command;
fn main() {

    let project_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    let lib_source_dir = Path::new(&project_dir).join("android_native_control_support/");
    let lib_dir = Path::new(&project_dir).join("android_native_control_support/build/");
    let lib_path = lib_dir.join("libnative-window-control.a");
    

    // 检查目标文件是否存在，如果不存在则构建
    if !lib_path.exists() {
        //使用脚本进行构建
        let _output = Command::new("bash")
        .current_dir(lib_source_dir)
        .arg("build.sh")
        .output()
        .expect("Failed to build cxx deps");
    }

    // 告诉 Cargo 在哪里查找库文件
    println!("cargo:rustc-link-search=native={}", lib_dir.to_str().unwrap());


    // 如果你需要根据文件变化重新编译，可以添加此行
    println!("cargo:rerun-if-changed=build.rs");
}
