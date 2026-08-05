class SmartShellCopilot < Formula
  desc "Local-first terminal completion copilot"
  homepage "https://github.com/smart-shell-copilot/smart-shell-copilot"
  version "0.1.0"

  if OS.mac?
    if Hardware::CPU.arm?
      url "https://github.com/smart-shell-copilot/smart-shell-copilot/releases/download/v0.1.0/smart-shell-copilot-aarch64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER"
    else
      url "https://github.com/smart-shell-copilot/smart-shell-copilot/releases/download/v0.1.0/smart-shell-copilot-x86_64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER"
    end
  elsif OS.linux?
    url "https://github.com/smart-shell-copilot/smart-shell-copilot/releases/download/v0.1.0/smart-shell-copilot-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "PLACEHOLDER"
  end

  depends_on "rust" => :build

  service do
    if OS.mac?
      run [opt_bin/"smart-shell-copilot-daemon"]
      keep_alive true
      log_path var/"log/smart-shell-copilot.log"
      error_log_path var/"log/smart-shell-copilot.error.log"
      environment_variables(
        SSC_SOCKET_PATH: "#{Dir.home}/.smart-shell-copilot/daemon.sock",
        SSC_MODEL_PATH: "#{opt_prefix}/share/smart-shell-copilot/models/qwen3-0.6b-base.gguf",
        SSC_DATA_DIR: "#{Dir.home}/.smart-shell-copilot"
      )
      working_dir Dir.home
    elsif OS.linux?
      require "formula"
      system "systemctl", "--user", "start", "smart-shell-copilot"
    end
  end

  def install
    bin.install "smart-shell-copilot-daemon"

    share.install "models" => "smart-shell-copilot/models"

    zsh_hook_dir = share/"smart-shell-copilot/shell/zsh"
    zsh_hook_dir.install "shell/zsh/smart-shell-copilot.zsh"

    bash_hook_dir = share/"smart-shell-copilot/shell/bash"
    bash_hook_dir.install "shell/bash/smart-shell-copilot.bash"

    install_shell_hooks
    start_service
  end

  def install_shell_hooks
    if OS.mac?
      site_zsh = HOMEBREW_PREFIX/"share/zsh/site-functions"
      site_zsh.mkpath unless site_zsh.exist?
      hook_source = share/"smart-shell-copilot/shell/zsh/smart-shell-copilot.zsh"
      site_zsh.install_symlink hook_source => "_smart-shell-copilot" unless (site_zsh/"_smart-shell-copilot").exist?

      etc_bash = HOMEBREW_PREFIX/"etc/bash_completion.d"
      etc_bash.mkpath unless etc_bash.exist?
      hook_source_bash = share/"smart-shell-copilot/shell/bash/smart-shell-copilot.bash"
      etc_bash.install_symlink hook_source_bash => "smart-shell-copilot" unless (etc_bash/"smart-shell-copilot").exist?
    end
  end

  def start_service
    if OS.mac?
      ohai "Starting Smart Shell Copilot service..."
      system "launchctl", "load", "-w", "#{Dir.home}/Library/LaunchAgents/com.smart-shell-copilot.daemon.plist" rescue nil
    elsif OS.linux?
      system "systemctl", "--user", "enable", "smart-shell-copilot" rescue nil
      system "systemctl", "--user", "start", "smart-shell-copilot" rescue nil
    end
  end

  def uninstall
    stop_service
    remove_shell_hooks
    remove_service_registration
  end

  def stop_service
    if OS.mac?
      system "launchctl", "unload", "#{Dir.home}/Library/LaunchAgents/com.smart-shell-copilot.daemon.plist" rescue nil
    elsif OS.linux?
      system "systemctl", "--user", "stop", "smart-shell-copilot" rescue nil
      system "systemctl", "--user", "disable", "smart-shell-copilot" rescue nil
    end
  end

  def remove_shell_hooks
    if OS.mac?
      site_zsh = HOMEBREW_PREFIX/"share/zsh/site-functions/_smart-shell-copilot"
      site_zsh.unlink if site_zsh.symlink?
      site_zsh.delete if site_zsh.exist?

      etc_bash = HOMEBREW_PREFIX/"etc/bash_completion.d/smart-shell-copilot"
      etc_bash.unlink if etc_bash.symlink?
      etc_bash.delete if etc_bash.exist?
    end
  end

  def remove_service_registration
    if OS.mac?
      plist = "#{Dir.home}/Library/LaunchAgents/com.smart-shell-copilot.daemon.plist"
      File.delete(plist) if File.exist?(plist)
    elsif OS.linux?
      system "systemctl", "--user", "disable", "smart-shell-copilot" rescue nil
    end
  end

  def caveats
    <<~EOS
      Smart Shell Copilot has been installed.

      The daemon service has been started automatically.
      Open a new terminal to activate the shell hook.

      To uninstall:
        brew uninstall smart-shell-copilot

      Configuration:
        SSC_SOCKET_PATH: ~/.smart-shell-copilot/daemon.sock
        SSC_MODEL_PATH: #{opt_prefix}/share/smart-shell-copilot/models/
        Logs: ~/.smart-shell-copilot/daemon.log
    EOS
  end

  test do
    system bin/"smart-shell-copilot-daemon", "--version"
  end
end
