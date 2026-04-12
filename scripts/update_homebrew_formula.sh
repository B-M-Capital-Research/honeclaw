#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat <<'EOF' >&2
Usage:
  scripts/update_homebrew_formula.sh \
    --version <version> \
    --darwin-aarch64-sha <sha256> \
    --darwin-x86_64-sha <sha256> \
    --linux-x86_64-sha <sha256>
EOF
  exit 1
}

VERSION=""
DARWIN_AARCH64_SHA=""
DARWIN_X86_64_SHA=""
LINUX_X86_64_SHA=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --version)
      VERSION="${2:-}"
      shift 2
      ;;
    --darwin-aarch64-sha)
      DARWIN_AARCH64_SHA="${2:-}"
      shift 2
      ;;
    --darwin-x86_64-sha)
      DARWIN_X86_64_SHA="${2:-}"
      shift 2
      ;;
    --linux-x86_64-sha)
      LINUX_X86_64_SHA="${2:-}"
      shift 2
      ;;
    *)
      usage
      ;;
  esac
done

if [[ -z "$VERSION" || -z "$DARWIN_AARCH64_SHA" || -z "$DARWIN_X86_64_SHA" || -z "$LINUX_X86_64_SHA" ]]; then
  usage
fi

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUTPUT_PATH="$ROOT_DIR/Formula/honeclaw.rb"

mkdir -p "$(dirname "$OUTPUT_PATH")"

cat > "$OUTPUT_PATH" <<EOF
class Honeclaw < Formula
  desc "CLI bundle for the Hone investment research assistant"
  homepage "https://github.com/B-M-Capital-Research/honeclaw"
  license "MIT"
  version "$VERSION"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/B-M-Capital-Research/honeclaw/releases/download/v$VERSION/honeclaw-darwin-aarch64.tar.gz"
      sha256 "$DARWIN_AARCH64_SHA"
    else
      url "https://github.com/B-M-Capital-Research/honeclaw/releases/download/v$VERSION/honeclaw-darwin-x86_64.tar.gz"
      sha256 "$DARWIN_X86_64_SHA"
    end
  end

  on_linux do
    url "https://github.com/B-M-Capital-Research/honeclaw/releases/download/v$VERSION/honeclaw-linux-x86_64.tar.gz"
    sha256 "$LINUX_X86_64_SHA"
  end

  def install
    libexec.install "bin", "share"

    (bin/"hone-cli").write <<~EOS
      #!/usr/bin/env bash
      set -euo pipefail

      HONE_HOME="\${HONE_HOME:-\$HOME/.honeclaw}"
      HONE_DATA_DIR="\${HONE_DATA_DIR:-\$HONE_HOME/data}"
      HONE_USER_CONFIG_PATH="\${HONE_USER_CONFIG_PATH:-\$HONE_HOME/config.yaml}"
      HONE_SKILLS_DIR="\${HONE_SKILLS_DIR:-#{libexec}/share/honeclaw/skills}"

      mkdir -p "\$HONE_DATA_DIR/runtime"

      if [[ "\$HONE_USER_CONFIG_PATH" == "\$HONE_HOME/config.yaml" && ! -f "\$HONE_USER_CONFIG_PATH" ]]; then
        cp "#{libexec}/share/honeclaw/config.example.yaml" "\$HONE_USER_CONFIG_PATH"
      fi

      if [[ ! -f "\$HONE_HOME/soul.md" ]]; then
        cp "#{libexec}/share/honeclaw/soul.md" "\$HONE_HOME/soul.md"
      fi

      export HONE_HOME
      export HONE_INSTALL_ROOT="#{libexec}"
      export HONE_USER_CONFIG_PATH
      export HONE_DATA_DIR
      export HONE_SKILLS_DIR

      exec "#{libexec}/bin/hone-cli" "\$@"
    EOS

    chmod 0755, bin/"hone-cli"
  end

  def caveats
    <<~EOS
      Hone stores user config in ~/.honeclaw/config.yaml and runtime data in ~/.honeclaw/data.

      Next steps:
        hone-cli doctor
        hone-cli onboard
        hone-cli start
    EOS
  end

  test do
    output = shell_output("#{bin}/hone-cli --help")
    assert_match "Hone CLI", output
  end
end
EOF
