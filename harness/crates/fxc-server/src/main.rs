#[cfg(target_os = "windows")]
mod imp;

#[cfg(not(target_os = "windows"))]
mod imp {
    pub fn run() -> color_eyre::Result<()> {
        unimplemented!("fxc-server can only be used on Windows (or with Wine)");
    }
}

fn main() -> color_eyre::Result<()> {
    imp::run()
}
