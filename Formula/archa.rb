class Archa < Formula
  desc "Local-first Agent session reader and explorer with an embedded web UI"
  homepage "https://github.com/aeroxy/archa"
  version "0.2.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/aeroxy/archa/releases/download/#{version}/archa_macos_arm64.zip"
      sha256 "40644c4cf7fcd59d88b028191e55b08f3d8b1b81750ad6bc172528ecca3f3e3d"
    end
  end

  def install
    bin.install "archa"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/archa --version")
  end
end
