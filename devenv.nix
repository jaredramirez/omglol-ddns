{ pkgs, inputs, ... }:

{
  packages = with pkgs; [ rustc cargo rustfmt gcc pkg-config openssl ];
}
