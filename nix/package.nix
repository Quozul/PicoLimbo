{
  lib,
  rustPlatform,
}:

rustPlatform.buildRustPackage {
  pname = "picolimbo";
  version = "1.12.2+mc26.1.2";

  src = lib.cleanSource ../.;

  cargoLock.lockFile = ../Cargo.lock;

  cargoBuildFlags = [
    "--bin"
    "pico_limbo"
  ];

  meta = with lib; {
    description = "A lightweight Minecraft limbo server written in Rust";
    homepage = "https://github.com/Quozul/PicoLimbo";
    license = licenses.mit;
    mainProgram = "pico_limbo";
    platforms = [
      "x86_64-linux"
      "aarch64-linux"
      "aarch64-darwin"
    ];
  };
}
