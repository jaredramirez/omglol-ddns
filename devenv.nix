{ pkgs, inputs, ... }:

{
  packages = with pkgs; [ rustc cargo rustfmt ];
}
