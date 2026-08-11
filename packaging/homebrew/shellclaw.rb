class Shellclaw < Formula
  desc "Local-first smart shell completion copilot"
  homepage "https://github.com/Edwardd02/Shell-Claw"
  version "0.0.1"

  if OS.mac?
    if Hardware::CPU.arm?
      url "https://github.com/Edwardd02/Shell-Claw/releases/download/v0.0.1/shellclaw-aarch64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER"
    else
      url "https://github.com/Edwardd02/Shell-Claw/releases/download/v0.0.1/shellclaw-x86_64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER"
    end
  elsif OS.linux?
    url "https://github.com/Edwardd02/Shell-Claw/releases/download/v0.0.1/shellclaw-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "PLACEHOLDER"
  end

  depends_on "rust" => :build

  def install
    bin.install "shellclaw"

    share.install "models" => "shellclaw/models" if File.exist?("models")
    share.install "shell" => "shellclaw/shell" if File.exist?("shell")

    install_shell_hooks
    start_service
  end

  def install_shell_hooks
    if OS.mac?
      site_zsh = HOMEBREW_PREFIX/"share/zsh/site-functions"
      site_zsh.mkpath unless site_zsh.exist?
      hook_source = share/"shellclaw/shell/zsh/shellclaw.zsh"
      site_zsh.install_symlink hook_source => "_shellclaw" unless (site_zsh/"_shellclaw").exist?

      etc_bash = HOMEBREW_PREFIX/"etc/bash_completion.d"
      etc_bash.mkpath unless etc_bash.exist?
      bash_hook = share/"shellclaw/shell/bash/shellclaw.bash"
      etc_bash.install_symlink bash_hook => "shellclaw" unless (etc_bash/"shellclaw").exist?
    end
  end

  def start_service
    if OS.mac?
      ohai "Starting ShellClaw service..."
      system "launchctl", "load", "-w", "#{Dir.home}/Library/LaunchAgents/com.shellclaw.daemon.plist" rescue nil
    elsif OS.linux?
      system "systemctl", "--user", "enable", "shellclaw" rescue nil
      system "systemctl", "--user", "start", "shellclaw" rescue nil
    end
  end

  def uninstall
    stop_service
    remove_shell_hooks
  end

  def stop_service
    system "launchctl", "unload", "#{Dir.home}/Library/LaunchAgents/com.shellclaw.daemon.plist" rescue nil if OS.mac?
    system "systemctl", "--user", "stop", "shellclaw" rescue nil if OS.linux?
    system "systemctl", "--user", "disable", "shellclaw" rescue nil if OS.linux?
  end

  def remove_shell_hooks
    if OS.mac?
      z = HOMEBREW_PREFIX/"share/zsh/site-functions/_shellclaw"
      z.unlink if z.symlink?
      z.delete if z.exist?

      b = HOMEBREW_PREFIX/"etc/bash_completion.d/shellclaw"
      b.unlink if b.symlink?
      b.delete if b.exist?
    end
  end

  def caveats
    <<~EOS
      ShellClaw has been installed.

      The daemon service has been started.
      Open a new terminal to activate the shell hook.

      To install the model, run:
        shellclaw model install   # 或按 docs 指引下载 GGUF 到 ~/.shellclaw/models/

      Config / log control:
        shellclaw log on|off      # 开启/关闭文件日志
        shellclaw status          # 查看 daemon 状态

      Uninstall:
        brew uninstall shellclaw
    EOS
  end

  test do
    system bin/"shellclaw", "status"
  end
end
