{ pkgs ? (import <nixpkgs> {}) }:

pkgs.mkShell rec {
    name = "libical";

    buildInputs = with pkgs; [
      rustc
      cargo
      cmake
      curl
      gcc
      libpsl
      openssl
      pkgconfig
      which
      zlib
      dbus
      libtool
      libical
      llvmPackages.libclang
      clang
    ];

    LIBCLANG_PATH="${pkgs.llvmPackages.libclang}/lib";
}


