class Archa < Formula
  desc "Local-first Agent session reader and explorer with an embedded web UI"
  homepage "https://github.com/aeroxy/archa"
  version "0.1.1"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/aeroxy/archa/releases/download/#{version}/archa_macos_arm64.zip"
      sha256 "eb517d7efbfa42f431e09e2a2dbfbb3250ecdca9ef498c5c7973ea0ab53006b2"
    end
  end

  def install
    bin.install "archa"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/archa --version")
  end
end
