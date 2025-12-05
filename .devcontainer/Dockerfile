FROM fedora:latest@sha256:6cd815d862109208adf6040ea13391fe6aeb87a9dc80735c2ab07083fdf5e03a

# ---
# Setup base system ...
# ---

# Enable man pages by commenting out the nodocs flag
COPY <<EOF /etc/dnf/dnf.conf
[main]
gpgcheck=True
installonly_limit=3
clean_requirements_on_remove=True
best=False
skip_if_unavailable=True
install_weak_deps=False
# tsflags=nodocs
EOF

# Rust stuff goes in /opt so we don't end up with system and user installs: this is a single user system.
ENV RUSTUP_HOME=/opt/rustup \
    CARGO_HOME=/opt/cargo \
    PATH=/opt/cargo/bin:$PATH
RUN mkdir --mode=777 --parents $RUSTUP_HOME \
&& mkdir --mode=777 --parents $CARGO_HOME

# Create the default user
ARG USERNAME=ninjacoder
ARG USER_UID=1000
ARG USER_GID=${USER_UID}
RUN groupadd --gid ${USER_GID} ${USERNAME} \
&& useradd --uid ${USER_UID} --gid ${USER_GID} -m ${USERNAME} \
&& echo ${USERNAME} ALL=\(root\) NOPASSWD:ALL > /etc/sudoers.d/${USERNAME} \
&& chmod 0440 /etc/sudoers.d/${USERNAME}

# ---
# Install ...
# ---

# Man pages for all the stuff which is already installed, man itself and basic manpages
RUN dnf -y install \
    awk \
    man \
    man-db \
    man-pages \
&& dnf -y update \
&& dnf -y reinstall --skip-unavailable $(dnf list --installed | awk '{print $1}')

# Basic development tools
RUN dnf -y install \
    bash-completion \
    git \
    just \
    which

# Rust
ENV RUSTUP_HOME=/opt/rustup \
    CARGO_HOME=/opt/cargo \
    PATH=/opt/cargo/bin:$PATH

RUN groupadd rust \
    && usermod -a -G rust root \
    && usermod -a -G rust ${USERNAME}

RUN dnf -y install \
    clang \
    mold \
    rustup

USER root:rust

RUN umask 0002 \
&& rustup-init -v -y \
&& rustup target add \
    x86_64-pc-windows-msvc \
&& rustup component add \
    clippy \
    llvm-tools \
    llvm-tools-preview \
    rustfmt \
    rust-src

RUN umask 0002 \
&& curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash \
&& cargo binstall --secure -y \
    cargo-cyclonedx \
    cargo-expand \
    cargo-machete \
    cargo-nextest \
    cargo-udeps \
    cargo-xwin \
    grcov \
    mdbook \
    miniserve

RUN umask 0002 \
&& cat <<EOF >> ${CARGO_HOME}/config.toml
[target.'cfg(target_os = "linux")']
rustflags = ["-C", "link-arg=-fuse-ld=mold"]

[target.'cfg(target_os = "windows")']
rustflags = ["-C", "link-arg=-fuse-ld=mold"]
EOF

USER root

# Install SurrealDb server
RUN curl -sSf https://install.surrealdb.com | sh \
&& dnf -y install logrotate

# Libs for building & previewing UI
RUN dnf install -y \
    libinput \
    wayland-devel \
    xorg-x11-server-Xwayland

# ---
# Final setup steps
# ---

# Set the default user
USER ${USERNAME}
