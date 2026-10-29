#!/bin/sh
# Instala o que falta para compilar o monk: compilador C (linker), curl, Rust
# (via rustup) e os componentes rustfmt e clippy. Pode ser rodado várias vezes.
set -eu

tem() { command -v "$1" >/dev/null 2>&1; }

instalar_pacotes() {
    if [ "$(id -u)" -eq 0 ]; then sudo=""; else sudo="sudo"; fi
    if tem dnf; then
        $sudo dnf install -y "$@"
    elif tem apt-get; then
        $sudo apt-get update && $sudo apt-get install -y "$@"
    elif tem pacman; then
        $sudo pacman -S --needed --noconfirm "$@"
    elif tem zypper; then
        $sudo zypper install -y "$@"
    elif tem brew; then
        brew install "$@"
    else
        echo "bootstrap: gerenciador de pacotes não reconhecido; instale manualmente: $*" >&2
        exit 1
    fi
}

if ! tem cc; then
    echo "==> instalando um compilador C (o Rust usa como linker)"
    if [ "$(uname -s)" = "Darwin" ]; then
        xcode-select --install || true
    elif tem apt-get; then
        instalar_pacotes build-essential
    else
        instalar_pacotes gcc
    fi
fi

if ! tem curl; then
    echo "==> instalando curl"
    instalar_pacotes curl
fi

PATH="$HOME/.cargo/bin:$PATH"
if ! tem rustup; then
    if tem cargo; then
        echo "bootstrap: cargo encontrado sem rustup; garanta que rustfmt e clippy estejam instalados" >&2
    else
        echo "==> instalando o Rust via rustup"
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
    fi
fi

if tem rustup; then
    echo "==> instalando rustfmt e clippy"
    rustup component add rustfmt clippy
fi

echo "==> baixando as dependências do crate"
cargo fetch

echo "==> pronto: $(cargo --version)"
case "$(command -v cargo)" in
    "$HOME/.cargo/bin/cargo")
        echo "Se o cargo não estiver no PATH do seu shell, adicione ~/.cargo/bin"
        echo "(bash/zsh: . ~/.cargo/env; fish: fish_add_path ~/.cargo/bin)."
        ;;
esac
