# i3nav

A Terminal-user-interface for navigating and displaying the i3wm windows.


## Dependencies

While this could have been implemented in pure rust, this project depends on the `jq` executable (because I wanted to learn it!).


## Implementation

We follow the The Elm Architecture (TEA) model as outlined in ratatui's [documentation](https://ratatui.rs/concepts/application-patterns/the-elm-architecture/).

The central idea in 

1. Model
2. Handling updates
3. Update

## Installation

After cloning the repository run

```
cargo install --path .
```

to have access to the `i3-mgr` binary.

## Launching with i3

We recommand opening up `i3-mgr` in a new window and then sending the shell to the scratchpad. We can accomplish that with
the following bash script:

```bash
#!/usr/bin/bash
i3-msg exec "terminator --command 'i3-msg move scratchpad && i3-msg scratchpad show && i3-mgr'"
```
