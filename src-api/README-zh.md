# airdo

健康检查并自动切换可用的应用配置

# 使用

1. 下载 [release](https://github.com/entitle40/airdo/release) 中的 `airdo.exe` 和 `config.example.com`
2. 将 `config.example.com` 改名为 `config.yml`，并按照自己的情况修改配置
3. 启动，配置文件默认为当前文件夹内的 `config.yml`，也可以使用 `-c` 参数进行指定，示例 `airdo.exe -c C:\Temp\123.yml`

# 开发

## sqlx migrate 功能
1. 该功能可以将 `sql` 的变更记录应用到数据库中
2. 首先在 `src-api/migrations` 添加一个格式为 `年月日时分秒_修改描述.sql` 的的文件
3. 然后将对数据库修改的 `sql` 放到上述文件中
4. 启动程序，程序会将更改应用到数据库中
5. 注意：一旦将更改应用到数据库中，就不允许再修改 `sql` 文件，因为 `sqlx` 维护了一个版本历史表，记录了每个 `sql` 文件的指纹，如果再次修改了 `sql` 文件，会直接抛出异常，如果需要修改或回退 `sql` ，请重新创建一个 `sql` 文件
6. 打包到生产环境时，`sqlx` 会将 `migrations` 目录包含到二进制文件中，启动程序时，同样会对数据库应用更改

## sqlx 调试
1. 安装 `sqlx-cli` 通过 `cargo install sqlx-cli`
2. 进入 `src-api` 目录，并运行设置数据库环境变量，`powershell` 如下 `$env:DATABASE_URL = sqlite://db.sqlite`
3. 执行 `sqlx migrate add 修改描述` ，命令将会在 `src-api/migrations` 目录下生成 `年月日时分秒_修改描述.sql` 文件
4. 将数据库更改 `sql` 放入文件中，执行 `sqlx migrate run` 命令，程序会将更改应用到数据库中