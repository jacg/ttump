{
  description = "Table Tennis Upmire's Clock";

  inputs = {

    # Version pinning is managed in flake.lock. Upgrading can be done with
    # something like
    #
    #    nix flake lock --update-input nixpkgs

    nixpkgs     .url = "github:nixos/nixpkgs/nixos-24.05";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils .url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:

    # Option 1: try to support each default system
    flake-utils.lib.eachDefaultSystem # NB Some packages in nixpkgs are not supported on some systems

      # Option 2: try to support selected systems
      # flake-utils.lib.eachSystem ["x86_64-linux" "i686-linux" "aarch64-linux" "x86_64-darwin"]
      (system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [
              # ===== Specification of the rust toolchain to be used ====================
              rust-overlay.overlays.default (final: prev:
                { rust-tools = final.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml; }
              )
            ];
          };

          show-image-support = [
            pkgs.libGL
            pkgs.libxkbcommon
            pkgs.xorg.libX11
            pkgs.xorg.libXcursor
            pkgs.xorg.libXi
            pkgs.xorg.libXrandr
            #pkgs.vulkan-loader
          ];

        in
          {
            devShell = pkgs.mkShell {
              name = "my-rust-project";
              buildInputs = [
                pkgs.rust-tools
                pkgs.cargo-nextest
                pkgs.cargo-watch
                pkgs.bacon
                pkgs.just

                pkgs.trunk
                pkgs.openssl
                pkgs.pkg-config

                pkgs.fontconfig

                #pkgs.wayland

              ];
              packages = [
                pkgs.lolcat
                pkgs.eza
              ];
              shellHook =
                ''
                  export PS1="TT umpire devenv> "
                  alias foo='cowsay Foo'
                  alias bar='eza -l | lolcat'
                  alias baz='cowsay What is the difference between buildIntputs and packages? | lolcat'
                '';
              # Requires "rust-src" to be present in components in ./rust-toolchain.toml
              RUST_SRC_PATH = "${pkgs.rust-tools}/lib/rustlib/src/rust/library";
              LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath show-image-support;
            };
          }
      );
}
