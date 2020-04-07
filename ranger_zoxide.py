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
            output = subprocess.check_output(["zoxide", "query"] + self.args[1:])
            output = output.decode("utf-8")

            directory = output.strip()
            self.fm.cd(directory)
            self.fm.notify(directory)
        except subprocess.CalledProcessError as e:
            if e.returncode == 1:
                pass
        except Exception as e:
            self.fm.notify(e, bad=True)

