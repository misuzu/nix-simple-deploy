# copy this file to directory with your configuration.nix and add this to configuration.nix
# environment.systemPackages = [
#   (pkgs.callPackage ./nix-simple-deploy.nix { })
# ];
{ stdenv, fetchFromGitHub, rustPlatform }:
rustPlatform.buildRustPackage rec {
  pname = "nix-simple-deploy";
  version = "0.1.1";
  src = fetchFromGitHub {
    owner = "misuzu";
    repo = pname;
    rev = version;
    sha256 = "12g0sbgs2dfnk0agp1kagfi1yhk26ga98zygxxrjhjxrqb2n5w80";
  };
  cargoSha256 = "02v8lrwjai45bkm69cd98s5wlvq8w5yz4wfzf7bjcv6n61k05n6f";
  verifyCargoDeps = true;
  meta = with stdenv.lib; {
    description = "Deploy software or an entire NixOS system configuration to another NixOS system";
    homepage = "https://github.com/misuzu/nix-simple-deploy";
    license = with licenses; [ mit ];
    platforms = platforms.all;
    maintainers = with maintainers; [ misuzu ];
  };
}
