# 要求输入vcpkg路径，并将路径以：export VCPKG_ROOT=...的形式写入.vcpkg_env文件中
WORK_DIR=$(pwd)
# 1. 输入vcpkg路径
echo "Please input the path of vcpkg: "
read vcpkg_path
# 2. 检查路径是否存在，不存在返回 路径错误并退出
if [ ! -d $vcpkg_path ]; then
    echo "Path error: $vcpkg_path"
    exit 1
fi
# 转换成绝对路径
vcpkg_path=$(cd $vcpkg_path; pwd)
# 3. 写入.vcpkg_env文件当前目录下
echo "export VCPKG_ROOT=$vcpkg_path" > $WORK_DIR/.vcpkg_env
# 4. 输出提示信息，可以使用 source $WORK_DIR/.vcpkg_env 来加载环境变量
echo "Please use 'source $WORK_DIR/.vcpkg_env' to load vcpkg environment."
echo "Done."
