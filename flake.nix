{
  description = "SyntaxDot FFI";

  inputs = {
    crate2nix = {
      url = "github:kolloch/crate2nix";
      flake = false;
    };
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, crate2nix, nixpkgs, utils }:
    utils.lib.eachSystem  [ "x86_64-linux" ] (system:
    let
      pkgs = import nixpkgs {
        inherit system;
        config = {
          allowUnfreePredicate = pkg: builtins.elem (pkgs.lib.getName pkg) [
            "libtorch"
          ];
        };
      };
    in {
      devShell = with pkgs; mkShell {
        nativeBuildInputs = [ cmake pkg-config rustup ];

        buildInputs = [ openssl ];

        LIBTORCH = symlinkJoin {
          name = "torch-join";
          paths = [ libtorch-bin.dev libtorch-bin.out ];
        };

        PROTOC = "${protobuf}/bin/protoc";
      };

      defaultPackage = let
        crateOverrides = pkgs.callPackage build-support/crate-overrides.nix {};
        crateTools = pkgs.callPackage "${crate2nix}/tools.nix" {};
        buildRustCrate = pkgs.buildRustCrate.override {
          defaultCrateOverrides = crateOverrides;
        };
        cargoNix = pkgs.callPackage (crateTools.generatedCargoNix {
          name = "syntaxdot";
          src = ./.;
        }) {
          inherit buildRustCrate;
        };
        crate = cargoNix.rootCrate.build;
      in with pkgs; stdenv.mkDerivation {
        pname = "libsyntaxdot-ffi";
        version = crate.version;

        src = ./.;

        outputs = [ "out" "dev" ];

        nativeBuildInputs = [ removeReferencesTo ];

        dontConfigure = true;

        installPhase = ''
          runHook preInstall

          install -Dm 0644 -t $dev/include include/syntaxdot.h
          install -Dm 0755 -t $out/lib ${crate.lib}/lib/libsyntaxdot_ffi.so

          remove-references-to -t ${libtorch-bin.dev} \
            $out/lib/libsyntaxdot_ffi.so

          runHook postinstall
        '';

        disallowedReferences = [ libtorch-bin.dev ];

        meta = with lib; {
          description = "C FFI for the SyntaxDot neural syntax annotator";
          license = licenses.agpl3Only;
          maintainers = with maintainers; [ danieldk ];
          platforms = platforms.linux;
        };
      };
    });
}
