# Install and Build

Install rust (with rustup), clone the repo, and build.

```sh
# install rust (using rustup)
$ curl https://sh.rustup.rs -sSf | sh
# clone this repo
$ git clone git@github.com:lucasem/context
# build
$ cargo build
```

# Run

We are using the EC algorithm with primitives designed for string
transformation, implemented at
[`lucasem/ec`](https://github.com/lucasem/ec).

The easiest way to run this is to place the ec binary created from jetty's
`make` in the root directory of this project, and to use `cargo run` from
the project root:

```sh
# start from this project's root directory.
# clone, build, and copy jetty's ec:
$ git clone git@github.com:lucasem/ec ec-repo
$ cd ec-repo
$ make && cp ec ../ec
$ cd ..
# run
$ cargo run
```

For more customization, you can have the `$EC` environment variable point to
the jetty's ec binary and the `$EC_CURRICULUM` environment variable point to
a directory with similar structure to [`./curriculum/ec`](./curriculum/ec).
