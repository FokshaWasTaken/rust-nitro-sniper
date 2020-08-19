set -ex

main() {
    local src=$(pwd) \
          stage=$(mktemp -d)

    test -f Cargo.lock || cargo generate-lockfile

    cross build --target $TARGET --release

    cp target/$TARGET/release/rust-nitro-sniper* $stage/

    cd $stage
    tar czf $src/rns-$TRAVIS_TAG-$TARGET.tar.gz *
    cd $src

    rm -rf $stage
}

main
