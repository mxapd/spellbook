{ lib
, rustPlatform
, pkg-config
, makeWrapper
, openssl
, libX11
, libxcb
, libxkbcommon
, wayland
, wl-clipboard
, xclip
, libnotify
, procps
, bash
, coreutils
, sudo
}:

rustPlatform.buildRustPackage {
  pname = "spellbook";
  version = "1.0.0";

  src = lib.cleanSource ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  nativeBuildInputs = [
    pkg-config
    makeWrapper
  ];

  buildInputs = [
    openssl
    libX11
    libxcb
    libxkbcommon
    wayland
  ];

  postInstall = ''
    wrapProgram $out/bin/spellbook \
      --prefix PATH : ${lib.makeBinPath [
        wl-clipboard
        xclip
        libnotify
        procps
        bash
        coreutils
        sudo
      ]}
  '';

  meta = {
    description = "TUI application for managing and executing CLI command snippets";
    homepage = "https://github.com/mxapd/spellbook";
    license = lib.licenses.mit;
    mainProgram = "spellbook";
  };
}
