# Maintainer: Simon Aron <simonarons15@gmail.com>

pkgname=simple-osd
pkgver=0.1.0
pkgrel=1
pkgdesc="Simple on-screen indicator"
arch=('i686' 'x86_64')
url="https://github.com/sheepkill15/osd"
license=('GPL')
depends=()
makedepends=(cargo)

prepare() {
    cargo fetch --locked --target "$CARCH-unknown-linux-gnu"
}
build() {
    cd "$startdir"
    export RUSTUP_TOOLCHAIN=stable
    cargo build --frozen --release --all-features
}
check() {
    export RUSTUP_TOOLCHAIN=stable
    cargo test --frozen --all-features
}
package() {
    cd "$startdir"
    install -Dm755  "target/release/${pkgname}" "${pkgdir}/usr/bin/${pkgname}"
}