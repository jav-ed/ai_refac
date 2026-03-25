# Absolute direct import — Rope updates this.
from myapp.utils.formatters import format_currency


def render_balance(amount: float) -> str:
    return format_currency(amount)
