cd web/
tar xzf web_deps.tar.gz
cd js/
yarn build
cd ../../
flutter build web --release

# 判断如果带有--release参数，则执行输出版本号的操作
if [[ $* == *--release* ]]; then
    version=$(date "+%Y%m%d%H%M%S")
    echo "version: $version"
    #把build/web目录中除了web_deps.tar.gz以外的文件压缩到../web-client-release/web_{version}.tar.gz
    tar -zcvf ../web-client-release/web_${version}.tar.gz --exclude=web_deps.tar.gz -C build/web .
fi
    
