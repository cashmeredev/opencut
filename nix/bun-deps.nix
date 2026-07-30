# Fixed-output derivation: workspace node_modules from bun.lock.
# Only re-runs when the lockfile or manifests change; the hash covers the
# installed dependency tree, so it must be updated whenever bun.lock changes
# (set outputHash = lib.fakeHash, build once, paste the reported hash).
{
  lib,
  runCommand,
  bun,
  cacert,
}:

let
  minimalSrc = runCommand "opencut-bun-src" { } ''
    mkdir -p $out/apps/web $out/apps/desktop
    cp ${../package.json} $out/package.json
    cp ${../bun.lock} $out/bun.lock
    cp ${../bunfig.toml} $out/bunfig.toml
    cp ${../.npmrc} $out/.npmrc
    cp ${../apps/web/package.json} $out/apps/web/package.json
    cp ${../apps/desktop/package.json} $out/apps/desktop/package.json
  '';
in
runCommand "opencut-bun-deps"
  {
    nativeBuildInputs = [
      bun
      cacert
    ];
    outputHashMode = "recursive";
    outputHashAlgo = "sha256";
    outputHash = "sha256-+zRGGTHFoM9d4wjP999J3JdcRgNksTTVOb+nceyvNak=";
  }
  ''
    cp -r ${minimalSrc} src
    chmod -R u+w src
    cd src

    export HOME=$TMPDIR
    export BUN_INSTALL_CACHE_DIR=$TMPDIR/bun-cache
    # The electron binary is unused; nixpkgs electron is used at runtime.
    export ELECTRON_SKIP_BINARY_DOWNLOAD=1

    bun install --frozen-lockfile

    mkdir -p $out/apps/web $out/apps/desktop
    cp -r node_modules $out/node_modules
    cp -r apps/web/node_modules $out/apps/web/node_modules
    cp -r apps/desktop/node_modules $out/apps/desktop/node_modules
  ''
