# copy this file to directory with your configuration.nix and add this to configuration.nix
# environment.systemPackages = [
#   (pkgs.callPackage ./nix-simple-deploy.nix { })
# ];
{ lib, fetchFromGitHub, rustPlatform, makeWrapper, openssh, nix-serve }:
rustPlatform.buildRustPackage rec {
  pname = "nix-simple-deploy";
  version = "0.2.0";

  src = ./.;

  cargoSha256 = "1n6q962lbrlmj7p2i290jww65a0s9xckf1pnqyn43fpx4a9ibqaa";

  nativeBuildInputs = [ makeWrapper ];

  postInstall = ''
    wrapProgram "$out/bin/nix-simple-deploy" \
      --prefix PATH : "${lib.makeBinPath [ openssh nix-serve ]}"
  '';

  meta = with lib; {
    description = "Deploy software or an entire NixOS system configuration to another NixOS system";
    homepage = "https://github.com/misuzu/nix-simple-deploy";
    license = with licenses; [ asl20 /* OR */ mit ];
    maintainers = with maintainers; [ misuzu ];
  };
}
