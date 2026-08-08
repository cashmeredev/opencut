let
  sources = import ./npins;
  pkgs = import sources.nixpkgs { };
in
rec {
  packages.web = pkgs.callPackage ./nix/web.nix { inherit sources; };
  packages.desktop = pkgs.callPackage ./nix/desktop.nix { };
  packages.default = packages.desktop;
}
