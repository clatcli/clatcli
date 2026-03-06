class Clat < Formula
  desc "Natural language shell assistant — describe what you want, get a script"
  homepage "https://github.com/OWNER/clat"

  # Update url and sha256 when cutting a release:
  #   sha256sum clat-X.Y.Z.tar.gz
  url "https://github.com/OWNER/clat/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "REPLACE_WITH_SHA256_OF_RELEASE_TARBALL"
  license "MIT"
  version "0.1.0"

  # Allow: brew install --HEAD OWNER/clat/clat
  head "https://github.com/OWNER/clat.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install",
           "--locked",
           "--root", prefix,
           "--path", "."
  end

  def caveats
    <<~EOS
      Config file: ~/.clat/config.toml
      Run `clat --init` to create it with defaults, then set your API URL and model:

        api_url = "http://localhost:1234/v1"   # LM Studio default
        model   = "your-model-name"

      List available models:
        clat --models

      Add to PATH if needed (Homebrew installs to #{opt_bin}):
        Already on PATH if Homebrew's bin is in your PATH.
    EOS
  end

  test do
    assert_match "clat", shell_output("#{bin}/clat --help")
  end
end
