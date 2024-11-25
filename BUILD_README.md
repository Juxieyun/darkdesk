<!--
 * @Author: SpenserCai
 * @Date: 2024-11-25 12:54:52
 * @version: 
 * @LastEditors: SpenserCai
 * @LastEditTime: 2024-11-26 00:05:22
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

为了实现可以编译完整版和精简版等多种形式的客户端，在[config.rs](./libs/hbb_common/src/config.rs#L81)中添加不同的编译配置。同时在[Cargo.toml](./Cargo.toml#L46)中添加不同的feature,如果时完整版直接去掉feature即可。

### 精简版编译Demo

```toml
[features]
....
hbb_common = { path = "libs/hbb_common",features = [ "easy_client" ] }
...
```

### 完整版编译

```toml
[features]
....
hbb_common = { path = "libs/hbb_common" }
...
```

