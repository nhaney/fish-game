"""Wraps cargo commands to make it easier to do specific tasks for both native and web build targets"""
import os

from doit.action import CmdAction
from functools import wraps
from pathlib import Path
from typing import Callable, List

### Utils
def convert_to_kebab(func, *args, **kwargs) -> Callable:
    """converts doit task to use kebab case"""

    @wraps(func)
    def wrapper():
        assert func.__name__.startswith("task_"), "this decorator must wrap a doit task"
        task_dict = func()
        task_dict["basename"] = "-".join(func.__name__.split("_")[1:])

        return task_dict

    return wrapper


def get_src_files() -> List[Path]:
    """gets all of the src files to pass as dependencies to build tasks"""
    src_directory = Path("./src")

    return [file for file in src_directory.glob("**/*.rs")]


def get_build_target(is_release: bool, is_wasm: bool) -> str:
    return (
        f"./target/{'wasm32-unknown-unknown/' if is_wasm else ''}{'release' if is_release else 'debug'}"
        f"/fish-game{'.wasm' if is_wasm else ''}"
    )


### BUILD TASKS
@convert_to_kebab
def task_build_native() -> dict:
    """Builds native binary"""
    return build(is_release=False, is_wasm=False)


@convert_to_kebab
def task_build_release() -> dict:
    """Builds release native binary"""
    return build(is_release=True, is_wasm=False)


@convert_to_kebab
def task_build_wasm() -> dict:
    """Builds debug wasm file"""
    return build(is_release=False, is_wasm=True)


@convert_to_kebab
def task_build_wasm_release() -> dict:
    """Builds release wasm file"""
    return build(is_release=True, is_wasm=True)


def build(is_release: bool, is_wasm: bool) -> dict:
    actions = [["cargo", "build", "--color", "always"]]

    target = get_build_target(is_release, is_wasm)

    if is_wasm:
        actions[0] += ["--target", "wasm32-unknown-unknown", "--features", "web"]
        actions.append(
            [
                "wasm-bindgen",
                "--out-dir",
                "./target",
                "--target",
                "web",
                "--no-typescript",
                target,
            ]
        )

        actions.append(
            [
                "zip",
                "build-wasm.zip",
                "index.html",
                "target/fish-game.js",
                "target/fish-game_bg.wasm",
                "-r",
                "assets"
            ]
        )
    else:
        actions[0] += ["--features", "native"]

    if is_release:
        actions[0] += ["--release"]

    return {
        "actions": actions,
        "file_dep": get_src_files(),
        "targets": [target],
    }


### RUN TASKS
@convert_to_kebab
def task_run_native() -> dict:
    """Runs the game natively in debug mode after building it"""
    return run_game(is_release=False, is_wasm=False)


@convert_to_kebab
def task_run_native_release() -> dict:
    """Runs the game natively in release mode after building it"""
    return run_game(is_release=True, is_wasm=False)


@convert_to_kebab
def task_run_wasm() -> dict:
    """Runs a server that will host the debug wasm build at localhost:8000"""
    return run_game(is_release=False, is_wasm=True)


@convert_to_kebab
def task_run_wasm_release() -> dict:
    """Runs a server that will host the release wasm build at localhost:8000"""
    return run_game(is_release=True, is_wasm=True)


def run_game(is_release: bool, is_wasm: bool) -> dict:
    target = get_build_target(is_release, is_wasm)

    if is_wasm:
        cmd = ["python3", "-m", "http.server"]
    else:
        cmd = ["cargo", "run", "--features", "native"]
        if is_release:
            cmd.append("--release")

    return {
        "actions": [cmd],
        "file_dep": [target],
        "uptodate": [False],
    }


# linting/testing
def task_test() -> dict:
    return {
        "actions": ["cargo", "test"],
    }


def task_lint() -> dict:
    """Runs linting"""
    return {
        "actions": [["cargo", "clippy", "--color", "always"]],
    }


def task_fmt() -> dict:
    """Formats the code"""
    return {
        "actions": [["cargo", "fmt", "--", "--check"]],
    }


def task_check():
    """Runs linting, format checks, and tests"""

    # fail on warnings
    cur_env = os.environ.copy()
    cur_env["RUSTFLAGS"] = "-Dwarnings"

    return {
        "actions": [
            ["cargo", "fmt", "--", "--check"],
            CmdAction(["cargo", "clippy", "--color", "always"], env=cur_env),
            ["cargo", "test"],
        ],
        "file_dep": get_src_files(),
    }
