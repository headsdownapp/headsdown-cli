# Homebrew formula for HeadsDown CLI
# This file is a template. The actual formula lives in the
# headsdown/homebrew-tap repository and is auto-updated by
# the release workflow.
#
# To install: brew install headsdown/tap/hd

class Hd < Formula
  desc "CLI tool for HeadsDown availability management"
  homepage "https://github.com/headsdownapp/headsdown-cli"
  version "0.1.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/headsdownapp/headsdown-cli/releases/download/v#{version}/hd-aarch64-darwin"
      sha256 "PLACEHOLDER"
    end

    on_intel do
      url "https://github.com/headsdownapp/headsdown-cli/releases/download/v#{version}/hd-x86_64-darwin"
      sha256 "PLACEHOLDER"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/headsdownapp/headsdown-cli/releases/download/v#{version}/hd-aarch64-linux"
      sha256 "PLACEHOLDER"
    end

    on_intel do
      url "https://github.com/headsdownapp/headsdown-cli/releases/download/v#{version}/hd-x86_64-linux"
      sha256 "PLACEHOLDER"
    end
  end

  def install
    bin.install "hd-*" => "hd"
  end

  test do
    assert_match "hd #{version}", shell_output("#{bin}/hd --version")
  end
end
