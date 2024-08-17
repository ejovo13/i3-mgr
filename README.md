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
