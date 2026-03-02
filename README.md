# mdbook-arborium

This integrates [arborium][https://arborium.bearcove.eu/] with mdbook.

## Roadmap 

- [x] Static mdbook preprocessor.
    - [x] replaces the code blocs with the highlighted code using custom HTML
    tags.
    - [x] Theme selector using js and css.
    - [x] Only include part of the themes with the arguments of the `install`
    subcommand.
- [ ] Add scrolling to the menu for selecting the theme.
- [ ] Switch theme configuration to `book.toml` instead of arguments.
- [ ] Expose some arborium config in the `book.toml`.
- [ ] Support for custom tree-sitter grammars. 

## How the preprocessor works

The preprocessor searches  for ```` ```lang ```` where `lang` is one of the
languages supported by arborium and replaces them with the highlighted html. If
`lang` isn't supported the codeblock is left as is.

So if any other plugins uses the same specifier as a grammar for the code it
should be declared __before__ `mdbook-arborium`.

For the preprocessor to work you have to run the install subcommand that will 
generate the necessary `js` and `css` files. Then you register them to the
