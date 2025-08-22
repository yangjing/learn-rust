# storage-cli

## devops-cli

下载项目

```shell
git clone https://github.com/yangbajing/learn-rust.git
cd technique-rust
```

执行示例：

```shell
cargo build --release

# 上传文件
RUST_LOG=debug ./target/release/devops-cli -f ./clis/storage-cli/.app.toml put ./target/release/devops-cli software/devops-cli

# 下载文件
RUST_LOG=debug ./target/release/devops-cli -f ./clis/storage-cli/.app.toml get software/devops-cli devops-cli

# 查询文件元数据
./target/release/devops-cli -f ./clis/storage-cli/.app.toml stat software/devops-cli
#chmod +x devops-cli && devops-cli -f ./clis/storage-cli/.app.toml stat software/devops-cli
```
