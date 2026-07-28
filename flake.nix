{
  description = "OpenCut Classic development shell";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { nixpkgs, ... }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
      ];
      forEachSystem = nixpkgs.lib.genAttrs systems;
    in
    {
      devShells = forEachSystem (
        system:
        let
          pkgs = import nixpkgs { inherit system; };
        in
        {
          default = pkgs.mkShell {
            packages = with pkgs; [
              bun
              just
              nodejs

              cargo
              clippy
              rustc
              rustfmt

              cmake
              pkg-config

              alsa-lib
              fontconfig
              freetype
              libxkbcommon
              openssl
              vulkan-loader
              wayland
              libX11
              libxcb

              ffmpeg-headless.dev
              llvmPackages.libclang
            ];

            env.LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";

            shellHook = ''
              export BINDGEN_EXTRA_CLANG_ARGS="$(cat ${pkgs.stdenv.cc}/nix-support/cc-cflags 2>/dev/null) $(cat ${pkgs.stdenv.cc}/nix-support/libc-cflags 2>/dev/null) $BINDGEN_EXTRA_CLANG_ARGS"
              export LD_LIBRARY_PATH=${
                pkgs.lib.makeLibraryPath (
                  with pkgs; [
                    alsa-lib
                    fontconfig
                    freetype
                    libxkbcommon
                    vulkan-loader
                    wayland
                    libX11
                    libxcb
                  ]
                )
              }:$LD_LIBRARY_PATH
            '';
          };
        }
      );
    };
}
