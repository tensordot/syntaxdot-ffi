{ defaultCrateOverrides
, lib
, symlinkJoin

# Native build inputs
, cmake
, installShellFiles
, protobuf
, removeReferencesTo

# Build inputs
, libtorch-bin
}:

defaultCrateOverrides // {
  prost-build = attrs: {
    PROTOC = "${protobuf}/bin/protoc";
  };

  sentencepiece-sys = attrs: {
    nativeBuildInputs = [ cmake ];

    postInstall = ''
      # Binaries and shared libraries contain references to /build,
      # but we do not need them anyway.
      rm -f $lib/lib/sentencepiece-sys.out/build/src/spm_*
      rm -f $lib/lib/sentencepiece-sys.out/build/src/*.so*
    '';

  };

  syntaxdot-ffi = attr: {
    buildInputs = [ libtorch-bin ];
  };

  torch-sys = attr: {
    LIBTORCH = libtorch-bin.dev;
  };
}
