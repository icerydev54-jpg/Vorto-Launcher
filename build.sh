#!/bin/bash

# 1. Install Rustup (the compiler manager)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"

# 2. Add the WebAssembly compilation target
rustup target add wasm32-unknown-unknown

# 3. Download Trunk (the Bevy web packager)
wget -qO- https://github.com/trunk-rs/trunk/releases/download/v0.17.5/trunk-x86_64-unknown-linux-gnu.tar.gz | tar -xzf-

# 4. Create index.html if it doesn't exist
if [ ! -f index.html ]; then
cat <<EOF > index.html
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0, user-scalable=no">
    <title>Vorto Studio</title>
    <style>
        html, body, canvas {
            margin: 0;
            padding: 0;
            width: 100%;
            height: 100%;
            overflow: hidden;
            background-color: #1a1a1a;
        }
    </style>
</head>
<body>
    <link data-trunk rel="rust" data-bin="vorto_engine" />
</body>
</html>
EOF
fi

# 5. Build the release using the downloaded trunk binary
./trunk build --release