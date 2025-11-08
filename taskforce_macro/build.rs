fn main() {
    if cfg!(target_os = "windows") {
        println!("cargo:rerun-if-changed=icon.ico");
        println!("cargo:rerun-if-changed=icon.rc");
        embed_resource::compile("icon.rc");
    }
}
