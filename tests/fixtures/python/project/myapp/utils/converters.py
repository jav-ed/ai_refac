# Absolute direct import — Rope updates this to myapp.core.formatters.
from myapp.utils.formatters import format_date


def to_display(ts: int) -> str:
    return f"date={format_date(ts)}"
