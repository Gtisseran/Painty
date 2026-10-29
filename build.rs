fn main() {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/painty.ico");
        res.compile().expect("impossible d'intégrer l'icône");
    }
}