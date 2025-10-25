{
  description = "Rust development shell using rust-overlay";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { nixpkgs, rust-overlay }:
    let
      system = "x86_64-linux"; # 自分の環境に合わせて変更
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ rust-overlay.overlays.default ];
      };
    in {
      devShells.${system} = pkgs.mkShell {
        name = "rust-dev-shell";

        packages = with pkgs; [
          (rust-bin.stable.latest.default)  # 最新の stable Rust
          rust-analyzer                       # LSP
          cargo-clippy                         # 静的解析
          rustfmt                              # コード整形
          cargo-edit                           # cargo add / remove / upgrade
          cargo-expand                         # マクロ展開
        ];

        shellHook = ''
          echo "🦀 Rust dev shell ready!"
          rustc --version
          cargo --version
        '';
      };
    };
}
