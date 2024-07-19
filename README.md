# Magpie Project

The Magpie Project try to make searching Inscryption card easier by providing a [Scryfall](https://scryfall.com) like interface.

-   `magpie_engine` is library crate and is the main backend behind everything. The `engine` handle fetching, query and storage of every set.
-   `magpie_tutor` is a binary crate and it is a discord bot that is build on top of the `engine` to make it easier to search for card in the middle of discord conversation.

Each project is store within their respective folder and all under the same git versioning system. Every commit should be tag with which project that commit is modifing like `[engine]`, `[tutor]` or just `[magpie]` if it does not modify any specific project. `magpie_engine` is not on [`crates.io`](crates.io) yet so you have to download it manually using the git link. Add this to you project

```toml
[dependencies]
magpie_engine = { git = "https://github.com/Mouthless-Stoat/Magpie.git"}
```
