# copy this file to directory with your configuration.nix and add this to configuration.nix
# environment.systemPackages = [
#   (pkgs.callPackage ./nix-simple-deploy.nix { })
# ];
{ stdenv, fetchFromGitHub, rustPlatform }:
rustPlatform.buildRustPackage rec {
  pname = "nix-simple-deploy";
  src = ./.;
  version = "0.1.1";
  cargoSha256 = "1v0xga8bnxlgn23gphprrpyl38a2l2f6si8zfph532pgckx8d10s";
  verifyCargoDeps = true;
  meta = with stdenv.lib; {
    description = "Deploy software or an entire NixOS system configuration to another NixOS system";
    homepage = "https://github.com/misuzu/nix-simple-deploy";
    license = with licenses; [ mit ];
    platforms = platforms.all;
    maintainers = with maintainers; [ misuzu ];
  };
}
