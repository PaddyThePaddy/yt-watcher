fn main() {
    println!("yt watcher build script started");
    //println!("cargo:rerun-if-changed=www/src");
    let pwd = std::env::current_dir().unwrap();
    std::process::Command::new("npm.cmd")
        .arg("run")
        .arg("build")
        .current_dir(pwd.join("www/"))
        .status()
        .expect("Build vue project failed.");
    println!("yt-watcher build script finished");
}
