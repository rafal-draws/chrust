let
  rust_overlay = import (builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz");
  pkgs = import <nixpkgs> { overlays = [ rust_overlay ]; };
  rustVersion = "latest";
  #rustVersion = "1.62.0";
  rust = pkgs.rust-bin.stable.${rustVersion}.default.override {
    extensions = [
      "rust-src" # for rust-analyzer
      "rust-analyzer"
    ];
  };
  in 
  pkgs.mkShell{
  buildInputs = [
    
	rust

  ] ++ (with pkgs; [
    pkg-config
    python312
	python312Packages.pip
	python312Packages.numpy
	python312Packages.matplotlib
	python312Packages.fastapi
	python312Packages.librosa
	python312Packages.redis
	python312Packages.apscheduler
	
	libiconv 
	openssl 
	
	cargo
	rustc
	openssl
	ffmpeg

	docker
	postgresql
	
  ]);
  
  
  RUST_BACKTRACE = 1;


	#nativeBuildInputs = with pkgs; [
	#	pkg-config
	#	rustc
	#	cargo
	#	gcc
	#	rustfmt
	#	clippy
	#];

	RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
	
	shellHook = ''
	export LIBTORCH=/home/util/libtorch/libtorch
	export LIBTORCH_LIB=/home/util/libtorch/libtorch
	export LIBTORCH_INCLUDE=/home/util/libtorch/libtorch
	
	export LD_LIBRARY_PATH=$LIBTORCH/lib:$LD_LIBRARY_PATH	

	export PATH=/home/rwd/.cargo/bin:$PATH

	echo "env is ready"
	echo "libtorch is $LIBTORCH"	
	echo "LD_LIBRARY_PATH is $LD_LIBRARY_PATH"
	'';

}
