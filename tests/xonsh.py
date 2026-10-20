# Behavioral checks for the generated Xonsh integration, run by src/shell.rs.

import asyncio
import builtins
import contextlib
import io
import subprocess
import sys
from types import SimpleNamespace
from unittest.mock import patch

from prompt_toolkit.application import Application
from prompt_toolkit.application.current import set_app
from prompt_toolkit.buffer import Buffer, CompletionState
from prompt_toolkit.completion import Completion
from prompt_toolkit.document import Document
from prompt_toolkit.enums import EditingMode
from prompt_toolkit.input import DummyInput
from prompt_toolkit.key_binding import KeyBindings
from prompt_toolkit.layout import Layout, Window
from prompt_toolkit.layout.controls import BufferControl
from prompt_toolkit.output import DummyOutput
from xonsh.aliases import Aliases
from xonsh.events import EventManager

source = sys.stdin.read()


async def check(late):
    """Run before/after-PTK initialization against real alias and binding objects."""
    events = EventManager()
    events.transmogrify("on_ptk_create", "LoadEvent")
    aliases = Aliases()
    bindings = KeyBindings()
    namespace = {}
    buffer = Buffer()
    app = Application(
        layout=Layout(Window(BufferControl(buffer=buffer))),
        input=DummyInput(),
        output=DummyOutput(),
    )
    with patch.object(builtins, "events", events), patch.object(
        builtins, "aliases", aliases
    ), patch.object(builtins.__xonsh__.commands_cache, "aliases", aliases), set_app(
        app
    ):
        if late:
            events.on_ptk_create.fire(bindings=bindings)
        exec(source, namespace)
        handler = namespace["__zoxide_keybindings"]
        original_names = set(aliases)
        assert len(original_names) in (0, 2)
        if not late:
            events.on_ptk_create.fire(bindings=bindings)
        exec(source, namespace)
        assert set(aliases) == original_names
        assert len(bindings.bindings) == 1
        assert namespace["__zoxide_keybindings"] is handler
        # A manually assigned alias also enables completion with --no-cmd.
        aliases["jump"] = namespace["__zoxide_z"]
        aliases["go"] = namespace["__zoxide_z"]
        binding = bindings.bindings[0]

        def document(text, cursor=None):
            buffer.document = Document(text, len(text) if cursor is None else cursor)

        for text in (
            "jump ",
            "jump keyword",
            "jump keyword/",
            "other keyword ",
            "jump 'unfinished ",
            "jump keyword; jump other ",
            "echo ignored | jump keyword ",
            "jump keyword > output ",
            "jump $(echo keyword) ",
            "jump @(print('side effect')) ",
            "jump $HOME ",
            'jump "$HOME" ',
            "jump '$HOME/path' ",
            'jump r"$HOME" ',
            'jump "~/path" ',
            "jump r'~/path' ",
            "jump *.py ",
            "jump foo\\ bar ",
            "jump f'{1}' ",
            "jump b'bytes' ",
            "jump keyword # comment ",
            "jump keyword\n ",
        ):
            document(text)
            assert not binding.filter(), text

        document("jump keyword ", 5)
        assert not binding.filter()
        document("jump keyword ")
        buffer.complete_state = CompletionState(buffer.document, [Completion("x")])
        assert not binding.filter()
        buffer.complete_state = None
        app.editing_mode = EditingMode.VI
        from prompt_toolkit.key_binding.vi_state import InputMode

        app.vi_state.input_mode = InputMode.NAVIGATION
        assert not binding.filter()
        app.vi_state.input_mode = InputMode.INSERT
        assert binding.filter()
        app.editing_mode = EditingMode.EMACS

        def dangerous_alias(args):
            raise AssertionError("alias must not execute during completion")

        dangerous_alias.return_what = "command"
        aliases["other"] = dangerous_alias
        document("other keyword ")
        assert not binding.filter()

        async def select(
            text, stdout=b"/selected\n", status=0, error=None, changed=None
        ):
            document(text)
            original = buffer.document
            assert binding.filter(), text

            def run(argv, **kwargs):
                assert argv[:6] == [
                    "zoxide-test",
                    "query",
                    "-i",
                    "--exclude",
                    "/cwd",
                    "--",
                ]
                assert kwargs["env"] == {"ZOXIDE_TEST": "env"}
                assert kwargs["stdout"] == subprocess.PIPE
                if changed is not None:
                    document(changed)
                if error is not None:
                    raise error
                return subprocess.CompletedProcess(argv, status, stdout)

            with patch.dict(
                namespace,
                __zoxide_bin=lambda: "zoxide-test",
                __zoxide_pwd=lambda: "/cwd",
                __zoxide_env=lambda: {"ZOXIDE_TEST": "env"},
            ), patch("subprocess.run", side_effect=run) as process:
                binding.handler(SimpleNamespace(current_buffer=buffer))
                assert not binding.filter(), "a second selector must not be queued"
                # run_in_terminal schedules the callback on the event loop.
                await asyncio.sleep(0)
                await asyncio.sleep(0)
                assert process.call_count == 1
                argv = process.call_args.args[0]
            return original, argv

        path = "/space café/quotes'\"/$HOME/back\\slash"
        _, argv = await select("  jump 'two words' café -x ", (path + "\n").encode())
        assert argv[6:] == ["two words", "café", "-x"]
        assert buffer.text == "  jump @(" + repr(path) + ") "

        # Execute the inserted argument through Xonsh itself, not a Python decoder.
        seen = []

        def capture(args):
            seen.extend(args)

        aliases["capture"] = capture
        with builtins.__xonsh__.env.swap(TITLE=""):
            builtins.__xonsh__.execer.exec(
                "capture" + buffer.text[len("  jump") :] + "\n", glbs={}
            )
        assert seen == [path], seen

        for status, stdout in ((1, b""), (130, b""), (0, b""), (0, b"\n")):
            original, _ = await select("jump keyword ", stdout, status)
            assert buffer.document == original
            assert binding.filter(), "cancellation must permit another attempt"
        with contextlib.redirect_stderr(io.StringIO()) as errors:
            original, _ = await select("jump keyword ", error=OSError("missing binary"))
            assert buffer.document == original
            assert "missing binary" in errors.getvalue()
            original, _ = await select("jump keyword ", stdout=b"\xff\n")
            assert buffer.document == original
        await select("jump keyword ", changed="typed meanwhile")
        assert buffer.text == "typed meanwhile"
        buffer.insert_text(" more")
        assert buffer.text == "typed meanwhile more"

        # Reinitialization updates the callable and retains a single binding.
        exec(source, namespace)
        assert len(bindings.bindings) == 1
        assert namespace["__zoxide_keybindings"] is handler
        document("go keyword ")
        assert (
            binding.filter()
        ), "an existing manual alias must survive reinitialization"
        for name in original_names:
            document(name + " keyword ")
            if name.endswith("i"):
                assert not binding.filter()
            else:
                assert binding.filter()
        document("jump keyword ")
        assert binding.filter()
        aliases["jump"] = dangerous_alias
        assert not binding.filter()


asyncio.run(check(False))
asyncio.run(check(True))
