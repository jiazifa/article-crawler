# my-server-boilerplate
fastapi / sqlalchemy / permission / tasks

## 项目介绍

本项目是一个基于fastapi的后端项目模板，包含了常用的功能，如权限管理、任务调度等。

## 安装教程

[项目初始化教程](./rfcs/002-安装教程.md)

你也可以 直接使用命令行工具进行初始化

```shell
./scripts/setup_project.sh
```

## FAQ

- 有时候会碰到 `ModuleNotFoundError: No module named 'h11'`, 需要单独安装 `pip install h11`
- 参考  `./scripts/setup_project.sh` 中的安装命令
