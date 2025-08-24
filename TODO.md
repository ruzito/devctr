# TODOs

- [x] - make_project.sh
- [ ] - Test suite
- [x] - Inner directory approach
- [ ] - Bootstrap
- [x] - Multirepo setup
- [x] - Polyglot monorepo setup
- [ ] - update README.md

## make_project.sh

Turn this into an executable that actually builds the needed directory structure with interactive inputs
Similar to make_react_project or whatever.

That would mean this repo itself could have a different directory structure than the final one used by the user

I guess I could make it a rust project even.
Target executable candidates:
    - appimage
    - bash script
    - ELF executable
        - Rust
        - C++

## Test suite

Now that the structure is controlled by the script and not by this repo, I can add test suites and everything

## Inner directory approach

- Evaluate if I should use submodule or .gitignored directory for the inner repo
- Refactor so that this is a parent directory (which possibly uses git submodule as the main code repo?)
- So that you don't have to tinker with the repo inside and do some gitignore stuff to hide this snippet.

- The usage of this snippet would then be to:
    - fork the repo
    - clone the fork
    - run initial setup script (which asks for the link to the project repo and clones it into subdirectory?)

- Issues:
    - repo's CI/CD has no idea about outer directory so it cannot reuse the image?
        - is that good or bad?
        - if I make it so that the actual build dependency Dockerfile remains intact (everything else would be additional stages)
            - I could build the image from the subrepo first with custom tag and then build the additional stage ready for dev
            - instead of first constructing all this shit from scratch and injecting it

## Multirepo setup

Allow for managing multiple git repos

## Polyglot monorepo setup

Allow for managing multiple dev containers targetting the same repo (with possible subdirectory targets in that single repo)

## Bootstrap

Use this tool to create a devcontainer for developing this project