class Shellclaw < Formula
  desc "Local-first LLM-powered shell completion copilot"
  homepage "https://github.com/Edwardd02/Shell-Claw"
  version "0.0.3"

  if OS.mac? && Hardware::CPU.arm?
    url "https://github.com/Edwardd02/Shell-Claw/releases/download/v0.0.3/shellclaw-aarch64-apple-darwin.tar.gz"
    sha256 "8dd992dbfb1c00094bb22e95adbcb3eeb62899b9d279eb10c8311b05eb218f90"
  else
    odie "ShellClaw v0.0.3 currently supports Apple Silicon only"
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
        opoo "Model download failed. Run 'brew postinstall shellclaw' to retry." unless system "sh", script.to_s
      else
        opoo "download-model.sh not found in package"
      end
    rescue StandardError
      opoo "Model download failed. Run 'brew postinstall shellclaw' to retry."
    end
    # Upgrades must replace an already-running daemon from the previous Cellar.
    system bin/"shellclaw", "stop"
    opoo "Daemon setup failed. Run 'shellclaw setup #{share}/shellclaw'." unless system bin/"shellclaw", "setup", (share/"shellclaw").to_s
  end

  def caveats
    <<~EOS
      ShellClaw has been installed.

      The Zsh hook was added to ~/.zshrc and the daemon was started.
      Open a new terminal to load the Zsh hook. This is required after an
      upgrade so the old session does not retain outdated key bindings.

      Bash support is experimental and is not enabled automatically.

      Config / log:
        shellclaw log on|off
        shellclaw status

      Uninstall:
        shellclaw stop
        brew uninstall shellclaw
    EOS
  end

  service do
    run [opt_bin/"shellclaw", "daemon"]
    keep_alive true
    process_type :background
    working_dir var
    log_path var/"log/shellclaw.log"
    error_log_path var/"log/shellclaw.error.log"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/shellclaw --version")
  end
end
