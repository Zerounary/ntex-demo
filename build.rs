fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("failed to locate vendored protoc");
    unsafe {
        std::env::set_var("PROTOC", protoc);
    }

    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .compile_protos(
            &["proto/nodepanel.proto", "proto/node_control.proto"],
            &["proto"],
        )
        .expect("failed to compile protos");

    println!("cargo:rerun-if-changed=proto/nodepanel.proto");
    println!("cargo:rerun-if-changed=proto/node_control.proto");
}
