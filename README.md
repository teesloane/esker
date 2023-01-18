> Esker is a static site generator for Obsidian.

Esker is alpha software and probably won't work for your Obsidian vault. Continue at your own peril (not recommended for non-technical users). Please note that filing issues will not be replied to until documentation is written (and who knows if even then). Good luck 😘!

# Usage

To use Esker and generate a site you'll need to do the following:

1. Ensure that you are *only using absolute links in obsidian* (see obsidian preferences).
1. Compile a binary for your system and move it into your path
1. Open your terminal, navigate to your vault.
1. Run `esker new`.
1. Inspect the `_esker` folder that now lives in your obsidian vault.
1. Try running `esker build` and see if it creates your site under `<my_vauly>/_esker/_site`.

# Development

Want to try hacking on this project? To get going with development you will need:

- Rust >= 1.64

If you are doing cross compilation the following are needed:

- [Cargo ZigBuild](https://github.com/rust-cross/cargo-zigbuild)
- Zig 0.10 (only needed for cross compilation)

Most of the building of this project was [documented on youtube](https://www.youtube.com/watch?v=7uUn4GWYRuY&list=PLP_KZ-cWc_-hd_aGIk7-4VHoTr926Qgro) if that tickles
your fancy.
