class Shellclaw < Formula
  desc "Local-first smart shell completion copilot"
  homepage "https://github.com/Edwardd02/Shell-Claw"
  version "0.0.1"

  if OS.mac? && Hardware::CPU.arm?
    url "https://github.com/Edwardd02/Shell-Claw/releases/download/v0.0.1/shellclaw-aarch64-apple-darwin.tar.gz"
    sha256 "883f503593e986d6469d031bfb4b546996e2c51a41cd37698656384a0848b055"
  else
    odie "ShellClaw v0.0.1 currently supports Apple Silicon only"
  end

  def install
    bin.install "shellclaw"
    # 模型自动下载脚本
    (share/"shellclaw").install "scripts/download-model.sh" if File.exist?("scripts/download-model.sh")
  end

  # brew install 后自动拉取模型(双源测速)。
  # 宽松策略:下载失败仅警告、不中断 install。
  def post_install
    ohai "Downloading ShellClaw model (please wait, may take a while)..."
    script = share/"shellclaw/download-model.sh"
    begin
      if script.exist?
        system "sh", script.to_s
      else
        opoo "download-model.sh not found in package"
      end
    rescue StandardError
      opoo "Model download failed. Run 'shellclaw model install' later."
    end
  end

  def caveats
    <<~EOS
      ShellClaw has been installed.

      The daemon service is available via:
        shellclaw start   # start daemon
        shellclaw status  # check status

      Config / log:
        shellclaw log on|off
        shellclaw status

      Uninstall:
        brew uninstall shellclaw
    EOS
  end

  test do
    system bin/"shellclaw", "status"
  end
end
