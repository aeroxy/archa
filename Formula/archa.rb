class Archa < Formula
  desc "Local-first Agent session reader and explorer with an embedded web UI"
  homepage "https://github.com/aeroxy/archa"
  version "0.1.2"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/aeroxy/archa/releases/download/#{version}/archa_macos_arm64.zip"
      sha256 "955dc56a050ef7aca80a1495056fc6d168e4df84b0ebd0bf273866172815f440"
    end
  end

  def install
    bin.install "archa"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/archa --version")
  end
end
