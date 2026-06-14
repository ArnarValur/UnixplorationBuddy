# Flatpak Sandboxing Polling Log Tailer

> **Recorded:** 2026-05-27 17:41
> **Status:** accepted

On Linux systems running Elite Dangerous inside flatpak Steam, sandboxing boundaries prevent inotify directory notification events (used by the standard `ed_journals` LiveLogDirReader) from crossing into the host namespace. This causes TUI real-time updates to stall. We replaced inotify-based file tailing with a custom filesystem-polling watcher thread that checks active log files and directory sizes every 250 milliseconds, seeking to the end on startup to monitor only new events, which is 100% reliable inside containerized Linux environments.
