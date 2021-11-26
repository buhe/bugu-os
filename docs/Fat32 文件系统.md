### 数据结构

1. 目录的内容是 ShortDirEntry 数据，每个 ShortDirEntry 指向一个目录或者文件
2. 先创建文件，再用 write_at 写文件内容
3. 一个个簇写
4. 下一个簇在 Fat 项，Fat 包含若干项，每一项的内容是下一个簇的标号