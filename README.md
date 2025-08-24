# Ultimate devcontainer setup

This projects is intended as a snippet for anyone
to copy-paste into their repo at will.

It's goal is to provide a base for using the "same"
docker/podman image for both automated scripts and
for interactive dev environment.

The idea is to have a place where to specify build dependencies and
separate place for DX dependencies
while the DX dependencies also imply the build dependencies.

The snippet expects either being able to run docker (without sudo)
or a working rootless podman installation.

## Demo

- clone this repo
- `./setup`
- `./run`
- `ls -l /examples/` (inside the container now)
- `exit`
- `./run ls -l /examples/` (outside the container now)

As you can see the `./setup` command has prebuilt both containers.
(the second uses the previous one as a base so no duplication should happen)
Then `./run` command expects either a bash command (in which case it executes it in the build-only environment)
or if run without any command, launches interactive bash shell with the DX friendly environment set up.

As a bonus, `./setup` command also created `./.devcontainer/` directory (unless already present)
with a working vscode devcontainer which you can launch and also contains all the DX awesomeness.

## Intended usage

- Copy the environment directory
- inspire yourself by `./run`, `./setup` and `./enter` scripts
- this is just a starting point, explore and modify to your liking
