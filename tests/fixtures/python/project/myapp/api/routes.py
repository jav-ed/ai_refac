# Absolute direct import — another file to verify all-files coverage.
from myapp.utils.formatters import format_date


def date_route(ts: int) -> str:
    return f"/date/{format_date(ts)}"
