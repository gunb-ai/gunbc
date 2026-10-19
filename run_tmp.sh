set -e
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
p=src/v2/test/claim/emit/swift_type_expression_projection.dag
fns=$(grep -o '^test fn [a-z_0-9]*' $p | awk '{print $3}' | paste -sd,)
./target/release/claim_batch --source-root dag --source-root src/v2 --entry $p --functions $fns 2>&1 | grep "^PASS\|^FAIL\|error"
echo "=== swift typecheck (MANUAL RECEIPT)"
W=$(mktemp -d); cd $W
B=https://download.swift.org/swift-6.4.1-release/debian13/swift-6.4.1-RELEASE/swift-6.4.1-RELEASE-debian13.tar.gz
curl -fsSLO $B; curl -fsSLO $B.sig
curl -fsSL https://swift.org/keys/all-keys.asc -o keys.raw
(gzip -dc keys.raw 2>/dev/null || cat keys.raw) > all-keys.asc
gpg --import all-keys.asc 2>&1 | tail -1
gpg --verify swift-6.4.1-RELEASE-debian13.tar.gz.sig swift-6.4.1-RELEASE-debian13.tar.gz 2>&1 | tee gpg.txt
grep -q 52BB7E3DE28A71BE22EC05FFEF80A866B47A981F <(gpg --verify --status-fd 1 swift-6.4.1-RELEASE-debian13.tar.gz.sig swift-6.4.1-RELEASE-debian13.tar.gz 2>/dev/null | tr -d ' ') || gpg --verify swift-6.4.1-RELEASE-debian13.tar.gz.sig swift-6.4.1-RELEASE-debian13.tar.gz 2>&1 | grep -i "52BB 7E3D\|52BB7E3D"
echo "3273c647a4e29231e1d0962a170f4ba14eca06675591dfe384e7c22b0e476a47  swift-6.4.1-RELEASE-debian13.tar.gz" | sha256sum -c
apt-get install -y -qq libncurses6 libcurl4 libxml2 libedit2 libsqlite3-0 libz3-4 >/dev/null 2>&1 || (apt-get update -qq && apt-get install -y -qq libncurses6 libcurl4 libxml2 libedit2 libsqlite3-0 libz3-4 >/dev/null)
tar xzf swift-6.4.1-RELEASE-debian13.tar.gz
cat > goldens.swift <<'SW'
struct X {}
struct Y {}
struct FooBar<A, B> {}
enum Gunbc { enum v2 { enum test { enum drf_provider { struct DrfDeclared {} } } } }
typealias T1 = X
typealias T2 = (X: X, Y: Y)
typealias T3 = FooBar<X, Y>
typealias T4 = Optional<X>
typealias T5 = Optional<Optional<X>>
typealias T6 = (X) -> Y
typealias T7 = Gunbc.v2.test.drf_provider.DrfDeclared
SW
swift-6.4.1-RELEASE-debian13/usr/bin/swiftc -typecheck goldens.swift && echo "SWIFTC TYPECHECK OK"
