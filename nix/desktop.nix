{ lib, appimageTools }:

let
  manifest = lib.importJSON ../apps/desktop/package.json;
  pname = "opencut";
  version = manifest.version;
  src = ../apps/desktop/release/OpenCut-${version}-linux-x86_64.AppImage;
  appimageContents = appimageTools.extract { inherit pname version src; };
in
appimageTools.wrapType2 {
  inherit pname version src;

  extraInstallCommands = ''
    install -Dm444 ${appimageContents}/opencut.png $out/share/icons/hicolor/512x512/apps/opencut.png
    install -Dm444 ${appimageContents}/OpenCut.desktop $out/share/applications/opencut.desktop
    substituteInPlace $out/share/applications/opencut.desktop \
      --replace-fail 'Exec=AppRun --no-sandbox %U' 'Exec=opencut %U'
  '';

  meta = {
    description = "Offline-first video editor";
    homepage = manifest.homepage;
    license = lib.licenses.mit;
    mainProgram = "opencut";
    platforms = [ "x86_64-linux" ];
  };
}
