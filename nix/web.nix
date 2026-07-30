# Static export of the Next.js web app (apps/web/out).
# Deps come from the bun-deps FOD; the Google-font fetch in next/font is
# replaced by the npins-pinned Inter so the build runs offline.
{
  lib,
  stdenvNoCC,
  bun,
  nodejs,
  callPackage,
  sources ? import ../npins,
}:

let
  bunDeps = callPackage ./bun-deps.nix { };
  webPkg = builtins.fromJSON (builtins.readFile ../apps/web/package.json);
in
stdenvNoCC.mkDerivation {
  pname = "opencut-web";
  version = webPkg.version;
  src = lib.cleanSource ../.;

  nativeBuildInputs = [
    bun
    nodejs
  ];

  postPatch = ''
    mkdir -p apps/web/public/fonts
    cp ${sources.inter}/web/InterVariable.woff2 apps/web/public/fonts/InterVariable.woff2
    substituteInPlace apps/web/src/app/layout.tsx \
      --replace-fail 'import { Inter } from "next/font/google";' 'import localFont from "next/font/local";' \
      --replace-fail 'const siteFont = Inter({ subsets: ["latin"] });' 'const siteFont = localFont({ src: "../../public/fonts/InterVariable.woff2" });'
  '';

  configurePhase = ''
    runHook preConfigure
    cp -r ${bunDeps}/node_modules node_modules
    cp -r ${bunDeps}/apps/web/node_modules apps/web/node_modules
    chmod -R u+w node_modules apps/web/node_modules
    patchShebangs node_modules apps/web/node_modules
    runHook postConfigure
  '';

  buildPhase = ''
    runHook preBuild
    export HOME=$TMPDIR
    export NEXT_TELEMETRY_DISABLED=1
    export NODE_ENV=production
    cd apps/web
    bun run build
    runHook postBuild
  '';

  installPhase = ''
    runHook preInstall
    cp -r out $out
    runHook postInstall
  '';

  meta = {
    description = "OpenCut web app (static export)";
    license = lib.licenses.mit;
  };
}
