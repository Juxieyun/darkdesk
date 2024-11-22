#!/usr/bin/env bash
###
 # @Author: SpenserCai
 # @Date: 2024-11-22 00:34:11
 # @version: 
 # @LastEditors: SpenserCai
 # @LastEditTime: 2024-11-22 22:24:34
 # @Description: file content
### 

echo $MACOS_CODESIGN_IDENTITY
cargo install flutter_rust_bridge_codegen --version 1.80.1 --features uuid
cd flutter; flutter pub get; cd -
~/.cargo/bin/flutter_rust_bridge_codegen --rust-input ./src/flutter_ffi.rs --dart-output ./flutter/lib/generated_bridge.dart --c-output ./flutter/macos/Runner/bridge_generated.h
./build.py --flutter
rm rustdesk-$VERSION.dmg
# security find-identity -v
codesign --force --options runtime -s $MACOS_CODESIGN_IDENTITY --deep --strict ./flutter/build/macos/Build/Products/Release/DarkDesk.app -vvv
create-dmg --icon "DarkDesk.app" 200 190 --hide-extension "DarkDesk.app" --window-size 800 400 --app-drop-link 600 185 rustdesk-$VERSION.dmg ./flutter/build/macos/Build/Products/Release/DarkDesk.app
codesign --force --options runtime -s $MACOS_CODESIGN_IDENTITY --deep --strict rustdesk-$VERSION.dmg -vvv
# notarize the rustdesk-${{ env.VERSION }}.dmg
rcodesign notary-submit --api-key-path ~/.p12/api-key.json  --staple rustdesk-$VERSION.dmg
