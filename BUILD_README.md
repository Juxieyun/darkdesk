<!--
 * @Author: SpenserCai
 * @Date: 2024-11-25 12:54:52
 * @version: 
 * @LastEditors: SpenserCai
 * @LastEditTime: 2024-11-25 12:59:13
 * @Description: file content
-->
# 编译

在flutter编译，生成ffi桥梁时需要讲执行dart升级，然后生成桥梁文件，接着使用撤销pubspec.lock文件变动然后编译。
```bash
cd flutter
flutter clean
flutter pub get
dart pub get
dart pub upgrade
flutter_rust_bridge_codegen --rust-input ../src/flutter_ffi.rs --dart-output ./lib/generated_bridge.dart --c-output ./macos/Runner/bridge_generated.h
# 此处撤销pubspec.lock文件变动
dart pub get
cd ..
python3 build.py --flutter
```