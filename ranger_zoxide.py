import os.path
import ranger.api
import ranger.core.fm
import ranger.ext.signals
import subprocess

hook_init_old = ranger.api.hook_init


def hook_init(fm: ranger.core.fm.FM):
    def zoxide_add(signal: ranger.ext.signals.Signal):
        path = signal.new.path
        process = subprocess.Popen(["zoxide", "add", path])
        process.wait()

    fm.signal_bind("cd", zoxide_add)
    return hook_init_old(fm)


ranger.api.hook_init = hook_init


class z(ranger.api.commands.Command):
    def execute(self):
        try:
            zoxide = subprocess.Popen(
                ["zoxide", "query"] + self.args[1:],
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
            )
            stdout, stderr = zoxide.communicate()

            if zoxide.returncode == 0:
                output = stdout.decode("utf-8").strip()
                if output:
                    self.fm.notify(output)
                    if os.path.isdir(output):
                        self.fm.cd(output)
                else:
                    self.fm.notify("zoxide: unexpected exit", bad=True)
            else:
                output = stderr.decode("utf-8").strip() or "zoxide: unknown error"
                self.fm.notify(output, bad=True)

        except Exception as e:
            self.fm.notify(e, bad=True)
