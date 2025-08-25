use clap::{Arg, Command};
use indoc::formatdoc;
use users::{get_current_gid, get_current_uid, get_current_username};
use std::{os::unix::fs::PermissionsExt, path::PathBuf, io::Write};

fn cli() -> Command {
    Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand(
            Command::new("add-repo")
                .about("Add a new source code repository")
                .arg(Arg::new("name").required(true))
                .arg(Arg::new("git").long("git").required(false)),
        )
        .subcommand(
            Command::new("add-container")
                .about("Add a new container definition and point it to a repository or its subdir")
                .arg(Arg::new("name").required(true))
                .arg(Arg::new("repo").short('r').long("repo").required(true))
                .arg(Arg::new("subdir").short('s').long("subdir").required(false)),
        )
        .subcommand(
            Command::new("init")
                .about("Initialize the application")
                .arg(Arg::new("name").required(true))
        )
}

fn main() {
    let matches = cli().get_matches();
    match matches.subcommand() {
        Some(("add-repo", sub_m)) => {
            let name = sub_m.get_one::<String>("name").unwrap();
            let git = sub_m.get_one::<String>("git");
            match add_repo(&name, git.map(|s| s.as_str())) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Error adding repo:\n\t- {}", e);
                    std::process::exit(1);
                }
            }
        }
        Some(("add-container", sub_m)) => {
            let name = sub_m.get_one::<String>("name").unwrap();
            let repo = sub_m.get_one::<String>("repo").unwrap();
            let subdir = sub_m.get_one::<String>("subdir");
            match add_container(&name, &name, &repo, &subdir.unwrap_or(&"".into())) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Error adding container:\n\t- {}", e);
                    std::process::exit(1);
                }
            }
        }
        Some(("init", sub_m)) => {
            let name = sub_m.get_one::<String>("name").unwrap();
            match init(&name) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Error initializing application:\n\t- {}", e);
                    std::process::exit(1);
                }
            }
        }
        _ => {
            println!("No subcommand was used");
            let _ = cli().print_help();
            std::process::exit(2);
        }
    }
}

fn init(name: &str) -> std::io::Result<()> {
    if PathBuf::from(format!("{name}")).exists() {
        return Err(std::io::Error::new(std::io::ErrorKind::AlreadyExists, "Dir already exists"));
    }
    std::fs::create_dir_all(format!("{name}/repos"))?;
    std::fs::create_dir_all(format!("{name}/.devcontainer"))?;
    std::fs::write(format!("{name}/.devcontainer/common.sh"), formatdoc! (r#"
        #!/bin/bash

        if [ -z "$CONT_CMD" ]; then
            if command -v podman &> /dev/null; then
                CONT_CMD=podman
            elif command -v docker &> /dev/null; then
                CONT_CMD=docker
            fi
        fi

        function in_container() {{
            if [[ -n "$CONTAINERIZED_ENV" && "$CONTAINERIZED_ENV" == "true" ]]; then
                return 0
            else
                return 1
            fi
        }}

        function build() {{
            # build_dockerfile
            local CONTAINER_NAME="$1"
            shift
            local CONTAINER_TARGET="$1"
            shift

            .devcontainer/"$CONTAINER_NAME"/prebuild > /dev/null 2> /dev/null
            $CONT_CMD build \
                -q \
                --target $CONTAINER_TARGET \
                --build-arg USER_UID=$(id -u) \
                --build-arg USER_GID=$(id -g) \
                --build-arg USERNAME=$(id -un) \
                --build-arg SUBREPO_TAG="subrepo_image_$CONTAINER_NAME" \
                --file .devcontainer/"$CONTAINER_NAME"/Dockerfile \
                .devcontainer/"$CONTAINER_NAME"
        }}

        function run() {{
            local CONTAINER_NAME="$1"
            shift
            local CONTAINER_TARGET="$1"
            shift
            if in_container; then
                echo "Already inside a containerized environment, running command directly."
                "$@"
                return $?
            fi
            $CONT_CMD run \
                --rm \
                -it \
                -v "$(pwd)":/workspace \
                -w /workspace \
                $(build "$CONTAINER_NAME" "$CONTAINER_TARGET") \
                "$@"
            return $?
        }}

    "#))?;
    std::fs::set_permissions(format!("{name}/.devcontainer/common.sh"), std::fs::Permissions::from_mode(0o775))?;
    Ok(())
}

fn add_repo(name: &str, git: Option<&str>) -> std::io::Result<()> {
    if PathBuf::from(format!("repos/{name}")).exists() {
        return Err(std::io::Error::new(std::io::ErrorKind::AlreadyExists, "Repo already exists"));
    }
    if let Some(git) = git {
        println!("Adding repo: {name}, git={git}");
        std::process::Command::new("git")
            .arg("clone")
            .arg(git)
            .arg(format!("./repos/{name}"))
            .status()?;
    } else {
        println!("Adding repo: {name}");
        std::fs::create_dir_all(format!("repos/{name}"))?;
        std::fs::write(format!("repos/{name}/Dockerfile"), formatdoc! (r#"
            FROM ubuntu:latest
        "#))?;
    }
    Ok(())
}

fn add_container(name: &str, display_name: &str, repo: &str, subdir: &str) -> std::io::Result<()> {
    if PathBuf::from(format!(".devcontainer/{name}")).exists() {
        return Err(std::io::Error::new(std::io::ErrorKind::AlreadyExists, "Container already exists"));
    }
    if !PathBuf::from(format!("repos/{repo}")).exists() {
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Repo does not exist, please run `devctr add-repo` first"));
    }

    if !PathBuf::from(format!("repos/{repo}/{subdir}")).exists() {
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Repo does not have specified subdir"));
    }

    let uid = get_current_uid();
    let gid = get_current_gid();
    let user = get_current_username()
        .and_then(|s| s.into_string().ok())
        .unwrap_or("unknown_user".into());

    let mut input = String::new();
    println!("Fill in the rest of the docker build command to build the image from your repo.");
    println!("For example: `docker build -f ./Dockerfile ./`");
    println!("(Or just hit `enter` but make sure to edit `.devcontainer/{name}/prebuild` script later)");
    println!();
    print!("repos/{repo}$> docker build ");
    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut input)?;
    let mut docker_build_cmd = input.trim().to_string();
    if docker_build_cmd.is_empty() {
        docker_build_cmd = "-f ./Dockerfile ./".into();
    }

    std::fs::create_dir_all(format!(".devcontainer/{name}"))?;
    std::fs::write(format!(".devcontainer/{name}/Dockerfile"), formatdoc!(r#"
        ARG SUBREPO_TAG=subrepo_image
        FROM $SUBREPO_TAG AS base

        ARG USERNAME=dev
        ARG USER_UID=1000
        ARG USER_GID=$USER_UID

        WORKDIR /

        # Set environment variable to indicate containerized environment
        ENV CONTAINERIZED_ENV=true

        # Ensure the current user's uid gid is setup in the container as well
        RUN u=$(getent passwd "$USER_UID" | cut -d: -f1); [ -n "$u" ] && userdel -r "$u" || true
        RUN g=$(getent group "$USER_GID" | cut -d: -f1); [ -n "$g" ] && groupdel "$g" || true
        RUN groupadd -g "$USER_GID" "$USERNAME"
        RUN useradd -m -u "$USER_UID" -g "$USER_GID" -s /bin/bash -c '' "$USERNAME"
        RUN passwd -l "$USERNAME"

        ########################################################################################
        # Here put commands that run as root and apply to both devcontainer and buildcontainer #
        ########################################################################################

        USER $USERNAME
        WORKDIR /home/${{USERNAME}}
        ENV PATH="/home/${{USERNAME}}/.local/bin:${{PATH}}"

        ########################################################################################
        # Here put commands that run as user and apply to both devcontainer and buildcontainer #
        ########################################################################################

        # Setup default entrypoint for devcontainer and buildcontainer
        WORKDIR /workspace
        CMD ["bash"]

        FROM base AS devcontainer

        USER root

        #####################################################################
        # Here put commands that run as root and only apply to devcontainer #
        #####################################################################
        RUN apt-get update && apt-get install -y \
            git \
            curl \
            jq \
            vim \
            && rm -rf /var/lib/apt/lists/* # clean up apt cache for small and cachable layer

        ARG USERNAME=dev
        USER $USERNAME
        WORKDIR /home/${{USERNAME}}

        #####################################################################
        # Here put commands that run as user and only apply to devcontainer #
        #####################################################################

        # Without specifying devcontainer target, build only the buildcontainer by default
        FROM base AS buildcontainer

    "#))?;


    std::fs::write(format!(".devcontainer/{name}/prebuild"), formatdoc!(r#"
        #!/bin/bash

        set -e -o pipefail

        if [ -z "$CONT_CMD" ]; then
            if command -v podman &> /dev/null; then
                CONT_CMD=podman
            elif command -v docker &> /dev/null; then
                CONT_CMD=docker
            fi
        fi

        # build base image from repo
        # the devcontainer expects to find an image tagged `subrepo_image_{name}`
        cd ./repos/{repo} && $CONT_CMD build -q -t subrepo_image_{name} {docker_build_cmd}

    "#))?;
    std::fs::set_permissions(format!(".devcontainer/{name}/prebuild"), std::fs::Permissions::from_mode(0o775))?;


    std::fs::write(format!(".devcontainer/{name}/devcontainer.json"), formatdoc!(r#"
        {{
            "name": "{display_name}",
            "build": {{
                "dockerfile": "./Dockerfile",
                "context": "..",
                "target": "devcontainer",
                "args": {{
                    "USERNAME": "{user}",
                    "USER_UID": "{uid}",
                    "USER_GID": "{gid}",
                    "SUBREPO_TAG": "subrepo_image_{name}"
                }}
            }},
            "initializeCommand": "CONT_CMD=docker ./.devcontainer/{name}/prebuild",
            "containerUser": "{user}",
            "customizations": {{
                "vscode": {{
                    "settings": {{
                    }},
                    "extensions": [
                    ]
                }}
            }},
            "workspaceMount": "source=${{localWorkspaceFolder}}/repos/{repo}/{subdir},target=/workspaces/${{localWorkspaceFolderBasename}},type=bind",
            "workspaceFolder": "/workspaces/${{localWorkspaceFolderBasename}}"
        }}
    "#))?;


    std::fs::write(format!("./{name}"), formatdoc!(r#"
        #!/bin/bash

        set -e -o pipefail

        . .devcontainer/common.sh

        if [ $# -eq 0 ]; then
            # echo "Building devcontainer using $CONT_CMD"
            run "{name}" "devcontainer" bash
        else
            # echo "Building buildcontainer using $CONT_CMD"
            run "{name}" "buildcontainer" "$@"
        fi

    "#))?;
    std::fs::set_permissions(format!("./{name}"), std::fs::Permissions::from_mode(0o775))?;

    Ok(())
}
