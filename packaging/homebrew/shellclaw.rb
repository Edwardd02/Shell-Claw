class Shellclaw < Formula
  desc "Local-first LLM-powered shell completion copilot"
  homepage "https://github.com/Edwardd02/Shell-Claw"
  version "0.0.1"

  if OS.mac? && Hardware::CPU.arm?
    url "https://github.com/Edwardd02/Shell-Claw/releases/download/v0.0.1/shellclaw-aarch64-apple-darwin.tar.gz"
    sha256 "8f7c5f03c95297bb5898e847eed90e059c07dddebf4d36fdc07b4d6036d6e74f"
  else
    odie "ShellClaw v0.0.1 currently supports Apple Silicon only"
  end

  def install
    bin.install "shellclaw"
    # 把 hook 脚本放到 share/shellclaw/ 下,供 shell 加载
    (share/"shellclaw").install "shellclaw.zsh"
    (share/"shellclaw").install "shellclaw.bash"
    # 模型自动下载脚本
    (share/"shellclaw").install "scripts/download-model.sh"
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
      opoo "Model download failed. Run 'shellclaw start' and then 'shellclaw model install' (if available) later."
    end
  end

  def caveats
    <<~EOS
      ShellClaw has been installed.

      To enable completions in your shell, source the hook. For Zsh, add to ~/.zshrc:
        source #{opt_share}/shellclaw/shellclaw.zsh
      For Bash, add to ~/.bashrc:
        source #{opt_share}/shellclaw/shellclaw.bash

      Then start the daemon:
        shellclaw start

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
