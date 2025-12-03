{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
    buildInputs = [
        pkgs.openssl
    ];
    nativeBuildInputs = [
        pkgs.pkg-config
        pkgs.cargo
    ];
}
