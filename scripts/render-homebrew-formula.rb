#!/usr/bin/env ruby

require "fileutils"

version, arm_sha256, intel_sha256, output_path = ARGV
unless output_path
  abort "usage: #{$PROGRAM_NAME} VERSION ARM_SHA256 INTEL_SHA256 OUTPUT_PATH"
end

repository = ENV.fetch("SOURCE_REPOSITORY", "knothhe/scroll-split")

formula = <<~RUBY
  class Scrollsplit < Formula
    desc "Separate natural scrolling settings for a mouse and trackpad on macOS"
    homepage "https://github.com/#{repository}"
    version "#{version}"

    on_macos do
      if Hardware::CPU.arm?
        url "https://github.com/#{repository}/releases/download/v#{version}/scrollsplit-v#{version}-aarch64-apple-darwin.tar.gz"
        sha256 "#{arm_sha256}"
      else
        url "https://github.com/#{repository}/releases/download/v#{version}/scrollsplit-v#{version}-x86_64-apple-darwin.tar.gz"
        sha256 "#{intel_sha256}"
      end
    end

    def install
      bin.install "scrollsplit"
    end

    service do
      run [opt_bin/"scrollsplit", "run"]
      keep_alive true
      process_type :interactive
    end

    test do
      assert_match version.to_s, shell_output("\#{bin}/scrollsplit --version")
    end
  end
RUBY

FileUtils.mkdir_p(File.dirname(output_path))
File.write(output_path, formula)
