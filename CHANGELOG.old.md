
## [unreleased]

### Documentation

- Mixed name punctuation and node span ([#44](https://github.com/stoically/syn-rsx/issues/44))
- Missing module documentation ([#45](https://github.com/stoically/syn-rsx/issues/45))
- Fix formatting ([#46](https://github.com/stoically/syn-rsx/issues/46))
- Fix code block ([#47](https://github.com/stoically/syn-rsx/issues/47))

### Features

- Track the source span for all nodes ([#42](https://github.com/stoically/syn-rsx/issues/42))

## [0.9.0] - 2022-11-10

### Documentation

- Remove outdated node link ([#36](https://github.com/stoically/syn-rsx/issues/36))
- Hint regarding braced blocks ([#38](https://github.com/stoically/syn-rsx/issues/38))
- Fix typo ([#39](https://github.com/stoically/syn-rsx/issues/39))

## [0.9.0-beta.2] - 2022-11-06

### Features

- [**breaking**] Replace Colon and Dash with a merged variant ([#34](https://github.com/stoically/syn-rsx/issues/34))

## [0.9.0-beta.1] - 2022-11-03

### Chore

- Add rustfmt.toml

### Documentation

- Update README
- Update example
- Update html macro docs
- Add link to example
- Fix node links
- Improve parser docs
- Remove TODO
- Fix attribute value example ([#28](https://github.com/stoically/syn-rsx/issues/28))
- Fix blocks example ([#29](https://github.com/stoically/syn-rsx/issues/29))
- Fix typo ([#30](https://github.com/stoically/syn-rsx/issues/30))

### Features

- [**breaking**] Make path_to_string private

### Other

- Move flat tree converter back into node parser
- Switch fmt to nightly toolchain

### Refactor

- [**breaking**] Drop `NodeName::span` method
- Pass block_transform a forked stream
- Move flat tree converter to node method
- Replace extrude with let-else ([#31](https://github.com/stoically/syn-rsx/issues/31))

## [0.9.0-alpha.1] - 2022-10-21

### Chore

- Add Cargo.lock to .gitignore
- Add git-cliff configuration
- Add CHANGELOG
- Use the actual html-to-string-macro crate as example
- Update README badges

### Refactor

- Move config into dedicated module
- [**breaking**] Switch `Node` to enum-style ([#23](https://github.com/stoically/syn-rsx/issues/23))

## [0.8.1] - 2022-06-26

### Chore

- Clippy
- Clippy
- Remove Cargo.lock
- Bump dependencies

### Documentation

- Update README

## [0.8.0] - 2021-02-17

### Documentation

- Fix and sync
- Typo

### Features

- Doctypes, comments and fragments
- Value_as_string support for `ExprPath`

### Other

- Should be value not name

### Refactor

- Remove unnecessary `Result`

### Testing

- More reserved keywords tests

## [0.8.0-beta.2] - 2021-01-04

### Documentation

- Sync lib with readme

### Features

- [**breaking**] Block in node name position ([#11](https://github.com/stoically/syn-rsx/issues/11))

### Other

- Tarpaulin and codecov

## [0.8.0-beta.1] - 2021-01-03

### Documentation

- Node

### Features

- Properly handle empty elements
- [**breaking**] Transform_block callback ([#9](https://github.com/stoically/syn-rsx/issues/9))
- [**breaking**] Doctype ([#6](https://github.com/stoically/syn-rsx/issues/6))
- [**breaking**] Html comments ([#7](https://github.com/stoically/syn-rsx/issues/7))
- [**breaking**] Fragments ([#8](https://github.com/stoically/syn-rsx/issues/8))

### Other

- Formatting
- Bump criterion

### Refactor

- Cleanup

## [0.7.3] - 2020-10-30

### Documentation

- Rephrase misleading unquoted text hint
- Update node description
- Update NodeName description

### Features

- Value_as_block method for nodes
- Implement ToString for NodeName
- Support blocks in html-to-string-macro
- Implement ToTokens for NodeName

### Other

- Only count top level nodes in case of flat_tree
- Parse2 with criterion
- More test tokens
- Update

### Performance

- More peeking and better block parsing ([#5](https://github.com/stoically/syn-rsx/issues/5))
- Use `node_name_punctuated_ident` to parse name path

### Refactor

- Better error reporting
- Rename test file
- Switch impl ToString on Node to impl Display
- Merge text and block handling

## [0.7.2] - 2020-09-10

### Documentation

- Error reporting

### Features

- Expose span fn on NodeName as well

### Refactor

- Better error messages

## [0.7.1] - 2020-09-09

### Other

- Check after parsing is done

## [0.7.0] - 2020-09-09

### Documentation

- Update readme
- Update readme
- Test feature examples

### Features

- Helper function to get node name span
- Support blocks as attributes
- Configure maximum number of allowed top level nodes
- Configure type of top level nodes

### Other

- Bump
- Add html_to_string macro

### Refactor

- Peek to determine node type
- Better error messages
- Move integration tests into tests folder
- Move parse configuration from arg to dedicated fns
- Check value first
- Get rid of helper struct
- Exactly required number of top level nodes

## [0.6.1] - 2020-06-06

### Chore

- Update cargo lock

### Documentation

- Typo

## [0.6.0] - 2020-06-06

### Chore

- Update cargo.lock

### Documentation

- Exposed Dash and minor improvements

### Features

- Node names with colons

### Refactor

- Cleanup
- Rename Dashed to Dash
- Tests cleanup

## [0.5.0] - 2020-06-04

### Features

- Dashed node names

## [0.4.1] - 2020-06-04

### Documentation

- Update readme

### Refactor

- Cleanup

## [0.4.0] - 2020-06-03

### Documentation

- Update example

### Refactor

- Rename childs to children ([#1](https://github.com/stoically/syn-rsx/issues/1))

## [0.3.4] - 2020-06-03

### Documentation

- Spelling
- Update example
- Update readme

### Refactor

- Change node name to `syn::ExprPath`
- Use advance_to after fork
- Restructure code

## [0.3.1] - 2020-06-03

### Refactor

- Cleanup

## [0.3.0] - 2020-06-03

### Documentation

- Update readme

### Features

- Parse tag name and attribute value as `syn::Path`

### Refactor

- Clippy lints
- Cleanup
- Use iter::once
- Cleanup
- Block expression parsing

## [0.2.0] - 2020-05-30

### Chore

- Update cargo.lock
- Bump syn dep

### Documentation

- Project keywords

### Features

- Parse full block

## [0.1.2] - 2020-05-29

### Chore

- Update cargo.lock

### Documentation

- Readme badges
- Readme key for crates.io
- Update
- Update readme

### Other

- Initial commit

- Build workflow

### Refactor

- Pub not needed
- Parse blocks as NodeType::Block

<!-- generated by git-cliff -->
