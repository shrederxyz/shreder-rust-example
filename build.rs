use tonic_build::configure;

fn main() {
    const PROTOC_ENVAR: &str = "PROTOC";
    if std::env::var(PROTOC_ENVAR).is_err() {
        #[cfg(not(windows))]
        std::env::set_var(PROTOC_ENVAR, protobuf_src::protoc());
    }

    configure()
        .compile(
            &["proto/shredstream.proto", "proto/shreder_binary.proto"],
            &["proto"],
        )
        .unwrap();

    let preconf_out_dir =
        std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("preconf");
    std::fs::create_dir_all(&preconf_out_dir).unwrap();

    configure()
        .out_dir(preconf_out_dir)
        .compile(&["proto/shreder_preconf.proto"], &["proto"])
        .unwrap();
}
