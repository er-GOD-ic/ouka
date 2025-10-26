{
  description = "Rust development shell using rust-overlay";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
  }: let
    system = "x86_64-linux"; # 自分の環境に合わせて変更
    pkgs = import nixpkgs {
      inherit system;
      overlays = [rust-overlay.overlays.default];
    };
  in {
    devShells.${system}.default = pkgs.mkShell {
      name = "rust-dev-shell";

      packages = with pkgs; [
        (rust-bin.stable.latest.default) # 最新の stable Rust
        cargo-edit # cargo add / remove / upgrade
        cargo-expand # マクロ展開
        rust-analyzer # lsp
        rustfmt # formatter
      ];
      nativeBuildInputs = with pkgs; [
        # ここに追加
        pkg-config
        systemd # libudev-dev相当（ヘッダーとライブラリを提供）
        lua5_4 # Luaの開発ヘッダーとライブラリ（.dev出力含む）
        luajit # LuaJit
        readline # readlineサポート（Luaの入力機能用）
        gcc # Cコンパイラ（Luaソースビルド用）
        gnumake # makeツール（ビルドスクリプト用）
      ];

      shellHook = ''
        echo "🦀 Rust dev shell ready!"
        rustc --version
        cargo --version
      '';
    };
  };
}
