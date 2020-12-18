import inspect


def _task_name_to_kebab_case():
    task_name = inspect.stack()[1][3]
    assert task_name.startswith(
        "task_"
    ), "this function must be called from a doit task"

    return "-".join(task_name.split("_")[1:])


def task_build_native():
    pass


def task_build_release():
    pass


def task_run_native():
    return {
        "basename": _task_name_to_kebab_case(),
        "actions": ["cargo run --color always --features native"],
    }


def task_run_native_release():
    pass


def task_build_wasm():
    pass


def task_build_wasm_release():
    pass


def task_run_wasm():
    pass


def task_run_wasm_release():
    pass
