Write-Host "Building wh2-luadb...";
cargo build --release;
Write-Host "Building wh2-luadb-kmm-launcher...";
cargo build --release --manifest-path="wh2-luadb-kmm-launcher/Cargo.toml";
Write-Host "Copying items to release dir...";
Copy-Item -Path ".\target\release\wh2-luadb.exe" -Destination ".\release";
Copy-Item -Path ".\wh2-luadb-kmm-launcher\target\release\wh2-luadb-kmm-launcher.exe" -Destination ".\release";