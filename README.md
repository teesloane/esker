[![justforfunnoreally.dev badge](https://img.shields.io/badge/justforfunnoreally-dev-9ff)](https://justforfunnoreally.dev)

> Esker is a static site generator for Obsidian.

Esker is alpha software and might not work for your Obsidian vault. Continue at your own peril (not recommended for non-technical users). Please note that filing issues will not be replied to until documentation is written (and who knows if even then). Good luck 😘!

# Usage

Esker was built to work with Obsidian, however it does not work out of the box without having Obsidian settings changed to work with Esker (at this moment).

Before using Esker, you will need to meet the following pre-requisites:

- you have an obsidian vault (or a structure similar to one) with the following settings:
  - in **Files & Links**, `New Link Format` should be set to &ldquo;Absolute path in vault&rdquo;
  - `Use wikilinks` is set to &ldquo;false&rdquo;
  - have your attachments in a specific directory.

Once the above have been met, the following steps should build a static site for your obsidian vault:

1.  Get a release of esker from Github Releases. Currently only Linux and Mac are available.
2.  Move the release into your path
3.  Navigate in your terminal to wherever your obsidian vault is
4.  run the command `esker new`
5.  You should notice that a new folder in your vault is created called `_esker`
6.  try running `esker watch` to create a live server for your site (viewable at localhost:8080) (or whatever `--port` you provide it).
7.  You can also run `esker build` to just build your site, which should be available at `<your vault directory>/_esker/_site`
8.  If you are not seeing anything, you&rsquo;ll need to ensure that your markdown files have valid frontmatter (see frontmatter section).

Additional documentation can be found [here](./docs/docs.org)

# Development

Want to try hacking on this project? To get going with development you will need:

- Rust >= 1.64

If you are doing cross compilation the following are needed:

- [Cargo ZigBuild](https://github.com/rust-cross/cargo-zigbuild)
- Zig 0.10 (only needed for cross compilation)

Most of the building of this project was [documented on youtube](https://www.youtube.com/watch?v=7uUn4GWYRuY&list=PLP_KZ-cWc_-hd_aGIk7-4VHoTr926Qgro) if that tickles
your fancy.
