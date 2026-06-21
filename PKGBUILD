# Mantenedor: Release Engineering <releng@laico.org>
# Modificado por: harpia <harpia@archon>
pkgname=laico-boot
pkgver=1.0.0
pkgrel=4
pkgdesc="Orquestrador de vetores operacionais e gerenciador de sessao TTY (Suporte DWL)"
arch=('x86_64')
url="https://github.com"
license=('GPL')

# zsh, fluxbox, dwm e dwl-git permanecem como dependências de execução
# Adicionamos 'cargo' em makedepends para garantir o ambiente de build
depends=('zsh' 'fluxbox' 'dwm' 'dwl-git')
makedepends=('cargo')

# Deixamos o source vazio porque o código já está presente nesta pasta local
source=()
sha256sums=()

build() {
    # Subimos um nível ($startdir) para executar o Cargo na raiz real do seu projeto
    cd "$startdir"
    
    # O Cargo compila o seu src/main.rs isoladamente e cria o target/release/archia
    cargo build --frozen --release
}

package() {
    # Acessa a raiz para coletar o binário compilado pelo Cargo
    cd "$startdir"
    
    # Coleta o binário 'archia' e instala renomeando para 'laico-boot' no pacote
    install -Dm755 "target/release/archia" "${pkgdir}/usr/bin/laico-boot"
}
