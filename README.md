# build-your-own-docker

This is a starting point for Rust solutions to the
["Build Your Own Docker" Challenge](https://codecrafters.io/challenges/docker).

In this challenge, you'll build a program that can pull an image from
[Docker Hub](https://hub.docker.com/) and execute commands in it. Along the way,
we'll learn about [chroot](https://en.wikipedia.org/wiki/Chroot),
[kernel namespaces](https://en.wikipedia.org/wiki/Linux_namespaces), the
[docker registry API](https://docs.docker.com/registry/spec/api/) and much more.


```sh
mydocker run ubuntu:latest /usr/local/bin/docker-explorer echo hey
```

This command compiles your Rust project, so it might be slow the first time you
run it. Subsequent runs will be fast.
