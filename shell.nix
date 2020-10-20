with import <nixpkgs> {};

let
  sources = import ./nix/sources.nix;
  libtorch = libtorch-bin;
  rust-toolchain = pkgs.symlinkJoin {
    name = "rust-toolchain";
    paths = [ pkgs.rustc pkgs.cargo pkgs.rustPlatform.rustcSrc ];
  };
in mkShell {
  nativeBuildInputs = [
    rustup
    pkgconfig
    protobuf
  ];

  buildInputs = [
    libtorch
    openssl
    python3
    sentencepiece
  ];



  LIBTORCH = "${libtorch.dev}";
  PROTOC = "${protobuf}/bin/protoc";
}
