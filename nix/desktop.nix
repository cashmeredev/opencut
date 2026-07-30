# Electron desktop app: tsc-compiled main process + the static web export,
# wrapped with nixpkgs electron (no electron-builder, no asar).
# ffmpeg-static is satisfied by nixpkgs ffmpeg.
{
  lib,
  stdenvNoCC,
  bun,
  nodejs,
  callPackage,
  electron,
  ffmpeg,
  makeWrapper,
  sources ? import ../npins,
}:

let
  bunDeps = callPackage ./bun-deps.nix { };
  web = callPackage ./web.nix { inherit sources; };
  desktopPkg = builtins.fromJSON (builtins.readFile ../apps/desktop/package.json);
in
stdenvNoCC.mkDerivation {
  pname = "opencut-desktop";
  version = desktopPkg.version;
  src = lib.cleanSource ../.;

  nativeBuildInputs = [
    bun
    nodejs
    makeWrapper
  ];

  configurePhase = ''
    runHook preConfigure
    cp -r ${bunDeps}/node_modules node_modules
    cp -r ${bunDeps}/apps/desktop/node_modules apps/desktop/node_modules
    chmod -R u+w node_modules apps/desktop/node_modules
    patchShebangs node_modules apps/desktop/node_modules
    runHook postConfigure
  '';

  buildPhase = ''
    runHook preBuild
    export HOME=$TMPDIR
    cd apps/desktop
    bun run build
    cd $NIX_BUILD_TOP/source
    runHook postBuild
  '';

  installPhase = ''
    runHook preInstall

    appdir=$out/share/opencut
    mkdir -p $appdir/node_modules/ffmpeg-static
    cp apps/desktop/package.json $appdir/package.json
    cp -r apps/desktop/dist $appdir/dist
    cp -r ${web} $appdir/web

    # main.js resolves ffmpeg via require("ffmpeg-static"); point it at nixpkgs ffmpeg.
    cp apps/desktop/node_modules/ffmpeg-static/index.js $appdir/node_modules/ffmpeg-static/index.js
    cp apps/desktop/node_modules/ffmpeg-static/package.json $appdir/node_modules/ffmpeg-static/package.json
    ln -s ${lib.getExe ffmpeg} $appdir/node_modules/ffmpeg-static/ffmpeg

    makeWrapper ${lib.getExe electron} $out/bin/opencut \
      --add-flags "$appdir"

    mkdir -p $out/share/applications
    cat > $out/share/applications/opencut.desktop <<EOF
    [Desktop Entry]
    Type=Application
    Name=OpenCut
    Comment=Offline-first video editor
    Exec=opencut
    Categories=AudioVideo;Video;
    EOF

    runHook postInstall
  '';

  meta = {
    description = "OpenCut desktop (Electron)";
    license = lib.licenses.mit;
    mainProgram = "opencut";
    platforms = lib.platforms.linux;
  };
}
