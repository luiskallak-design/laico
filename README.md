# 🏛️ Projeto Archia (laico-boot) - Guia de Configuração e Dependências

[![License: MIT](https://shields.io)](https://opensource.org)

## 📸 Demonstração Visual do Ecossistema

### Interface Principal e Temas
![Tema Principal](assets/laico-boottema.png)
![Variação de Tema 1](assets/laico-boottema1.png)
![Variação de Tema 2](assets/laico-boottema2.png)

### Integração com os Vetores Operacionais
![Navegação e Editor Nano](assets/laico-bootnano.png)
![Gerenciador de Arquivos Thunar](assets/laico-bootthunar.png)
![Player de Mídia Integrado](assets/laico-bootplayer.png)
![Navegador Web Integrado](assets/laico-bootweb.png)

---

Este documento centraliza as dependências, comandos de build e instalação via terminal para o projeto `laico-boot`.

---

## 📥 1. Como Clonar o Projeto

Utilize o `git clone` com a URL HTTPS:

```bash
git clone https://github.com/luiskallak-design/laico.git
cd laico
```

---

## 🛠️ 2. Dependências do Sistema (Arch Linux)

Instale as ferramentas base e utilitários para **dwm (X11)** e **dwl (Wayland)**:

```bash
sudo pacman -S --needed base-devel rustup fastfetch yazi nano kitty qterminal foot feh swww xcompmgr make
```

**Principais Ferramentas:**
*   **X11/Wayland:** `feh`/`xcompmgr` ou `swww`.
*   **Terminais:** `kitty`, `qterminal`, `foot`.
*   **TUIs:** `nano` (F1), `fastfetch` (F2), `yazi` (F3).

---

## ⚙️ 3. Configuração do Ambiente Rust

```bash
rustup default stable
```

---

## 📦 4. Dependências do Projeto (`Cargo.toml`)

```toml
[dependencies]
crossterm = "0.27"
ratatui = "0.26"
libc = "0.2"
```

---

## 🚀 5. Compilação e Implantação Automatizada (`Makefile`)

A automação é gerenciada via `make`:
*   `make`: Compila o projeto.
*   `make install`: Instala o binário em `~/.cargo/bin/laico-boot`.
*   `make deploy`: Compila, instala e reinicia o launcher.

---

## 🛡️ 6. Mapeamento de Atalhos Operacionais

*   [**H**] Home
*   [**F1**] Nano
*   [**F2**] Fastfetch
*   [**F3**] Yazi
*   [**F4**] Espectro
*   [**F5**] Wallpaper
*   [**F10**] Terminate
