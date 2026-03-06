class Clat < Formula
  desc "Natural language shell assistant — describe what you want, get a script"
  homepage "https://github.com/clatcli/clatcli"

  # Update url and sha256 when cutting a release:
  #   curl -sL https://github.com/clatcli/clatcli/archive/refs/tags/vX.Y.Z.tar.gz | shasum -a 256
  url "https://github.com/clatcli/clatcli/archive/refs/tags/v0.1.2.tar.gz"
  sha256 "c81c9c7ba02d71f3fd37cd355736bd1da16240d466af8dec1e2f680c1c7d5d7c"
  license "MIT"
  version "0.1.2"

  # Allow: brew install --HEAD clatcli/clat/clat
  head "https://github.com/clatcli/clatcli.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install",
           "--locked",
           "--root", prefix,
           "--path", "."
  end

  def caveats
    <<~EOS
      Config file: ~/.clat/config.toml (created automatically on first run)
      Edit it to set your API URL and model:

        api_url = "http://localhost:1234/v1"   # LM Studio default
        model   = "your-model-name"

      List available models:
        clat -l

      Add to PATH if needed (Homebrew installs to #{opt_bin}):
        Already on PATH if Homebrew's bin is in your PATH.
    EOS
  end

  test do
    assert_match "clat", shell_output("#{bin}/clat --help")
  end
end
