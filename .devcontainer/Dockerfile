FROM fedora:latest@sha256:ee88ab8a5c8bf78687ddcecadf824767e845adc19d8cdedb56f48521eb162b43

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

# Python
RUN dnf -y install \
    python

# Rust (and python headers)
# and chown CARGO_HOME and RUSTUP_HOME to the default user 
RUN dnf -y install \
    clang \
    mold \
    python3-devel \
    rustup \
&& rustup-init -v -y \
&& rustup component add \
    clippy \
    llvm-tools-preview \
    rustfmt \
    rust-src
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash \
&& cargo binstall --secure -y \
    cargo-cyclonedx \        
    cargo-expand \
    cargo-nextest \
    grcov \
    mdbook
RUN chmod -R a+rwX ${CARGO_HOME} \
&& chmod -R a+rwX ${RUSTUP_HOME}

# install uv and nbstripout
COPY --from=ghcr.io/astral-sh/uv:0.6.17 /uv /uvx /bin/

# nbstripout - git diff & strip handling for jupyter metadata. (Needs to be activated in workspace)
RUN uv pip install --system nbstripout

# ---
# Final setup steps
# ---

# Set uv cache to inside workspace (allowing for hardlinking & explicit rentention management via uv)
ENV UV_CACHE_DIR=.uv_cache

# Set the default user
USER ${USERNAME}
