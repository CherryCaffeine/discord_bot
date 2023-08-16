# Discord bot Vampy ❤️

[![.github/workflows/ci.yml](https://github.com/CherryCaffeine/discord_bot/actions/workflows/ci.yml/badge.svg)](https://github.com/CherryCaffeine/discord_bot/actions/workflows/ci.yml)

![Vampy](https://i.imgur.com/TdtIHhP.png)

## Required tools

* [Git](https://git-scm.com/downloads)
* [Rust programming language tooling](https://www.rust-lang.org/tools/install)
* [`cargo-binstall`](https://github.com/cargo-bins/cargo-binstall)
* [`cargo-make`](https://github.com/sagiegurari/cargo-make)
* [`cargo-shuttle`](https://docs.shuttle.rs/introduction/installation)
* 🎉 That's it! 🎉

## How to get the local version of the repository or its fork

* Ensure you have `git` installed.
* Open the terminal (or command prompt). On Windows, you can do this by pressing `Win + R`, typing `cmd`, and pressing the "OK" button.
* Navigate to the directory where you would like to keep the local version of the repository or its fork. You can do this by running `cd <path-to-the-directory>` in the terminal. For example, running `cd C:\Users\<username>\Desktop` will navigate you to the desktop of user `<username>`.
* Run `git clone <repo>` where `<repo>` is the link to the repository or its fork. For example, running `git clone https://github.com/CherryCaffeine/discord_bot` will get the local version of the original repository.

## How to run locally (for testing/development)

* Ensure you have the [required tools](#required-tools) installed.
* Ensure you have the [local version of the repository or its fork](#how-to-get-the-local-version-of-the-repository-or-its-fork).
* Run `git pull` in the terminal from `discord_bot/` to get the latest updates.
* Run `cargo make ensure_cfg` in the terminal from `discord_bot/` to create `Secrets.toml` based on `Secrets.example.coml`.
* Ensure you have [Docker](https://docs.docker.com/get-docker/) installed. If you're new to this, [Docker Desktop](https://www.docker.com/products/docker-desktop/) will be the easiest way to get started.
* Run `cargo shuttle run` in the terminal from `discord_bot/` to run the bot locally.
* 😎 You got it! 😎

## How to contribute

* Share your ✨ stellar ✨ idea in the [`#server-suggestions`](https://discord.com/channels/1123378968607858769/1127121324716859423) channel.
* [Fork](https://docs.github.com/en/get-started/quickstart/fork-a-repo) this [repository](https://docs.github.com/en/get-started/quickstart/github-glossary#repository).
* [Clone](https://docs.github.com/en/get-started/quickstart/fork-a-repo#cloning-your-forked-repository) your forked repository via `git clone <the-link-to-the-fork>`.
* Make your changes. That's the *fun* part! Contact me (`@dmitrii_demenev`) on Discord if you need help 😄.
* "Stage" your changes with `git add .` in order to add all changes to the next commit.
* [Commit](https://docs.github.com/en/get-started/quickstart/contributing-to-projects#making-and-pushing-changes) your changes with a descriptive message via `git commit -m "<your-message>"`.
* [Push](https://docs.github.com/en/get-started/quickstart/contributing-to-projects#making-and-pushing-changes) your changes to your forked repository via `git push`.
* [Create a pull request](https://docs.github.com/en/get-started/quickstart/contributing-to-projects#making-a-pull-request) to the original repository.
* Ping me (`@dmitrii_demenev`) on Discord to speed up the [review](https://docs.github.com/en/get-started/quickstart/github-glossary#pull-request-review) of the [pull request](https://docs.github.com/en/get-started/quickstart/github-glossary#pull-request).
* 🎉 Celebrate your contribution! 🎉 You deserve it because you're 🔥

## Learn more about writing Discord bots with Shuttle, Serenity, and Rust

> Why not Python? 🐍
> Shuttle is free. ❤️

* YouTube playlist ["Discord Bot with Rust and Serenity - Meta Open Source"](https://www.youtube.com/watch?v=NVMHWUly1rc&list=PLzIwronG0sE5lQCPFP69Ukgz4d9dngaSi&ab_channel=MetaOpenSource).
* YouTube playlist ["Rust Programming Tutorials"](https://www.youtube.com/playlist?list=PLVvjrrRCBy2JSHf9tGxGKJ-bYAN_uDCUL).
* [The Rust Programming Language book](https://doc.rust-lang.org/book/).
* [Shuttle documentation](https://docs.shuttle.rs/introduction/welcome).
* [Serenity documentation](https://docs.rs/serenity/latest/serenity/).
* Ping me on Discord: `@dmitrii_demenev`.
