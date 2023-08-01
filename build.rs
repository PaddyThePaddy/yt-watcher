fn main() {
    println!("yt watcher build script started");
    //println!("cargo:rerun-if-changed=www/src");
    let pwd = std::env::current_dir().unwrap();
    if std::env::consts::OS == "windows" {
        println!("cargo:rerun-if-changed=www\\src");
        assert!(std::process::Command::new("npm.cmd")
            .arg("install")
            .current_dir(pwd.join("www/"))
            .status()
            .expect("Build vue project failed.")
            .success());
        assert!(std::process::Command::new("npm.cmd")
            .arg("run")
            .arg("build")
            .current_dir(pwd.join("www/"))
            .status()
            .expect("Build vue project failed.")
            .success());
    } else {
        println!("cargo:rerun-if-changed=www/src");
        assert!(std::process::Command::new("npm")
            .arg("install")
            .current_dir(pwd.join("www/"))
            .status()
            .expect("Build vue project failed.")
            .success());
        assert!(std::process::Command::new("npm")
            .arg("run")
            .arg("build")
            .current_dir(pwd.join("www/"))
            .status()
            .expect("Build vue project failed.")
            .success());
    }
    println!("yt-watcher build script finished");
}
