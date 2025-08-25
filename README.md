# Ultimate devcontainer setup

- This app is supposed to be used and then forgotten.
- It is supposed to solve "The user problem" of devcontainers

## TLDR

```bash
~$> whoami
example_user
~$> hostname
bumblebee
~$> devctr --help
~$> devctr init --help
~$> devctr init playground
~$> cd playground
~/playground$> devctr add-repo --help
~/playground$> devctr add-repo example_project
~/playground$> devctr add-container --help
~/playground$> devctr add-container example_container --repo example_project
~/playground$> ./example_container whoami
example_user
~/playground$> ./example_container hostname
4ffaef3ea758
~/playground$> ./example_container
example_user@4ffaef3ea758:/workspace$ exit
~/playground$> code . # and then "Reopen in Container"
```

> **NOTE:** The whole project is like 150 lines of rust code + 150 lines of embedded strings (which are pretty much the output of this tool) in a single file.
> It may actually be easier to read the code than this README.md.
> If you read further, you have been warned. :-D

## Why?

Devcontainers are amazing in theory, install all dependencies in sandbox environment and you can build anywhere.
There is one big issue tho. Everyone uses Docker. Docker runs it's processes with uid 0 (granted in sandboxed environment, but as sudo none the less)
Therefore files created in filesystems mounted inside the docker container will too be created under super user account.
That is not ideal for development environment, where we want to build artifacts using the current user's account.

### Possible solutions

At the time of creating this project multiple solutions seem plausible (with varying degree of usability)

**User mappings:**

This needs configuration on the host machine and maps ranges of uids/gids,
therefore if you want to use this as a solution, you would essentially offset the uid 0 into uid 1000.
This creates issues with apps that we actually want to run as superuser in the docker like installing dependencies and tools.
I may have just done something wrong, but it felt EXTREMELY fiddly the whole time.
To keep the root as uid 0 in container, you'd still need to create additional user which would be then be able to be offset by the user mapping.
At that point you can just create the user at the correct uid:gid which brings us to the second solution.

**Creating a user in container:**

This solution has it's own issues.
Docker does not have a builtin way to read the current user's uid:gid during `build`
and it is already too late to tinker with the image during `run` command.
You have to either guess the uid to be 1000, or you have to force users to use a specific `--build-arg` to match the actual uid of the current user.
In addition you need to actually create the user in the container. It is common that such user is created in devcontainer ready images from microsoft.
But it is not common for images provided by the toolchain makers themselves.

### This project's approach

The goal of this project is to provide a base for using the "same"
container image for both automated scripts and
for interactive dev environment.

The idea is to have a place where to specify build dependencies and
separate place for DX dependencies
while the DX dependencies also imply the build dependencies.
In ideal case to use a build container already provided by the repository itself.

This app is supposed to create such development environment
without the need to alter the targeted repository in any way,
while still being able to use devcontainers to open the project with all
the necessary dependencies.

The output of this tool is a directory with all the configs and scripts
needed for a quick and easy start with devcontainers adventure.
After this tool is used, to generate the environment,
it is no longer needed in order to actually use said environment.

The generated snippet takes the approach of the second possible solution,
it creates a Dockerfile that is intended to be used as additional layers of some other image
which includes the user whose uid:gid matches the current user (at the time of creating the environment).
For the sake of simplicity it is targeting ubuntu based images, but it should be possible
to use the generated code as a reference of what has to be done if any tweaking is necessary.



## Quick start demo

- Download the tool from github releases (or build it yourself with `cargo` from source)
- `PATH=/path/to/where/you/downloaded/the/app:"$PATH"`
    - this is a temporary "install", feel free to install the app however you like
- First make yourself familiar with `devctr --help`, additionally you can run `devctr <subcommand> --help`
- `devctr init playground`
- `cd playground`
- `devctr add-repo example_project`
    - this optionally takes `--git <url>` as argument which automatically clones your project
- `devctr add-container frontend --repo example_project`
    - **Now** it asks you to fill in a docker command, this will be explained later, for now just press enter
    - NOTE: If one container is enough, my personal recomendation would be to use `run` as the container name. (You'll see why later) If not, I'd recomend using the name of the component of your project this container is supposed to be used for.
- `devctr add-container backend --repo example_project`
    - Again just press enter for now
- `./frontend whoami` - there is now a premade script named after your container you can use to interact with your container
- `./frontend hostname`
- `./backend whoami`
- `./backend hostname`
- `./frontend` - now you are in the container
- `exit` - let's exit it now
- `code .` - now you can also open vscode here and `Reopen in Container`
    - Note that when devcontainer opens, it opens in the `repos/example_project`
    - If you have a specific subdirectory in mind, you can use optional argument `--subdir`
    - for example `devctr add-container frontend --repo example_project --subdir frontend` would instead configure `vscode` to open `repos/example_project/frontend` (if such directory existed) when you enter the frontend container. This can be useful for big polyglot monorepos
- Now you can explore the folder structure created
     - ```
        *
        '- playground/
            |- .devcontainer/
            |   |- frontend/
            |   |   |- devcontainer.json    - Here you can customize your `vscode` devcontainer experience
            |   |   |- Dockerfile           - Here you can add any dependencies needed for your devcontainer
            |   |   '- prebuild             - devcontainer configuration
            |   |- backend/
            |   |   |- devcontainer.json    - ditto
            |   |   |- Dockerfile           - ditto
            |   |   '- prebuild             - ditto
            |   '- common.sh
            |- repos/
            |   '- example_project          - This is a mock of a real repository, here you will be developing your project
            |       '- Dockerfile
            |- frontend                     - script to enter or run stuff in the devcontainer
            '- backend                      - ditto
        ```

**Dockerfile:**

Dockerfile expects `--build-arg SUBREPO_TAG="subrepo_image_$CONTAINER_NAME"`.
This argument is used in the first two lines of the Dockerfile

```Dockerfile
ARG SUBREPO_TAG=subrepo_image
FROM $SUBREPO_TAG AS base
```

This is expected to be using the container built by the `prebuild` script

```bash
docker build -q -t "subrepo_image_$CONTAINER_NAME" ...
```

Next, there are 4 sections intended for tweaking
- `base-root` - this will be common for non-interactive script running AND interactive devcontainer mode (runs under root)
- `base-user` - this will be common for non-interactive script running AND interactive devcontainer mode (runs under user)
- `devcontainer-root` - this will be present only in interactive devcontainer mode (runs under root)
- `devcontainer-user` - this will be present only in interactive devcontainer mode (runs under user)

**prebuild:**

This is intended to enable first building an image shipped with the actual repo.
This file consists of primarily of the command you specified in the `add-container` step where it asks you to fill in `docker build` command.
In the Quick start demo, we let it use a default value, designed to be compatible with the default (not git-cloned) repo.
If the repo you are working with does not include any docker image, you have multiple options:
- edit `prebuild` script to build some other image first,
-  directly edit `Dockerfile` to use some existing base image (instead of the parametrized tag) and leave `prebuild` script empty.

## You are on your own now, but do not worry

Just make one small change at a time and you'll be alright.